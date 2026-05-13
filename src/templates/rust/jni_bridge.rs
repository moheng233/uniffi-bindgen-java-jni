// Auto-generated JNI bridge functions. DO NOT EDIT.

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::*;


#[allow(unused_variables)]
{% for func in functions %}
#[no_mangle]
pub extern "system" fn {{ func.jni_name }}(
    _env: JNIEnv,
    _class: JClass,
    {%- for arg in func.arguments %}
    {{ arg.name }}: {{ arg.rust_type }},
    {%- endfor %}
    {%- if func.has_rust_call_status %}
    _status: jobject,
    {%- endif %}
) {% match func.return_type %}{% when Some with (ret) %}-> {{ ret }}{% when None %}{% endmatch %} {
    unimplemented!("JNI bridge: {{ func.jni_name }}")
}
{% endfor %}
