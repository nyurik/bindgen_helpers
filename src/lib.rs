#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

pub use bindgen::*;
pub use convert_case::Case;
pub use regex::Regex;

mod renamer;
pub use renamer::*;

mod define_enum;
pub use define_enum::*;

mod helpers;
pub use helpers::*;

/// Macro to help define renaming rules for an enum and its values.
/// See an example in the [`BindingsBuilder`] documentation.
#[macro_export]
macro_rules! rename_enum {
    (
        $cb:expr,
        $c_name:literal => $rust_name:literal
        $(, remove: $remove:literal)*
        $(, case: $case:ident)?
        $(, $itm:literal => $ren:literal)*
        $(,)?
    ) => {
        $cb.rename_item($c_name, $rust_name);
        #[allow(clippy::needless_update)]
        $cb.rename_enum_val(
            // See https://github.com/rust-lang/rust-bindgen/issues/3113#issuecomment-2844178132
            Some(concat!("^(enum )?", $c_name, "$")),
            $crate::IdentRenamer {
                remove: {
                    let patterns: Vec<&str> = vec![$($remove),*];
                    if patterns.is_empty() {
                        None
                    } else {
                        Some(
                            patterns
                                .into_iter()
                                .map(|v| $crate::Regex::new(v).expect("Unable to compile regex for remove parameter"))
                                .collect()
                        )
                    }
                },
                $( case: Some($crate::Case::$case), )?
                renames: vec![$( ($itm.into(), $ren.into()), )*].into_iter().collect(),
                ..$crate::IdentRenamer::default_case($crate::Case::Pascal)
            }
        );
    };
}

/// Macro to collect matching integer `#define` constants into a Rust enum.
#[macro_export]
macro_rules! define_enum {
    (
        $cb:expr,
        $rust_name:ident,
        $c_name:literal
        $(, repr = $repr:ident)?
        $(, min: $min:literal)?
        $(, max: $max:literal)?
        $(, exclude: $exclude:literal)*
        $(, sort: $sort:ident)?
        $(, derive: [$($derive:path),* $(,)?])?
        $(, remove: $remove:literal)*
        $(, case: $case:ident)?
        $(, $itm:literal => $ren:literal)*
        $(,)?
    ) => {
        #[allow(clippy::needless_update)]
        let define_enum = $crate::DefineEnum::new(
            stringify!($rust_name),
            $crate::Regex::new($c_name).expect("Unable to compile regex for define enum"),
            $crate::IdentRenamer {
                remove: {
                    let patterns: Vec<&str> = vec![$($remove),*];
                    if patterns.is_empty() {
                        None
                    } else {
                        Some(
                            patterns
                                .into_iter()
                                .map(|v| $crate::Regex::new(v).expect("Unable to compile regex for remove parameter"))
                                .collect()
                        )
                    }
                },
                $( case: Some($crate::Case::$case), )?
                renames: vec![$( ($itm.into(), $ren.into()), )*].into_iter().collect(),
                ..$crate::IdentRenamer::default_case($crate::Case::Pascal)
            },
        );
        $( let define_enum = define_enum.with_repr(stringify!($repr)); )?
        $( let define_enum = define_enum.sort($crate::DefineEnumSort::$sort); )?
        $(
            let define_enum = define_enum.derives(
                vec![$( stringify!($derive).replace(" :: ", "::"), )*]
            );
        )?
        $( let define_enum = define_enum.min($min); )?
        $( let define_enum = define_enum.max($max); )?
        $(
            let define_enum = define_enum.exclude(
                $crate::Regex::new($exclude).expect("Unable to compile regex for exclude parameter")
            );
        )*
        $cb.define_enum(define_enum);
    };
}
