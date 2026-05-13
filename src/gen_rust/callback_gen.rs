//! JNI callback generation for the Rust glue code.

use anyhow::Result;
use askama::Template;
use crate::pipeline::nodes::Root;

/// Template for rendering jni_callback module.
#[derive(Template)]
#[template(escape = "none", path = "rust/jni_callback.rs")]
struct JniCallbackTemplate;

/// Generate the jni_callback module source.
pub fn generate_jni_callback(_root: &Root, _crate_filter: Option<&str>) -> Result<String> {
    let tmpl = JniCallbackTemplate;
    Ok(tmpl.render()?)
}

