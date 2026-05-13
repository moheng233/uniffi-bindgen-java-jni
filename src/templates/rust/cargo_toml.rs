[package]
name = "{{ crate_name }}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
jni = "0.21"
uniffi = { version = "0.31", features = ["builtin-bindgen"] }
{%- if let Some(path) = main_crate_path %}
# Main crate dependency (path provided via --main-crate-path)
main_crate = { path = "{{ path }}" }
{%- else %}
# Add your main crate dependency here:
# main_crate = { path = "../your_crate" }
{%- endif %}

