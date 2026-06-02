use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::{
    BindingsBuilder, Case, DefineEnum, DefineEnumSort, IdentRenamer, Regex,
};

/// Helper-specific configuration loaded by the `bindgen-helper` CLI.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HelperConfig {
    /// Enable debug warnings from the helper parse callbacks.
    #[serde(default)]
    pub debug: bool,
    /// Exact item rename rules.
    #[serde(default)]
    pub rename_item: Vec<RenameItemConfig>,
    /// Regex item rename rules.
    #[serde(default)]
    pub rename_many: Vec<RenameManyConfig>,
    /// Enum and enum variant rename rules.
    #[serde(default)]
    pub rename_enum: Vec<RenameEnumConfig>,
    /// Enum variant rename rules without renaming the enum item itself.
    #[serde(default)]
    pub rename_enum_value: Vec<RenameEnumValueConfig>,
    /// Define-backed enum generation rules.
    #[serde(default)]
    pub define_enum: Vec<DefineEnumConfig>,
}

/// Exact C item rename.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenameItemConfig {
    /// C item name as reported by bindgen.
    pub c: String,
    /// Rust item name to generate.
    pub rust: String,
}

/// Regex-based C item rename.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenameManyConfig {
    /// Regex matching C item names. Must not contain `^` or `$`, matching the programmatic API.
    #[serde(rename = "match")]
    pub matcher: String,
    /// Identifier renaming rules for matching items.
    #[serde(flatten)]
    pub renamer: IdentRenamerConfig,
}

/// C enum and enum variant rename.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenameEnumConfig {
    /// C enum name to rename.
    pub c: String,
    /// Rust enum name to generate.
    pub rust: String,
    /// Identifier renaming rules for enum variants.
    #[serde(flatten)]
    pub renamer: IdentRenamerConfig,
}

/// Enum variant rename without an enum item rename.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenameEnumValueConfig {
    /// Regex matching the C enum name. Use `null` for unnamed enum callbacks.
    pub enum_match: Option<String>,
    /// Identifier renaming rules for enum variants.
    #[serde(flatten)]
    pub renamer: IdentRenamerConfig,
}

/// Define-backed enum generation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefineEnumConfig {
    /// Rust enum name to generate.
    pub name: String,
    /// Regex matching integer macro names to collect.
    #[serde(rename = "match")]
    pub matcher: String,
    /// Optional explicit repr type.
    pub repr: Option<String>,
    /// Include only values greater than or equal to this value.
    pub min: Option<i64>,
    /// Include only values less than or equal to this value.
    pub max: Option<i64>,
    /// Regexes matching macro names to exclude.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Optional generated variant sort order.
    pub sort: Option<String>,
    /// Optional derives for the generated enum.
    #[serde(default, rename = "derive")]
    pub derives: Option<Vec<String>>,
    /// Identifier renaming rules for generated variants.
    #[serde(flatten)]
    pub renamer: IdentRenamerConfig,
}

/// Identifier rename pipeline used by multiple helper rule types.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentRenamerConfig {
    /// Regexes to remove before explicit renames and case conversion.
    #[serde(default)]
    pub remove: Vec<String>,
    /// Case conversion to apply after removals, unless an explicit rename matches.
    pub case: Option<String>,
    /// Explicit renames after removals.
    #[serde(default)]
    pub renames: HashMap<String, String>,
}

/// Error returned when loading or applying helper config.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HelperConfigError {
    /// Reading the TOML file failed.
    #[error("failed to read helper config `{path}`")]
    Read {
        /// Path that failed to read.
        path: String,
        /// Source IO error.
        #[source]
        source: std::io::Error,
    },
    /// Parsing TOML failed.
    #[error("failed to parse helper config `{path}`")]
    Parse {
        /// Path that failed to parse.
        path: String,
        /// Source TOML parse error.
        #[source]
        source: toml::de::Error,
    },
    /// A regex field could not be compiled.
    #[error("invalid regex in `{field}`: `{value}`")]
    Regex {
        /// Config field name.
        field: &'static str,
        /// Invalid regex string.
        value: String,
        /// Source regex parse error.
        #[source]
        source: regex::Error,
    },
    /// `rename_many.match` violated the existing API invariant.
    #[error("`rename_many.match` must not contain `^` or `$`: `{0}`")]
    AnchoredRenameMany(String),
    /// The config named an unsupported case.
    #[error("unknown case `{0}`")]
    UnknownCase(String),
    /// The config named an unsupported define enum sort.
    #[error("unknown define enum sort `{0}`")]
    UnknownSort(String),
}

impl HelperConfig {
    /// Load helper config from a TOML file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_path(
        path: impl AsRef<Path>,
    ) -> Result<Self, HelperConfigError> {
        let path = path.as_ref();
        let path_display = path.display().to_string();
        let content = std::fs::read_to_string(path).map_err(|source| {
            HelperConfigError::Read {
                path: path_display.clone(),
                source,
            }
        })?;
        toml::from_str(&content).map_err(|source| HelperConfigError::Parse {
            path: path_display,
            source,
        })
    }

    /// Apply this config to an existing helper builder.
    ///
    /// # Errors
    /// Returns an error if any regex, case, or sort setting is invalid.
    pub fn apply_to(
        &self,
        helpers: &mut BindingsBuilder,
    ) -> Result<(), HelperConfigError> {
        for rename in &self.rename_item {
            helpers.rename_item(&rename.c, &rename.rust);
        }

        for rename in &self.rename_many {
            if rename.matcher.contains('^') || rename.matcher.contains('$') {
                return Err(HelperConfigError::AnchoredRenameMany(
                    rename.matcher.clone(),
                ));
            }
            helpers.rename_many(
                regex("rename_many.match", &rename.matcher)?,
                rename.renamer.to_ident_renamer(None)?,
            );
        }

        for rename in &self.rename_enum {
            helpers.rename_item(&rename.c, &rename.rust);
            let enum_match = format!("^(enum )?{}$", rename.c);
            helpers.rename_enum_val(
                Some(&enum_match),
                rename.renamer.to_ident_renamer(Some(Case::Pascal))?,
            );
        }

        for rename in &self.rename_enum_value {
            if let Some(enum_match) = &rename.enum_match {
                regex("rename_enum_value.enum_match", enum_match)?;
            }
            helpers.rename_enum_val(
                rename.enum_match.as_deref(),
                rename.renamer.to_ident_renamer(Some(Case::Pascal))?,
            );
        }

        for define in &self.define_enum {
            let mut define_enum = DefineEnum::new(
                &define.name,
                regex("define_enum.match", &define.matcher)?,
                define.renamer.to_ident_renamer(Some(Case::Pascal))?,
            );
            if let Some(repr) = &define.repr {
                define_enum = define_enum.with_repr(repr);
            }
            if let Some(min) = define.min {
                define_enum = define_enum.min(min);
            }
            if let Some(max) = define.max {
                define_enum = define_enum.max(max);
            }
            for exclude in &define.exclude {
                define_enum =
                    define_enum.exclude(regex("define_enum.exclude", exclude)?);
            }
            if let Some(sort) = &define.sort {
                define_enum = define_enum.sort(parse_sort(sort)?);
            }
            if let Some(derives) = &define.derives {
                define_enum = define_enum.derives(derives.clone());
            }
            helpers.define_enum(define_enum);
        }

        Ok(())
    }
}

impl IdentRenamerConfig {
    fn to_ident_renamer(
        &self,
        default_case: Option<Case<'static>>,
    ) -> Result<IdentRenamer, HelperConfigError> {
        let remove = self
            .remove
            .iter()
            .map(|value| regex("remove", value))
            .collect::<Result<Vec<_>, _>>()?;
        let case = match &self.case {
            Some(case) => parse_case_setting(case)?,
            None => default_case,
        };
        Ok(IdentRenamer {
            remove: if remove.is_empty() {
                None
            } else {
                Some(remove)
            },
            renames: self.renames.clone(),
            case,
        })
    }
}

fn regex(field: &'static str, value: &str) -> Result<Regex, HelperConfigError> {
    Regex::new(value).map_err(|source| HelperConfigError::Regex {
        field,
        value: value.to_owned(),
        source,
    })
}

fn parse_sort(value: &str) -> Result<DefineEnumSort, HelperConfigError> {
    match normalize_name(value).as_str() {
        "name" => Ok(DefineEnumSort::Name),
        "value" => Ok(DefineEnumSort::Value),
        "valuedesc" => Ok(DefineEnumSort::ValueDesc),
        _ => Err(HelperConfigError::UnknownSort(value.to_owned())),
    }
}

fn parse_case(value: &str) -> Result<Case<'static>, HelperConfigError> {
    match normalize_name(value).as_str() {
        "snake" => Ok(Case::Snake),
        "constant" | "uppersnake" | "screamingsnake" => Ok(Case::UpperSnake),
        "ada" => Ok(Case::Ada),
        "kebab" => Ok(Case::Kebab),
        "cobol" | "upperkebab" => Ok(Case::UpperKebab),
        "train" => Ok(Case::Train),
        "flat" => Ok(Case::Flat),
        "upperflat" => Ok(Case::UpperFlat),
        "pascal" | "uppercamel" => Ok(Case::Pascal),
        "camel" => Ok(Case::Camel),
        "lower" => Ok(Case::Lower),
        "upper" => Ok(Case::Upper),
        "title" => Ok(Case::Title),
        "sentence" => Ok(Case::Sentence),
        _ => Err(HelperConfigError::UnknownCase(value.to_owned())),
    }
}

fn parse_case_setting(
    value: &str,
) -> Result<Option<Case<'static>>, HelperConfigError> {
    match normalize_name(value).as_str() {
        "none" | "preserve" | "raw" => Ok(None),
        _ => parse_case(value).map(Some),
    }
}

fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|c| *c != '-' && *c != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "bindgen_helpers_config_{}_{name}",
            std::process::id()
        ))
    }

    #[test]
    fn config_defaults_to_empty() {
        let config: HelperConfig = toml::from_str("").unwrap();

        assert!(config.rename_item.is_empty());
        assert!(config.define_enum.is_empty());
    }

    #[test]
    fn from_path_reports_read_errors() {
        let path = temp_file("missing.toml");
        let _ = std::fs::remove_file(&path);

        assert!(matches!(
            HelperConfig::from_path(&path),
            Err(HelperConfigError::Read { .. })
        ));
    }

    #[test]
    fn from_path_reports_parse_errors() {
        let path = temp_file("invalid.toml");
        std::fs::write(&path, "not = [valid").unwrap();

        assert!(matches!(
            HelperConfig::from_path(&path),
            Err(HelperConfigError::Parse { .. })
        ));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn apply_rejects_anchored_rename_many() {
        let config: HelperConfig = toml::from_str(
            r#"
[[rename_many]]
match = "^bad"
"#,
        )
        .unwrap();
        let mut helpers =
            BindingsBuilder::new(bindgen::Builder::default(), false);

        assert!(matches!(
            config.apply_to(&mut helpers),
            Err(HelperConfigError::AnchoredRenameMany(value)) if value == "^bad"
        ));
    }

    #[test]
    fn apply_rejects_invalid_regex() {
        let config: HelperConfig = toml::from_str(
            r#"
[[define_enum]]
name = "ErrorCode"
match = "["
"#,
        )
        .unwrap();
        let mut helpers =
            BindingsBuilder::new(bindgen::Builder::default(), false);

        assert!(matches!(
            config.apply_to(&mut helpers),
            Err(HelperConfigError::Regex { field: "define_enum.match", value, .. }) if value == "["
        ));
    }

    #[test]
    fn apply_rejects_invalid_enum_value_match() {
        let config: HelperConfig = toml::from_str(
            r#"
[[rename_enum_value]]
enum_match = "["
"#,
        )
        .unwrap();
        let mut helpers =
            BindingsBuilder::new(bindgen::Builder::default(), false);

        assert!(matches!(
            config.apply_to(&mut helpers),
            Err(HelperConfigError::Regex { field: "rename_enum_value.enum_match", value, .. }) if value == "["
        ));
    }

    #[test]
    fn ident_renamer_config_can_preserve_case_without_removals() {
        let renamer = IdentRenamerConfig {
            case: Some("none".to_owned()),
            ..IdentRenamerConfig::default()
        }
        .to_ident_renamer(Some(Case::Pascal))
        .unwrap();

        assert!(renamer.remove.is_none());
        assert!(renamer.case.is_none());
    }

    #[test]
    fn apply_accepts_optional_rules() {
        let config: HelperConfig = toml::from_str(
            r#"
[[rename_many]]
match = "prefix_"
remove = ["^prefix_"]
case = "camel"

[[rename_enum_value]]
enum_match = "my_enum"
remove = ["^MY_ENUM_"]

[[define_enum]]
name = "ErrorCode"
match = "^ERR_"
repr = "i64"
min = -10
max = 10
exclude = ["^ERR_SKIP$"]
sort = "value_desc"
derive = ["Debug"]
remove = ["^ERR_"]
"#,
        )
        .unwrap();
        let mut helpers =
            BindingsBuilder::new(bindgen::Builder::default(), false);

        config.apply_to(&mut helpers).unwrap();
    }

    #[test]
    fn parses_case_names_flexibly() {
        assert!(matches!(parse_case("Pascal").unwrap(), Case::Pascal));
        assert!(matches!(
            parse_case("screaming_snake").unwrap(),
            Case::UpperSnake
        ));
        assert!(matches!(
            parse_case("upper-kebab").unwrap(),
            Case::UpperKebab
        ));
    }

    #[test]
    fn parses_case_disable_setting() {
        assert!(parse_case_setting("none").unwrap().is_none());
        assert!(parse_case_setting("preserve").unwrap().is_none());
    }

    #[test]
    fn rejects_unknown_case() {
        assert!(matches!(
            parse_case("NotACase"),
            Err(HelperConfigError::UnknownCase(case)) if case == "NotACase"
        ));
    }

    #[test]
    fn parses_sort_names_flexibly() {
        assert!(matches!(
            parse_sort("ValueDesc").unwrap(),
            DefineEnumSort::ValueDesc
        ));
        assert!(matches!(
            parse_sort("value_desc").unwrap(),
            DefineEnumSort::ValueDesc
        ));
    }

    #[test]
    fn rejects_unknown_sort() {
        assert!(matches!(
            parse_sort("NotASort"),
            Err(HelperConfigError::UnknownSort(sort)) if sort == "NotASort"
        ));
    }
}
