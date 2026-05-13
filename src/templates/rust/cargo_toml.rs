[package]
name = "{{ crate_name }}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
jni = "0.21"
uniffi = { version = "0.31", features = ["builtin-bindgen"] }
# Add your main crate dependency here:
# your_crate = { path = "../your_crate" }

