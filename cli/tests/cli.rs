use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join(format!("bindgen_helpers_cli_{}_{name}", std::process::id()))
}

fn write_temp(name: &str, content: &str) -> PathBuf {
    let path = temp_file(name);
    fs::write(&path, content).expect("failed to write temp test file");
    path
}

fn bindgen_helper() -> Command {
    let mut command = Command::new(env!("CARGO"));
    command.args(["run", "--quiet", "--bin", "bindgen-helper"]);
    command
}

#[test]
fn cli_generates_plain_bindgen_output_without_config() {
    let header = write_temp("plain.h", "#define FOO_VALUE 1\n");

    let output = bindgen_helper()
        .arg(&header)
        .arg("--disable-header-comment")
        .output()
        .expect("failed to run bindgen-helper");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout was not UTF-8"),
        "pub const FOO_VALUE: u32 = 1;\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("warning: no --helper-config provided"));
    let _ = fs::remove_file(header);
}

#[test]
fn cli_applies_helper_config_and_bindgen_flags() {
    let header = write_temp(
        "helper.h",
        r"
enum my_enum {
    I_SAID_YES,
    I_SAID_NO,
};

#define ERR_FOO 1
#define ERR_BAR 2
",
    );
    let config = write_temp(
        "helper.toml",
        r#"
[[rename_enum]]
c = "my_enum"
rust = "MyEnum"
remove = ["^I_SAID_"]

[[define_enum]]
name = "ErrorCode"
match = "^ERR_"
remove = ["^ERR_"]
"#,
    );

    let output = bindgen_helper()
        .arg(&header)
        .arg("--disable-header-comment")
        .arg("--no-layout-tests")
        .arg("--allowlist-item")
        .arg("my_enum|ERR_.*")
        .arg("--helper-config")
        .arg(&config)
        .output()
        .expect("failed to run bindgen-helper");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout =
        String::from_utf8(output.stdout).expect("stdout was not UTF-8");
    assert!(stdout.contains("pub enum MyEnum"));
    assert!(stdout.contains("Yes = 0"));
    assert!(stdout.contains("No = 1"));
    assert!(stdout.contains("pub enum ErrorCode"));
    assert!(stdout.contains("Foo = (ERR_FOO as u32)"));
    assert!(stdout.contains("Bar = (ERR_BAR as u32)"));
    let _ = fs::remove_file(header);
    let _ = fs::remove_file(config);
}

#[test]
fn cli_rejects_invalid_helper_config() {
    let header = write_temp("invalid.h", "#define ERR_FOO 1\n");
    let config = write_temp(
        "invalid.toml",
        r#"
[[define_enum]]
name = "ErrorCode"
match = "^ERR_"
case = "NotACase"
"#,
    );

    let output = bindgen_helper()
        .arg(&header)
        .arg("--disable-header-comment")
        .arg("--helper-config")
        .arg(&config)
        .output()
        .expect("failed to run bindgen-helper");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown case"));
    let _ = fs::remove_file(header);
    let _ = fs::remove_file(config);
}
