//! JNI function signature generation utilities.
//!
//! Helpers for constructing JNI-compliant function names and signatures
//! following the Java_package_Class_method convention.

/// Build a JNI function name from package, class, and method.
///
/// Example: `Java_com_example_MyLib_myFunction`
#[allow(dead_code)]
pub fn jni_func_name(package: &str, class: &str, method: &str) -> String {
    let pkg_part = package.replace('.', "_");
    format!("Java_{}_{}_{}", pkg_part, class, method)
}

/// Build a JNI constructor name from package and class.
///
/// Example: `Java_com_example_MyClass_init`
#[allow(dead_code)]
pub fn jni_ctor_name(package: &str, class: &str) -> String {
    let pkg_part = package.replace('.', "_");
    format!("Java_{}_{}_init", pkg_part, class)
}
