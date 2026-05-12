//! JNI callback generation for the Rust glue code.

use anyhow::Result;
use crate::pipeline::nodes::Root;

/// Generate the jni_callback module source.
pub fn generate_jni_callback(_root: &Root, _crate_filter: Option<&str>) -> Result<String> {
    Ok(r#"// Auto-generated JNI callback support. DO NOT EDIT.

use jni::JNIEnv;
use jni::sys::*;

/// Placeholder for callback support.
/// TODO: Implement callback interface support (Phase 6).
"#.to_string())
}
