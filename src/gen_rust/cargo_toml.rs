//! Cargo.toml generation for the Rust JNI glue crate.

use askama::Template;
use crate::pipeline::nodes::Root;

/// Template for rendering the glue crate's Cargo.toml.
#[derive(Template)]
#[template(escape = "none", path = "rust/cargo_toml.rs")]
struct CargoTomlTemplate {
    crate_name: String,
}

/// Generate the Cargo.toml content for the glue crate.
pub fn generate_cargo_toml(crate_name: &str, _root: &Root) -> String {
    let tmpl = CargoTomlTemplate {
        crate_name: crate_name.to_string(),
    };
    tmpl.render().expect("Failed to render Cargo.toml template")
}

