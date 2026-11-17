use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A temp file that is removed on drop, so failing assertions do not leak it.
struct TempFile(PathBuf);

impl TempFile {
    fn new(name: &str, content: &str) -> Self {
        let path = std::env::temp_dir()
            .join(format!("bindgen_helpers_cli_{}_{name}", std::process::id()));
        fs::write(&path, content).expect("failed to write temp test file");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn bindgen_helper() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bindgen-helper"))
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout was not UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn cli_generates_plain_bindgen_output_without_config() {
    let header = TempFile::new("plain.h", "#define FOO_VALUE 1\n");

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--disable-header-comment")
        .output()
        .expect("failed to run bindgen-helper");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "pub const FOO_VALUE: u32 = 1;\n");
    assert!(stderr(&output).contains("warning: no --helper-config provided"));
}

#[test]
fn cli_generates_plain_bindgen_enum_output_without_config() {
    let header = TempFile::new(
        "plain_enum.h",
        "enum plain_enum { PLAIN_A, PLAIN_B };\n",
    );

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--disable-header-comment")
        .output()
        .expect("failed to run bindgen-helper");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "pub const plain_enum_PLAIN_A: plain_enum = 0;\n\
         pub const plain_enum_PLAIN_B: plain_enum = 1;\n\
         pub type plain_enum = ::std::os::raw::c_uint;\n"
    );
}

const HELPER_HEADER: &str = r"
enum my_enum {
    I_SAID_YES,
    I_SAID_NO,
};

#define ERR_FOO 1
#define ERR_BAR 2
";

const HELPER_CONFIG: &str = r#"
[[rename_enum]]
c = "my_enum"
rust = "MyEnum"
remove = ["^I_SAID_"]

[[define_enum]]
name = "ErrorCode"
match = "^ERR_"
remove = ["^ERR_"]
"#;

fn assert_helper_output(stdout: &str) {
    assert!(stdout.contains("pub enum MyEnum"));
    assert!(stdout.contains("Yes = 0"));
    assert!(stdout.contains("No = 1"));
    assert!(stdout.contains("pub enum ErrorCode"));
    assert!(stdout.contains("Foo = 1,"));
    assert!(stdout.contains("Bar = 2,"));
}

#[test]
fn cli_define_enum_does_not_need_allowlisted_constants() {
    let header = TempFile::new("allowlist.h", HELPER_HEADER);
    let config = TempFile::new("allowlist.toml", HELPER_CONFIG);

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--disable-header-comment")
        .arg("--allowlist-type")
        .arg("my_enum")
        .arg("--helper-config")
        .arg(config.path())
        .output()
        .expect("failed to run bindgen-helper");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(!stdout.contains("pub const ERR_FOO"), "{stdout}");
    assert_helper_output(&stdout);
}

#[test]
fn cli_writes_debug_output_to_stderr() {
    let header = TempFile::new("debug.h", HELPER_HEADER);
    let config =
        TempFile::new("debug.toml", &format!("debug = true\n{HELPER_CONFIG}"));

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--disable-header-comment")
        .arg("--helper-config")
        .arg(config.path())
        .output()
        .expect("failed to run bindgen-helper");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(!stdout.contains("warning"), "{stdout}");
    assert_helper_output(&stdout);
    assert!(
        stderr(&output).contains("warning: unrecognized item ERR_FOO"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn cli_applies_helper_config_and_bindgen_flags() {
    let header = TempFile::new("helper.h", HELPER_HEADER);
    let config = TempFile::new("helper.toml", HELPER_CONFIG);

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--disable-header-comment")
        .arg("--no-layout-tests")
        .arg("--allowlist-item")
        .arg("my_enum|ERR_.*")
        .arg("--helper-config")
        .arg(config.path())
        .output()
        .expect("failed to run bindgen-helper");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_helper_output(&stdout(&output));
}

#[test]
fn cli_accepts_equals_form_and_output_file() {
    let header = TempFile::new("equals.h", HELPER_HEADER);
    let config = TempFile::new("equals.toml", HELPER_CONFIG);
    let out = TempFile::new("equals_out.rs", "");

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--disable-header-comment")
        .arg("--no-layout-tests")
        .arg("-o")
        .arg(out.path())
        .arg(format!("--helper-config={}", config.path().display()))
        .output()
        .expect("failed to run bindgen-helper");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "", "output went to stdout despite -o");
    assert_helper_output(
        &fs::read_to_string(out.path()).expect("failed to read output file"),
    );
}

#[test]
fn cli_rejects_invalid_helper_config() {
    let header = TempFile::new("invalid.h", "#define ERR_FOO 1\n");
    let config = TempFile::new(
        "invalid.toml",
        r#"
[[define_enum]]
name = "ErrorCode"
match = "^ERR_"
case = "NotACase"
"#,
    );

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--disable-header-comment")
        .arg("--helper-config")
        .arg(config.path())
        .output()
        .expect("failed to run bindgen-helper");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("error: unknown case `NotACase`"));
}

#[test]
fn cli_rejects_unknown_config_keys() {
    let header = TempFile::new("typo.h", "#define ERR_FOO 1\n");
    let config = TempFile::new(
        "typo.toml",
        r#"
[[define_enum]]
name = "ErrorCode"
match = "^ERR_"
remov = ["^ERR_"]
"#,
    );

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--helper-config")
        .arg(config.path())
        .output()
        .expect("failed to run bindgen-helper");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unknown field `remov`"));
}

#[test]
fn cli_reports_missing_helper_config() {
    let header = TempFile::new("missing_cfg.h", "#define ERR_FOO 1\n");

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--helper-config")
        .arg(std::env::temp_dir().join("bindgen_helpers_cli_no_such_file.toml"))
        .output()
        .expect("failed to run bindgen-helper");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to read helper config"));
}

#[test]
fn cli_rejects_duplicate_helper_config() {
    let header = TempFile::new("dup.h", "#define ERR_FOO 1\n");
    let config = TempFile::new("dup.toml", "");

    let output = bindgen_helper()
        .arg(header.path())
        .arg("--helper-config")
        .arg(config.path())
        .arg("--helper-config")
        .arg(config.path())
        .output()
        .expect("failed to run bindgen-helper");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("may only be provided once"));
}

#[test]
fn cli_keeps_existing_output_when_helper_config_is_invalid() {
    const EXISTING: &str = "// existing bindings\n";
    let header = TempFile::new("keep.h", "#define ERR_FOO 1\n");
    let bad_regex = TempFile::new(
        "keep_regex.toml",
        "[[define_enum]]\nname = \"E\"\nmatch = \"[\"\n",
    );
    let bad_toml = TempFile::new("keep_toml.toml", "not = [valid");
    let missing =
        std::env::temp_dir().join("bindgen_helpers_cli_keep_none.toml");

    for config in [bad_regex.path(), bad_toml.path(), &missing] {
        let out = TempFile::new("keep_out.rs", EXISTING);

        let output = bindgen_helper()
            .arg(header.path())
            .arg("-o")
            .arg(out.path())
            .arg("--helper-config")
            .arg(config)
            .output()
            .expect("failed to run bindgen-helper");

        assert!(!output.status.success(), "{}", config.display());
        assert_eq!(
            fs::read_to_string(out.path()).expect("failed to read output"),
            EXISTING,
            "output file was modified for {}",
            config.display()
        );
    }
}
