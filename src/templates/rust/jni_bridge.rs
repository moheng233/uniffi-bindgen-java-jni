// Auto-generated JNI bridge functions. DO NOT EDIT.

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::*;

use crate::jni_types::*;

{% for func in functions %}
#[no_mangle]
pub extern "system" fn {{ func.jni_name }}(
    mut env: JNIEnv,
    _class: JClass,
    {%- for arg in func.arguments %}
    {{ arg.name }}: {{ arg.rust_type }},
    {%- endfor %}
    {%- if func.has_rust_call_status %}
    status: jobject,
    {%- endif %}
) {% match func.return_type %}{% when Some with (ret) %}-> {{ ret }}{% when None %}{% endmatch %} {
    {%- if func.has_rust_call_status %}
    uniffi::ffi::rust_call(|_status| {
        {%- for arg in func.arguments %}
        {%- if arg.is_buffer %}
        let {{ arg.name }}_rb = unsafe { jni_bytebuffer_to_rustbuffer(&mut env, {{ arg.name }}) };
        {%- endif %}
        {%- endfor %}
        let result = unsafe { {{ func.ffi_name }}(
            {%- for arg in func.arguments %}
            {%- if arg.is_buffer %}{{ arg.name }}_rb{%- else %}{{ arg.name }}{%- endif %},
            {%- endfor %}
            _status,
        ) };
        {%- if func.return_is_buffer %}
        unsafe { rustbuffer_to_jni_bytebuffer(&mut env, result) }
        {%- else %}
        result
        {%- endif %}
    })
    {%- else %}
    {# No RustCallStatus - simple call #}
    {%- for arg in func.arguments %}
    {%- if arg.is_buffer %}
    let {{ arg.name }}_rb = unsafe { jni_bytebuffer_to_rustbuffer(&mut env, {{ arg.name }}) };
    {%- endif %}
    {%- endfor %}
    {%- if func.return_is_buffer %}
    let result = unsafe { {{ func.ffi_name }}(
        {%- for arg in func.arguments %}
        {%- if arg.is_buffer %}{{ arg.name }}_rb{%- else %}{{ arg.name }}{%- endif %},
        {%- endfor %}
    ) };
    unsafe { rustbuffer_to_jni_bytebuffer(&mut env, result) }
    {%- else %}
    unsafe { {{ func.ffi_name }}(
        {%- for arg in func.arguments %}
        {%- if arg.is_buffer %}{{ arg.name }}_rb{%- else %}{{ arg.name }}{%- endif %},
        {%- endfor %}
    ) }
    {%- endif %}
    {%- endif %}
}

{% endfor %}

