#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

pub use bindgen::*;
pub use convert_case::Case;
pub use regex::RegexSet;

mod renamer;
pub use renamer::*;

#[macro_export]
macro_rules! rename_enum {
    ( $cb:expr,
      $c_name:literal => $rust_name:literal
      $(, remove: $remove:literal)*
      $(, case: $case:ident)?
      $(, $itm:literal => $ren:literal)*
      $(,)?
    ) => {
        $cb.rename_item($c_name, $rust_name);
        #[allow(clippy::needless_update)]
        $cb.rename_enum_val(
            Some(concat!("enum ", $c_name)),
            $crate::IdentRenamer {
                remove: {
                    let patterns: Vec<&str> = vec![$($remove),*];
                    if patterns.is_empty() {
                        None
                    } else {
                        Some($crate::RegexSet::new(&patterns).expect("Unable to compile regex set for remove parameter"))
                    }
                },
                $( case: Some($crate::Case::$case), )?
                renames: vec![$( ($itm.into(), $ren.into()), )*].into_iter().collect(),
                ..$crate::IdentRenamer::default_case($crate::Case::Pascal)
            }
        );
    };
}
