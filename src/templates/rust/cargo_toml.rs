[package]
name = "uniffi-jni-glue"
version = "0.1.0"
edition = "2021"

[lib]
name = "{{ crate_name }}"
crate-type = ["cdylib"]

[dependencies]
{% for dep in dependencies %}
{{ dep.name }} = {{ dep.spec }}
{% endfor %}
{%- if let Some(path) = main_crate_path %}
# Main crate dependency (path provided via --main-crate-path)
{{ main_crate_name }} = { path = "{{ path }}" }
{%- else %}
# Add your main crate dependency here:
# your_crate = { path = "../your_crate" }
{%- endif %}
