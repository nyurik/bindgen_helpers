#![cfg(test)]

use bindgen_helpers::{BindingsBuilder, Builder, Renamer};

mod c_enum;
mod define_enum;
mod helpers;
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

fn test_with_define_enums(cb: &Renamer, header: &str) -> String {
    let mut output = Vec::new();
    Builder::default()
        .header_contents("test.h", header)
        .disable_header_comment() // version keeps changing
        .rustified_enum(cb.get_regex_str())
        .parse_callbacks(Box::new(cb.clone()))
        .generate()
        .expect("Failed to generate bindings")
        .write(Box::new(&mut output))
        .expect("Failed to write bindings");

    let mut output =
        String::from_utf8(output).expect("Output was not valid UTF-8");
    output.push_str(&cb.render_define_enums());
    output
}

fn builder(header: &str) -> BindingsBuilder {
    BindingsBuilder::new(
        Builder::default()
            .header_contents("test.h", header)
            .disable_header_comment(), // version keeps changing
        true,
    )
}
