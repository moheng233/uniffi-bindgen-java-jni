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
) {% match func.return_type %}{% when Some with (ret) %}-> {{ ret }}{% when None %}{% endmatch %} {
    // TODO: {{ func.comment }}
    {% if func.has_rust_call_status %}
    uniffi::ffi::rust_call(|status| {
        // call uniffi FFI function and handle status
        todo!("implement bridge for {}", "{{ func.name }}");
    })
    {% else %}
    todo!("implement bridge for {}", "{{ func.name }}");
    {% endif %}
}

{% endfor %}

