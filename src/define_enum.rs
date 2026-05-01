use std::sync::{Arc, Mutex};

use crate::{IdentRenamer, Regex};

/// Collect matching integer `#define` constants and render a Rust enum for them.
///
/// Bindgen still emits the matched constants as usual. The rendered enum uses those
/// constants as discriminants, e.g. `Foo = (ERR_FOO as u32)`.
#[derive(Debug, Clone)]
pub struct DefineEnum {
    enum_name: String,
    repr: Option<String>,
    matcher: Regex,
    exclude: Vec<Regex>,
    sort: Option<DefineEnumSort>,
    derives: Option<Vec<String>>,
    min: Option<i64>,
    max: Option<i64>,
    variant_renamer: IdentRenamer,
    values: Arc<Mutex<Vec<DefineEnumValue>>>,
}

/// Sort order for define-backed enum variants.
#[derive(Debug, Clone, Copy)]
pub enum DefineEnumSort {
    /// Sort by the rendered Rust variant name.
    Name,
    /// Sort by the integer macro value.
    Value,
    /// Sort by the integer macro value in reverse order.
    ValueDesc,
}

#[derive(Debug, Clone)]
struct DefineEnumValue {
    const_name: String,
    variant_name: String,
    value: i64,
}

impl DefineEnum {
    /// Create a new define-backed Rust enum collector.
    #[must_use]
    pub fn new(
        enum_name: impl Into<String>,
        matcher: Regex,
        variant_renamer: IdentRenamer,
    ) -> Self {
        Self {
            enum_name: enum_name.into(),
            repr: None,
            matcher,
            exclude: Vec::new(),
            sort: None,
            derives: None,
            min: None,
            max: None,
            variant_renamer,
            values: Arc::default(),
        }
    }

    /// Set an explicit `repr` for the generated enum.
    #[must_use]
    pub fn with_repr(mut self, repr: impl Into<String>) -> Self {
        self.repr = Some(repr.into());
        self
    }

    /// Exclude matching macro names from this enum.
    #[must_use]
    pub fn exclude(mut self, matcher: Regex) -> Self {
        self.exclude.push(matcher);
        self
    }

    /// Sort generated enum variants.
    #[must_use]
    pub fn sort(mut self, sort: DefineEnumSort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Set derives for the generated enum.
    #[must_use]
    pub fn derives(mut self, derives: Vec<String>) -> Self {
        self.derives = Some(derives);
        self
    }

    /// Include only values greater than or equal to `min`.
    #[must_use]
    pub fn min(mut self, min: i64) -> Self {
        self.min = Some(min);
        self
    }

    /// Include only values less than or equal to `max`.
    #[must_use]
    pub fn max(mut self, max: i64) -> Self {
        self.max = Some(max);
        self
    }

    /// Return true if this enum should include the given macro name.
    #[must_use]
    fn is_match(&self, name: &str) -> bool {
        self.matcher.is_match(name)
            && !self.exclude.iter().any(|re| re.is_match(name))
    }

    fn record_macro(&self, name: &str, value: i64) {
        let value = DefineEnumValue {
            const_name: name.to_owned(),
            variant_name: self.variant_renamer.apply(name),
            value,
        };

        match self.values.lock() {
            Ok(mut values) => values.push(value),
            Err(poisoned) => {
                poisoned.into_inner().push(value);
            }
        }
    }

    pub(crate) fn record_matching_macro(&self, name: &str, value: i64) {
        if self.is_match(name)
            && self.min.map_or(true, |v| value >= v)
            && self.max.map_or(true, |v| value <= v)
        {
            self.record_macro(name, value);
        }
    }

    /// Render the collected constants as a Rust enum.
    ///
    /// Returns an empty string if no matching constants were collected.
    #[must_use]
    pub fn render(&self) -> String {
        let mut values = match self.values.lock() {
            Ok(values) => values.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        };
        if values.is_empty() {
            return String::new();
        }

        match self.sort {
            Some(DefineEnumSort::Name) => {
                values.sort_by(|a, b| a.variant_name.cmp(&b.variant_name));
            }
            Some(DefineEnumSort::Value) => {
                values.sort_by_key(|v| v.value);
            }
            Some(DefineEnumSort::ValueDesc) => {
                values.sort_by(|a, b| b.value.cmp(&a.value));
            }
            None => {}
        }

        let repr = self
            .repr
            .as_deref()
            .unwrap_or_else(|| repr_for_values(values.iter().map(|v| v.value)));
        let mut derives = self.derives.as_ref().map_or_else(
            || ["Debug", "Clone", "Copy", "Hash", "PartialEq", "Eq"].join(", "),
            |v| v.join(", "),
        );
        if !derives.is_empty() {
            derives = format!("#[derive({derives})]\n");
        }
        let mut output = format!(
            "\n#[repr({repr})]\n{derives}pub enum {} {{\n",
            self.enum_name
        );
        for value in &values {
            output.push_str("    ");
            output.push_str(&value.variant_name);
            output.push_str(" = (");
            output.push_str(&value.const_name);
            output.push_str(" as ");
            output.push_str(repr);
            output.push_str("),\n");
        }
        output.push_str("}\n");
        output
    }
}

fn repr_for_values(values: impl Iterator<Item = i64>) -> &'static str {
    let (min, max) = values.fold((i64::MAX, i64::MIN), |(min, max), value| {
        (min.min(value), max.max(value))
    });

    if min < 0 {
        if min < i64::from(i32::MIN) || max > i64::from(i32::MAX) {
            "i64"
        } else {
            "i32"
        }
    } else if max > i64::from(u32::MAX) {
        "u64"
    } else {
        "u32"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Case;

    #[test]
    fn test_render_is_repeatable() {
        let define_enum = DefineEnum::new(
            "ErrorCode",
            Regex::new("^ERR_").unwrap(),
            IdentRenamer::default_case(Case::Pascal),
        );
        define_enum.record_matching_macro("ERR_FOO", 1);

        let rendered = define_enum.render();

        assert_eq!(define_enum.render(), rendered);
        assert!(rendered.contains("ErrFoo = (ERR_FOO as u32),"));
    }
}
