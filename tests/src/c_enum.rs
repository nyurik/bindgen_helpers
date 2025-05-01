use bindgen_helpers::{rename_enum, IdentRenamer, Renamer};
use insta::assert_snapshot;

use crate::test;

#[test]
fn test_enum_types() {
    let mut cb = Renamer::new(true);
    rename_enum!(cb, "my_enum1" => "MyEnum1", remove: "fo");
    rename_enum!(cb, "my_enum2" => "MyEnum2", remove: "fo");
    rename_enum!(cb, "my_enum3" => "MyEnum3", remove: "fo");
    rename_enum!(cb, "my_enum4a" => "MyEnum4a", remove: "fo");
    rename_enum!(cb, "my_enum4b" => "MyEnum4b", remove: "fo");

    assert_snapshot!(test(cb, r"
enum my_enum1 { foo1 };
typedef enum { foo2 } my_enum2;
typedef enum my_enum3 { foo3 } my_enum3;
typedef enum my_enum4a { foo4 } my_enum4b;
"), @r"
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum1 {
        O1 = 0,
    }
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum2 {
        O2 = 0,
    }
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum3 {
        O3 = 0,
    }
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyEnum4a {
        O4 = 0,
    }
    pub use self::MyEnum4a as MyEnum4b;
    ");
}

#[test]
fn test_remove_to_reserved() {
    let mut cb = Renamer::new(true);
    rename_enum!(cb, "my_fn_enum" => "MyFn", remove: "foo", case: Lower);

    assert_snapshot!(test(cb, r"
typedef enum { foo_fn } my_fn_enum;
"), @r"
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum MyFn {
        fn_ = 0,
    }
    ");
}

#[test]
fn test_remove_edges() {
    let mut cb = Renamer::new(true);
    cb.rename_enum_val(None, IdentRenamer::default());

    assert_snapshot!(test(cb, r"
enum my_enum1 { foo1 };
typedef enum { foo2 } my_enum2;
typedef enum my_enum3 { foo3 } my_enum3;
typedef enum my_enum4a { foo4 } my_enum4b;
"), @r"
    pub const my_enum1_foo1: my_enum1 = 0;
    pub type my_enum1 = ::std::os::raw::c_uint;
    pub const my_enum2_foo2: my_enum2 = 0;
    pub type my_enum2 = ::std::os::raw::c_uint;
    pub const my_enum3_foo3: my_enum3 = 0;
    pub type my_enum3 = ::std::os::raw::c_uint;
    pub const my_enum4a_foo4: my_enum4a = 0;
    pub type my_enum4a = ::std::os::raw::c_uint;
    pub use self::my_enum4a as my_enum4b;
    ");
}
