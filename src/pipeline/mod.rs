//! Java JNI Intermediate Representation.
//!
//! This module provides:
//! 1. The pipeline function to get the general IR
//! 2. Manual conversion from general IR to Java-specific nodes

pub mod config;
pub mod context;
pub mod filters;
pub mod modules;
pub mod nodes;
pub mod body_gen;
pub mod types;
pub mod jni_signature;

use uniffi_bindgen::pipeline::general;
use uniffi_pipeline::Pipeline;

/// Build and execute the general pipeline with "java" as the bindings TOML key.
pub fn general_pipeline() -> Pipeline<
    uniffi_bindgen::pipeline::initial::Root,
    general::Root,
> {
    uniffi_bindgen::pipeline::general::pipeline("java")
}
