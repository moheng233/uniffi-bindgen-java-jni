// Auto-generated JNI bridge functions. DO NOT EDIT.

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::*;

use crate::jni_types::*;
use uniffi::ffi::{RustBuffer, ForeignBytes, RustCallStatus};
use {{ main_crate_name }}::*;

{% for func in functions %}
#[no_mangle]
pub extern "system" fn {{ func.jni_name }}(
    mut env: JNIEnv,
    _class: JClass,
    {%- for arg in func.args %}
    {{ arg.name }}: {{ arg.jni_type }},
    {%- endfor %}
) {% match func.return_jni_type %}{% when Some with (ret) %}-> {{ ret }}{% when None %}{% endmatch %} {
    // Convert JNI args to FFI types
    {%- for arg in func.args %}
    let {{ arg.name }}_ffi: {{ arg.ffi_rust_type }} = {{ arg.conv_expr }};
    {%- endfor %}

    // Call the FFI function
    {%- if func.has_rust_call_status %}
    let mut call_status = RustCallStatus::default();
    {% match func.return_ffi_rust_type %}
    {% when Some with (_) %}
    let result = unsafe { {{ func.ffi_name }}({% for arg in func.args %}{{ arg.name }}_ffi, {% endfor %}&mut call_status) };
    {% when None %}
    unsafe { {{ func.ffi_name }}({% for arg in func.args %}{{ arg.name }}_ffi, {% endfor %}&mut call_status) };
    {% endmatch %}
    {%- else %}
    {% match func.return_ffi_rust_type %}
    {% when Some with (_) %}
    let result = unsafe { {{ func.ffi_name }}({% for arg in func.args %}{{ arg.name }}_ffi{% if !loop.last %}, {% endif %}{% endfor %}) };
    {% when None %}
    unsafe { {{ func.ffi_name }}({% for arg in func.args %}{{ arg.name }}_ffi{% if !loop.last %}, {% endif %}{% endfor %}) };
    {% endmatch %}
    {%- endif %}

    // Convert return value
    {%- match func.return_conv_expr %}
    {%- when Some with (expr) %}
    unsafe { {{ expr }} }
    {%- when None %}
    // void return
    {%- endmatch %}
}
{% endfor %}
