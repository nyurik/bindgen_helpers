use std::collections::HashMap;

use convert_case::{Case, Casing as _};
use regex::{Regex, RegexSet};

#[derive(Debug, Default)]
pub struct IdentRenamer {
    /// Any regexes to remove substrings from the value. Only first matching is applied. Applied before any explicit renames.
    pub remove: Option<RegexSet>,
    /// Additional explicit renames. If a match is found, skips automatic case change.
    pub renames: HashMap<String, String>,
    /// Which case to convert the value to, unless explicitly renamed.
    pub case: Option<Case>,
}

impl IdentRenamer {
    #[must_use]
    pub fn default_case(case: Case) -> Self {
        Self {
            case: Some(case),
            ..Default::default()
        }
    }

    fn apply(&self, val: &str) -> String {
        let mut val = val.to_owned();
        if let Some(remove) = &self.remove {
            // remove first matching string from val
            if let Some(idx) = remove.matches(val.as_ref()).iter().next() {
                // expect regex to pass - it was already added to RegexSet
                let re = Regex::new(&remove.patterns()[idx]).unwrap();
                val = re.replace(&val, "").to_string();
            }
        }
        if let Some(new_val) = self.renames.get(val.as_str()) {
            new_val.to_string()
        } else if let Some(case) = self.case {
            val.to_case(case)
        } else {
            val
        }
    }
}

#[derive(Debug, Default)]
pub struct Renamer {
    /// Enable debug output
    debug: bool,
    /// Rename C items like enums, structs, and aliases, replacing them with a new name.
    item_renames: HashMap<String, String>,
    /// Rename C items like enums, structs, and aliases that match a regex, and apply a renamer.
    /// The regex string must not contain '^' or '$' symbols.
    item_renames_ext: Vec<(Regex, IdentRenamer)>,
    /// Matches C enum names (i.e. "enum foo").
    /// Note that the regex might be None because the callback might also not have it for some enums.
    enum_renames: Vec<(Option<Regex>, IdentRenamer)>,
}

impl Renamer {
    #[must_use]
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            ..Default::default()
        }
    }

    /// Get a regex string that matches all configured C items
    #[must_use]
    pub fn get_regex_str(&self) -> String {
        self.item_renames_ext
            .iter()
            .map(|(re, _)| re.as_str())
            .chain(self.item_renames.keys().map(String::as_str))
            .fold(String::new(), |mut acc, re| {
                if !acc.is_empty() {
                    acc.push('|');
                }
                acc.push_str(re);
                acc
            })
    }

    pub fn rename_item(&mut self, c_name: impl AsRef<str>, rust_name: impl AsRef<str>) {
        self.item_renames
            .insert(c_name.as_ref().into(), rust_name.as_ref().into());
    }

    /// Rename any C item, including enums and structs.
    ///
    /// # Panics
    /// Will panic if the regex contains '^' or '$' symbols.
    pub fn rename_many(&mut self, c_name: Regex, renamer: IdentRenamer) {
        assert!(
            !c_name.as_str().contains('^'),
            "Regex must not contain '^' symbol"
        );
        assert!(
            !c_name.as_str().contains('$'),
            "Regex must not contain '$' symbol"
        );
        self.item_renames_ext.push((c_name, renamer));
    }

    /// Rename enum values. Make sure `enum_c_name` is in the form `enum some_enum_name`.
    ///
    /// # Panics
    /// Will panic if the `enum_c_name` is not a valid regex.
    pub fn rename_enum_val(&mut self, enum_c_name: Option<&str>, val_renamer: IdentRenamer) {
        self.enum_renames.push((
            enum_c_name.map(|v| Regex::new(v).expect("Invalid enum_c_name regex")),
            val_renamer,
        ));
    }
}

impl bindgen::callbacks::ParseCallbacks for Renamer {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        value: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        self.enum_renames
            .iter()
            .filter_map(|(re, rn)| match (enum_name, re) {
                (Some(enum_name), Some(re)) if re.is_match(enum_name) => Some(rn),
                (None, None) => Some(rn),
                _ => None,
            })
            .map(|rn| rn.apply(value))
            .next()
            .or_else(|| {
                if self.debug {
                    let name = enum_name.unwrap();
                    println!("cargo::warning=Unrecognized enum variant {name} :: {value}");
                }
                None
            })
    }

    fn item_name(&self, item_name: &str) -> Option<String> {
        self.item_renames
            .get(item_name)
            .map(ToString::to_string)
            .or_else(|| {
                self.item_renames_ext
                    .iter()
                    .filter_map(|(re, rn)| {
                        if re.is_match(item_name) {
                            Some(rn)
                        } else {
                            None
                        }
                    })
                    .map(|rn| rn.apply(item_name))
                    .next()
            })
            .or_else(|| {
                if self.debug {
                    println!("cargo::warning=Unrecognized item {item_name}");
                }
                None
            })
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_get_regex_str() {
        let mut cb = Renamer::new(false);
        cb.rename_item("bar", "baz");
        cb.rename_many(Regex::new(r"foo.*").unwrap(), IdentRenamer::default());
        cb.rename_many(Regex::new("bas").unwrap(), IdentRenamer::default());
        assert_snapshot!(cb.get_regex_str(), @"foo.*|bas|bar");
    }
}
