use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::{BindgenError, Builder, DefineEnum, IdentRenamer, Regex, Renamer};

/// High-level bindgen helper that owns a configured bindgen [`Builder`].
///
/// ```
/// use bindgen_helpers::{define_enum, BindingsBuilder, Builder};
///
/// let mut helpers = BindingsBuilder::new(
///     Builder::default().header_contents("test.h", "#define ERR_FOO 1"),
///     false, // hide debugging
/// );
///
/// define_enum!(helpers, ErrorCode, r"^ERR_", remove: "^ERR_");
///
/// let bindings = helpers.into_string().unwrap();
/// assert!(bindings.contains("pub enum ErrorCode"));
/// ```
#[derive(Debug, Clone)]
pub struct BindingsBuilder {
    builder: Builder,
    renamer: Renamer,
}

/// Error returned by [`BindingsBuilder`] finalizers.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BindingsBuilderError {
    /// Bindgen failed to generate bindings.
    #[error(transparent)]
    Bindgen(#[from] BindgenError),
    /// Writing generated output failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Generated output was not valid UTF-8.
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl BindingsBuilder {
    /// Create helpers with optional debug warnings from the internal callback.
    #[must_use]
    pub fn new(builder: Builder, debug: bool) -> Self {
        Self {
            builder,
            renamer: Renamer::new(debug),
        }
    }

    /// Rename a single C item, e.g., a struct, enum, or a typedef.
    pub fn rename_item(
        &mut self,
        c_name: impl AsRef<str>,
        rust_name: impl AsRef<str>,
    ) {
        self.renamer.rename_item(c_name, rust_name);
    }

    /// Rename any C item, including enums and structs.
    ///
    /// # Panics
    /// Will panic if the regex contains '^' or '$' symbols.
    pub fn rename_many(&mut self, c_name: Regex, renamer: IdentRenamer) {
        self.renamer.rename_many(c_name, renamer);
    }

    /// Rename enum values.
    ///
    /// # Panics
    /// Will panic if the `enum_c_name` is not a valid regex.
    pub fn rename_enum_val(
        &mut self,
        enum_c_name: Option<&str>,
        val_renamer: IdentRenamer,
    ) {
        self.renamer.rename_enum_val(enum_c_name, val_renamer);
    }

    /// Collect matching integer `#define` constants into a Rust enum.
    pub fn define_enum(&mut self, define_enum: DefineEnum) {
        self.renamer.define_enum(define_enum);
    }

    /// Generate bindings and write them to the given writer.
    ///
    /// # Errors
    /// Returns an error if bindgen generation fails or generated output cannot
    /// be written.
    pub fn write<W: Write>(
        self,
        mut writer: W,
    ) -> Result<(), BindingsBuilderError> {
        self.builder
            .rustified_enum(self.renamer.get_regex_str())
            .parse_callbacks(Box::new(self.renamer.clone()))
            .generate()?
            .write(Box::new(&mut writer))?;
        writer.write_all(self.renamer.render_define_enums().as_bytes())?;
        Ok(())
    }

    /// Generate bindings and write them to a file.
    ///
    /// # Errors
    /// Returns an error if bindgen generation fails, the output file cannot be
    /// opened, or generated output cannot be written.
    pub fn write_to_file<P: AsRef<Path>>(
        self,
        path: P,
    ) -> Result<(), BindingsBuilderError> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.as_ref())?;
        self.write(file)
    }

    /// Generate bindings and return them as a string.
    ///
    /// # Errors
    /// Returns an error if bindgen generation fails, generated output cannot be
    /// written to an internal buffer, or the output is not valid UTF-8.
    pub fn into_string(self) -> Result<String, BindingsBuilderError> {
        let mut output = Vec::new();
        self.write(&mut output)?;
        Ok(String::from_utf8(output)?)
    }
}
