#![cfg(test)]

use bindgen_helpers::{Builder, Renamer};

mod c_enum;
mod renames;

fn test(cb: Renamer, header: &str) -> String {
    let mut output = Vec::new();
    Builder::default()
        .header_contents("test.h", header)
        .disable_header_comment() // version keeps changing
        .rustified_enum(cb.get_regex_str())
        .parse_callbacks(Box::new(cb))
        .generate()
        .expect("Failed to generate bindings")
        .write(Box::new(&mut output))
        .expect("Failed to write bindings");

    String::from_utf8(output).expect("Output was not valid UTF-8")
}
