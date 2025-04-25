#![cfg(test)]

use bindgen_helpers::{rename_enum, Builder, Renamer};
use insta::assert_snapshot;

#[test]
fn test_renames() {
    let header = r"
enum my_enum {
	I_SAID_YES,
	I_SAID_NO,
	I_SAID_RENAME_IT,
	I_DID_NOT_SAY_ANYTHING,
};
";

    let mut cb = Renamer::new(true);
    rename_enum!(cb, "my_enum" => "MyEnum");

    assert_snapshot!(run(cb, header), @r"
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum {
        ISaidYes = 0,
        ISaidNo = 1,
        ISaidRenameIt = 2,
        IDidNotSayAnything = 3,
    }
    ");

    let mut cb = Renamer::new(true);
    rename_enum!(cb, "my_enum" => "MyEnum", remove: "^I_SAID_", remove: "DID_NOT" , remove: "_ANYTHING$", remove: "^I_SAID_(RENAME_)?", "RENAME_IT" => "Renamed", "YES" => "Maybe");

    assert_snapshot!(run(cb, header), @r"
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum {
        Maybe = 0,
        No = 1,
        Renamed = 2,
        ISay = 3,
    }
    ");

    let mut cb = Renamer::new(true);
    rename_enum!(cb, "my_enum" => "MyEnum", case: Snake);

    assert_snapshot!(run(cb, header), @r"
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum {
        i_said_yes = 0,
        i_said_no = 1,
        i_said_rename_it = 2,
        i_did_not_say_anything = 3,
    }
    ");
}

fn run(cb: Renamer, header: &str) -> String {
    let mut output = Vec::new();
    Builder::default()
        .header_contents("test.h", header)
        .disable_header_comment() // version keeps changing
        .rustified_enum(cb.get_regex_str())
        .parse_callbacks(Box::new(cb))
        .generate()
        .expect("Failed to generate bindings")
        .write(Box::new(&mut output))
        .expect("Failed to write bindings");

    String::from_utf8(output).expect("Output was not valid UTF-8")
}
