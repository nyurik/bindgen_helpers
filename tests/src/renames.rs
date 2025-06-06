use bindgen_helpers::{rename_enum, Renamer};
use insta::assert_snapshot;

use crate::test;

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

    assert_snapshot!(test(cb, header), @r"
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

    assert_snapshot!(test(cb, header), @r"
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

    assert_snapshot!(test(cb, header), @r"
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
