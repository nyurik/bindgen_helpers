use bindgen_helpers::{define_enum, Renamer};
use insta::assert_snapshot;

use crate::test_with_define_enums;

#[test]
fn test_define_enum_minimal() {
    let header = r"
#define ERR_FOO 1
#define ERR_BAR 2
#define OTHER_VALUE 12
";
    let mut cb = Renamer::new(true);
    define_enum!(cb, ErrorCode, r"^ERR_");

    assert_snapshot!(test_with_define_enums(&cb, header), @"
    pub const ERR_FOO: u32 = 1;
    pub const ERR_BAR: u32 = 2;
    pub const OTHER_VALUE: u32 = 12;

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum ErrorCode {
        ErrFoo = (ERR_FOO as u32),
        ErrBar = (ERR_BAR as u32),
    }
    ");
}

#[test]
fn test_define_enum_min_max() {
    let header = r"
#define ERR_TOO_LOW -1
#define ERR_LOW 0
#define ERR_MID 2
#define ERR_HIGH 3
#define ERR_TOO_HIGH 4
";
    let mut cb = Renamer::new(true);
    define_enum!(
        cb,
        ErrorCode,
        r"^ERR_",
        repr = i32,
        min: 0,
        max: 3,
        remove: "^ERR_",
    );

    assert_snapshot!(test_with_define_enums(&cb, header), @"
    pub const ERR_TOO_LOW: i32 = -1;
    pub const ERR_LOW: u32 = 0;
    pub const ERR_MID: u32 = 2;
    pub const ERR_HIGH: u32 = 3;
    pub const ERR_TOO_HIGH: u32 = 4;

    #[repr(i32)]
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum ErrorCode {
        Low = (ERR_LOW as i32),
        Mid = (ERR_MID as i32),
        High = (ERR_HIGH as i32),
    }
    ");
}

#[test]
fn test_define_enum_all_options() {
    let header = r"
#define ERR_WOULD_BLOCK 11
#define ERR_BAR 2
#define ERR_PRIVATE 20
#define ERR_FOO 1
#define OTHER_VALUE 12
";
    let mut cb = Renamer::new(true);
    define_enum!(
        cb,
        ErrorCode,
        r"^ERR_",
        repr = u32,
        exclude: "_PRIVATE$",
        sort: Value,
        derive: [Debug, Copy, Clone, PartialEq, Eq, serde::Serialize],
        remove: "^ERR_",
        case: Pascal,
        "WOULD_BLOCK" => "WouldBlock",
    );

    assert_snapshot!(test_with_define_enums(&cb, header), @r"
    pub const ERR_WOULD_BLOCK: u32 = 11;
    pub const ERR_BAR: u32 = 2;
    pub const ERR_PRIVATE: u32 = 20;
    pub const ERR_FOO: u32 = 1;
    pub const OTHER_VALUE: u32 = 12;

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize)]
    pub enum ErrorCode {
        Foo = (ERR_FOO as u32),
        Bar = (ERR_BAR as u32),
        WouldBlock = (ERR_WOULD_BLOCK as u32),
    }
    ");
}

#[test]
fn test_define_enum_sort_by_name() {
    let header = r"
#define ERR_ZULU 1
#define ERR_ALPHA 3
#define ERR_MIDDLE 2
";
    let mut cb = Renamer::new(true);
    define_enum!(
        cb,
        ErrorCode,
        r"^ERR_",
        sort: Name,
        remove: "^ERR_",
    );

    assert_snapshot!(test_with_define_enums(&cb, header), @"
    pub const ERR_ZULU: u32 = 1;
    pub const ERR_ALPHA: u32 = 3;
    pub const ERR_MIDDLE: u32 = 2;

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum ErrorCode {
        Alpha = (ERR_ALPHA as u32),
        Middle = (ERR_MIDDLE as u32),
        Zulu = (ERR_ZULU as u32),
    }
    ");
}

#[test]
fn test_define_enum_sort_by_value_desc() {
    let header = r"
#define ERR_LOW 1
#define ERR_HIGH 3
#define ERR_MIDDLE 2
";
    let mut cb = Renamer::new(true);
    define_enum!(
        cb,
        ErrorCode,
        r"^ERR_",
        sort: ValueDesc,
        remove: "^ERR_",
    );

    assert_snapshot!(test_with_define_enums(&cb, header), @"
    pub const ERR_LOW: u32 = 1;
    pub const ERR_HIGH: u32 = 3;
    pub const ERR_MIDDLE: u32 = 2;

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum ErrorCode {
        High = (ERR_HIGH as u32),
        Middle = (ERR_MIDDLE as u32),
        Low = (ERR_LOW as u32),
    }
    ");
}
