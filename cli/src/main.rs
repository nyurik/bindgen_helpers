use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use bindgen::builder_from_flags;
use bindgen_helpers::BindingsBuilder;

const HELPER_CONFIG_FLAG: &str = "--helper-config";

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let ParsedArgs {
        bindgen_args,
        helper_config,
    } = extract_helper_config(env::args_os())?;

    let (builder, output, _verbose) = builder_from_flags(
        bindgen_args
            .into_iter()
            .map(|v| v.into_string().map_err(CliError::NonUtf8Argument))
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_iter)?,
    )?;

    let helpers = if let Some(helper_config) = helper_config {
        BindingsBuilder::with_config_file(builder, helper_config)?
    } else {
        eprintln!("warning: no {HELPER_CONFIG_FLAG} provided; running as plain bindgen");
        BindingsBuilder::new(builder, false)
    };
    helpers.write(output)?;

    Ok(())
}

#[derive(Debug)]
struct ParsedArgs {
    bindgen_args: Vec<OsString>,
    helper_config: Option<PathBuf>,
}

fn extract_helper_config(
    args: impl IntoIterator<Item = OsString>,
) -> Result<ParsedArgs, CliError> {
    let mut bindgen_args = Vec::new();
    let mut helper_config = None;
    let mut args = args.into_iter();

    if let Some(program) = args.next() {
        bindgen_args.push(program);
    }

    while let Some(arg) = args.next() {
        if let Some(remainder) = arg
            .to_str()
            .and_then(|v| v.strip_prefix(HELPER_CONFIG_FLAG))
        {
            if remainder.is_empty() {
                let value =
                    args.next().ok_or(CliError::MissingHelperConfigValue)?;
                set_helper_config(&mut helper_config, value)?;
                continue;
            }
            if let Some(value) = remainder.strip_prefix('=') {
                set_helper_config(&mut helper_config, OsString::from(value))?;
                continue;
            }
        }

        bindgen_args.push(arg);
    }

    Ok(ParsedArgs {
        bindgen_args,
        helper_config,
    })
}

fn set_helper_config(
    helper_config: &mut Option<PathBuf>,
    value: OsString,
) -> Result<(), CliError> {
    if helper_config.is_some() {
        return Err(CliError::DuplicateHelperConfig);
    }
    *helper_config = Some(PathBuf::from(value));
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("`{HELPER_CONFIG_FLAG}` requires a path argument")]
    MissingHelperConfigValue,
    #[error("`{HELPER_CONFIG_FLAG}` may only be provided once")]
    DuplicateHelperConfig,
    #[error("bindgen arguments must be valid UTF-8: {0:?}")]
    NonUtf8Argument(OsString),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Bindgen(#[from] bindgen::BindgenError),
    #[error(transparent)]
    HelperConfig(#[from] bindgen_helpers::HelperConfigError),
    #[error(transparent)]
    Helpers(#[from] bindgen_helpers::BindingsBuilderError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<ParsedArgs, CliError> {
        extract_helper_config(args.iter().map(OsString::from))
    }

    #[test]
    fn strips_space_separated_helper_config() {
        let parsed = parse(&[
            "bindgen-helper",
            "wrapper.h",
            "--helper-config",
            "helper.toml",
            "-o",
            "bindings.rs",
        ])
        .expect("helper config should parse");

        assert_eq!(parsed.helper_config, Some(PathBuf::from("helper.toml")));
        assert_eq!(
            parsed.bindgen_args,
            vec![
                OsString::from("bindgen-helper"),
                OsString::from("wrapper.h"),
                OsString::from("-o"),
                OsString::from("bindings.rs"),
            ]
        );
    }

    #[test]
    fn strips_equals_helper_config() {
        let parsed = parse(&[
            "bindgen-helper",
            "wrapper.h",
            "--helper-config=helper.toml",
        ])
        .expect("helper config should parse");

        assert_eq!(parsed.helper_config, Some(PathBuf::from("helper.toml")));
        assert_eq!(
            parsed.bindgen_args,
            vec![
                OsString::from("bindgen-helper"),
                OsString::from("wrapper.h"),
            ]
        );
    }

    #[test]
    fn keeps_similarly_prefixed_bindgen_args() {
        let parsed = parse(&[
            "bindgen-helper",
            "wrapper.h",
            "--helper-configured",
            "value",
        ])
        .expect("helper config should parse");

        assert_eq!(parsed.helper_config, None);
        assert_eq!(
            parsed.bindgen_args,
            vec![
                OsString::from("bindgen-helper"),
                OsString::from("wrapper.h"),
                OsString::from("--helper-configured"),
                OsString::from("value"),
            ]
        );
    }

    #[test]
    fn rejects_missing_helper_config_value() {
        assert!(matches!(
            parse(&["bindgen-helper", "wrapper.h", "--helper-config"]),
            Err(CliError::MissingHelperConfigValue)
        ));
    }

    #[test]
    fn rejects_duplicate_helper_config() {
        assert!(matches!(
            parse(&[
                "bindgen-helper",
                "--helper-config",
                "first.toml",
                "--helper-config=second.toml",
            ]),
            Err(CliError::DuplicateHelperConfig)
        ));
    }
}
