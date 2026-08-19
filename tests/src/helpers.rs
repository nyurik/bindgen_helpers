use std::fs::write;

use bindgen_helpers::{define_enum, rename_enum, Case, IdentRenamer, Regex};
use insta::assert_snapshot;

use crate::builder;

fn write_helper_config(name: &str, content: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bindgen_helpers_config_{}_{name}.toml",
        std::process::id(),
    ));
    write(&path, content).expect("Failed to write helper config");
    path
}

#[test]
fn test_helpers_rename_enum() {
    let mut helpers = builder(
        r"
enum my_enum {
    I_SAID_YES,
    I_SAID_NO,
};
",
    );
    rename_enum!(helpers, "my_enum" => "MyEnum", remove: "^I_SAID_");

    assert_snapshot!(helpers.into_string().unwrap(), @r"
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum {
        Yes = 0,
        No = 1,
    }
    ");
}

#[test]
fn test_helpers_define_enum_write() {
    let mut helpers = builder(
        r"
#define ERR_FOO 1
#define ERR_BAR 2
",
    );
    define_enum!(helpers, ErrorCode, r"^ERR_", remove: "^ERR_");

    assert_snapshot!(helpers.into_string().unwrap(), @"
    pub const ERR_FOO: u32 = 1;
    pub const ERR_BAR: u32 = 2;

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum ErrorCode {
        Foo = (ERR_FOO as u32),
        Bar = (ERR_BAR as u32),
    }
    ");
}

#[test]
fn test_helpers_rename_many() {
    let mut helpers = builder(
        r"
enum foo_status {
    FOO_STATUS_OK,
    FOO_STATUS_ERR,
};

enum foo_mode {
    FOO_MODE_AUTO,
    FOO_MODE_MANUAL,
};
",
    );
    helpers.rename_many(
        Regex::new("foo_.*").unwrap(),
        IdentRenamer {
            remove: Some(vec![Regex::new("^foo_").unwrap()]),
            case: Some(Case::Pascal),
            ..IdentRenamer::default()
        },
    );

    assert_snapshot!(helpers.into_string().unwrap(), @r"
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum Status {
        FOO_STATUS_OK = 0,
        FOO_STATUS_ERR = 1,
    }
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum Mode {
        FOO_MODE_AUTO = 0,
        FOO_MODE_MANUAL = 1,
    }
    ");
}

#[test]
fn test_helpers_write_to_file() {
    let path = std::env::temp_dir()
        .join(format!("bindgen_helpers_test_{}.rs", std::process::id()));
    builder("#define FOO 1")
        .write_to_file(&path)
        .expect("Failed to write generated bindings to file");

    let output = std::fs::read_to_string(&path)
        .expect("Failed to read generated bindings file");
    let _ = std::fs::remove_file(path);

    assert_snapshot!(output, @"pub const FOO: u32 = 1;
");
}

#[test]
fn test_helpers_with_config_file() {
    let path = write_helper_config(
        "with_config_file",
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
    let helpers = bindgen_helpers::BindingsBuilder::with_config_file(
        bindgen_helpers::Builder::default()
            .header_contents(
                "test.h",
                r"
enum my_enum {
    I_SAID_YES,
    I_SAID_NO,
};

#define ERR_FOO 1
#define ERR_BAR 2
",
            )
            .disable_header_comment(),
        &path,
    )
    .expect("Failed to apply helper config");

    let output = helpers.into_string().expect("Failed to generate bindings");
    let _ = std::fs::remove_file(path);

    assert_snapshot!(output, @r"
    pub const ERR_FOO: u32 = 1;
    pub const ERR_BAR: u32 = 2;
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum {
        Yes = 0,
        No = 1,
    }

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum ErrorCode {
        Foo = (ERR_FOO as u32),
        Bar = (ERR_BAR as u32),
    }
    ");
}

#[test]
fn test_helpers_apply_config_file() {
    let path = write_helper_config(
        "apply_config_file",
        r#"
[[rename_item]]
c = "my_struct"
rust = "MyStruct"
"#,
    );
    let mut helpers = builder(
        r"
struct my_struct {
    int a;
};
",
    );
    helpers
        .apply_config_file(&path)
        .expect("Failed to apply helper config");

    let output = helpers.into_string().expect("Failed to generate bindings");
    let _ = std::fs::remove_file(path);

    assert_snapshot!(output, @r###"
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct MyStruct {
        pub a: ::std::os::raw::c_int,
    }
    #[allow(clippy::unnecessary_operation, clippy::identity_op)]
    const _: () = {
        ["Size of MyStruct"][::std::mem::size_of::<MyStruct>() - 4usize];
        ["Alignment of MyStruct"][::std::mem::align_of::<MyStruct>() - 4usize];
        ["Offset of field: MyStruct::a"]
            [::std::mem::offset_of!(MyStruct, a) - 0usize];
    };
    "###);
}
