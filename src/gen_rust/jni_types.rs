//! JNI type conversion utilities for the Rust glue code.

/// Generate the jni_types module source.
pub fn generate_jni_types() -> String {
    r#"// Auto-generated JNI type conversions. DO NOT EDIT.

use jni::JNIEnv;
use jni::objects::JByteBuffer;
use jni::sys::*;

/// Convert a Rust String to a JNI jstring.
pub fn rust_string_to_jni(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s)
        .expect("Failed to create Java string")
        .into_raw()
}

/// Convert a JNI jstring to a Rust String.
pub fn jni_to_rust_string(env: &mut JNIEnv, s: jstring) -> String {
    env.get_string(&unsafe { jni::objects::JString::from_raw(s) })
        .expect("Failed to read Java string")
        .into()
}

/// Convert a Direct ByteBuffer to a Rust Vec<u8>.
pub fn jni_bytebuffer_to_vec(env: &mut JNIEnv, buf: &JByteBuffer) -> Vec<u8> {
    let addr = env.get_direct_buffer_address(buf)
        .expect("Failed to get direct buffer address");
    let capacity = env.get_direct_buffer_capacity(buf)
        .expect("Failed to get direct buffer capacity");
    unsafe {
        std::slice::from_raw_parts(addr as *const u8, capacity).to_vec()
    }
}

/// Convert a Rust Vec<u8> to a Direct ByteBuffer.
pub fn vec_to_jni_bytebuffer(env: &mut JNIEnv, data: &[u8]) -> jobject {
    let buf = env.new_direct_byte_buffer(data)
        .expect("Failed to create direct byte buffer");
    buf.into_raw()
}
"#.to_string()
}
