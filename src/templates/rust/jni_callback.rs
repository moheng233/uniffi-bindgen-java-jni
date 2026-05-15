// Auto-generated JNI callback support. DO NOT EDIT.

#![allow(unused_imports)]
#![allow(unused_unsafe)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use jni::objects::{GlobalRef, JValue};
use jni::sys::*;
use jni::JavaVM;

use crate::jni_types::*;
use uniffi::ffi::RustBuffer;

{% if has_callbacks %}
/// Global map of callback handles to Java GlobalRef objects.
static CALLBACK_HANDLES: once_cell::sync::Lazy<Mutex<HashMap<u64, GlobalRef>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// Global atomic counter for callback handles.
static NEXT_HANDLE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Stored JavaVM raw pointer for JNI calls from callback context.
static mut JVM_PTR: *mut jni::sys::JavaVM = std::ptr::null_mut();

/// Store the JavaVM reference. Called from JNI_OnLoad.
pub fn store_jvm(vm: &JavaVM) {
    unsafe { JVM_PTR = vm.get_java_vm_pointer(); }
}

/// Get the JavaVM from the stored raw pointer.
unsafe fn get_jvm() -> JavaVM {
    let ptr = unsafe { JVM_PTR };
    assert!(!ptr.is_null(), "JavaVM not initialized");
    JavaVM::from_raw(ptr).expect("Invalid JavaVM pointer")
}

pub fn insert_callback(callback: GlobalRef) -> u64 {
    let handle = NEXT_HANDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    CALLBACK_HANDLES.lock().unwrap().insert(handle, callback);
    handle
}

fn remove_callback(handle: u64) {
    CALLBACK_HANDLES.lock().unwrap().remove(&handle);
}

fn clone_callback(handle: u64) -> u64 {
    let map = CALLBACK_HANDLES.lock().unwrap();
    let _ = map.get(&handle).expect("Callback handle not found");
    handle
}
{% endif %}

{% for cbi in callback_interfaces %}
// ============================================================
// Callback interface: {{ cbi.java_name }}
// ============================================================

{# Declare the init callback function as extern "C" with raw pointer #}
extern "C" {
    fn {{ cbi.init_fn }}(vtable: *const std::ffi::c_void);
}

{# Define a local repr(C) VTable with actual function pointer types for Sync safety #}
#[repr(C)]
struct VTable_{{ cbi.name|upper }} {
    uniffi_free: Option<unsafe extern "C" fn(u64)>,
    uniffi_clone: Option<unsafe extern "C" fn(u64) -> u64>,
{% for m in cbi.methods %}
    {{ m.vtable_field_name }}: *const std::ffi::c_void,
{% endfor %}
}

unsafe impl Sync for VTable_{{ cbi.name|upper }} {}

unsafe extern "C" fn callback_free_{{ cbi.name }}(handle: u64) {
    remove_callback(handle);
}

unsafe extern "C" fn callback_clone_{{ cbi.name }}(handle: u64) -> u64 {
    clone_callback(handle)
}

{% for m in cbi.methods %}
unsafe extern "C" fn callback_{{ cbi.name }}_{{ m.name }}(
    uniffi_handle: u64,
    {% for arg in m.ffi_args %}{{ arg.name }}: {{ arg.ffi_type }},
    {% endfor %}
    {% if m.has_return %}uniffi_out_return: *mut {{ m.return_ffi_type }},{% else %}_uniffi_out_return: *mut std::ffi::c_void,{% endif %}
    call_status: &mut uniffi::ffi::RustCallStatus,
) {
    let jvm = unsafe { get_jvm() };
    let mut env = jvm.attach_current_thread()
        .expect("Failed to attach JNI thread for callback");

    // Look up the Java callback object by handle
    let obj = {
        CALLBACK_HANDLES.lock().unwrap()
            .get(&uniffi_handle)
            .cloned()
    };
    let obj = match obj {
        Some(gref) => unsafe { jni::objects::JObject::from_raw(gref.as_raw()) },
        None => {
            *call_status = uniffi::ffi::RustCallStatus::error("callback handle not found");
            return;
        }
    };
    let class = env.get_object_class(&obj)
        .expect("Failed to get callback object class");

    // Build JNI args
    {% for decl in m.jni_object_decls %}{{ decl }}
    {% endfor %}
    let jni_args: &[JValue] = &[
        JValue::Long(uniffi_handle as jlong),
        {% for arg in m.jni_arg_values %}{{ arg }},
        {% endfor %}
    ];

    {% if m.has_return %}
    let result = env.call_static_method(
        &class,
        "{{ m.java_callback_name }}",
        "{{ m.jni_signature }}",
        jni_args,
    ).expect("Failed to call Java callback method");
    unsafe { *uniffi_out_return = {{ m.jni_extract_ret }}; }
    {% else %}
    let _ = env.call_static_method(
        &class,
        "{{ m.java_callback_name }}",
        "{{ m.jni_signature }}",
        jni_args,
    ).expect("Failed to call Java callback method");
    {% endif %}

    *call_status = uniffi::ffi::RustCallStatus::default();
}
{% endfor %}
// VTable struct instance for {{ cbi.java_name }}
static VTABLE_{{ cbi.name|upper }}: VTable_{{ cbi.name|upper }} = VTable_{{ cbi.name|upper }} {
    uniffi_free: Some(callback_free_{{ cbi.name }}),
    uniffi_clone: Some(callback_clone_{{ cbi.name }}),
{% for m in cbi.methods %}
    {{ m.vtable_field_name }}: callback_{{ cbi.name }}_{{ m.name }} as *const std::ffi::c_void,
{% endfor %}
};

{% endfor %}

{% if has_callbacks %}
/// Register all callback VTables with UniFFI.
pub fn register_callbacks() {
{% for cbi in callback_interfaces %}
    unsafe {
        {{ cbi.init_fn }}(&raw const VTABLE_{{ cbi.name|upper }} as *const std::ffi::c_void);
    }
{% endfor %}
}
{% endif %}


