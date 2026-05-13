//! JNI type conversion utilities for the Rust glue code.

use askama::Template;

/// Template for rendering jni_types module.
#[derive(Template)]
#[template(escape = "none", path = "rust/jni_types.rs")]
struct JniTypesTemplate;

/// Generate the jni_types module source.
pub fn generate_jni_types() -> String {
    let tmpl = JniTypesTemplate;
    tmpl.render().expect("Failed to render jni_types template")
}

