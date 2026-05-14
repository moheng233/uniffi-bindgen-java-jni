{# Included as part of wrapper.java for each Interface type definition (ObjectImpl::Trait or CallbackTrait). #}
{# An Interface is an object whose implementation lives in Rust. #}
{# This template generates the Java interface that matches the Rust trait. #}
{%- if obj is defined -%}
    /**
     * {{ obj.name }} — a trait interface implemented by Rust.
     *
     {%- if obj.docstring.is_some() %}
     * {{ obj.docstring.as_ref().unwrap() }}
     {%- endif %}
     */
    public interface {{ obj.java_name }} extends AutoCloseable {

        {%- for method in obj.methods %}
        {% match method.return_type %}{% when Some with (ret) %}{{ ret|java_type }}{% when None %}void{% endmatch %} {{ method.java_name }}(
            {%- for arg in method.arguments -%}
            {{ arg.ty|java_type }} {{ arg.java_name }}{% if !loop.last %}, {% endif %}
            {%- endfor -%}
        );
        {%- endfor %}

        @Override
        void close();
    }

    /**
     * Rust-side implementation of {@link {{ obj.java_name }}}.
     */
    static final class {{ obj.java_name }}Impl implements {{ obj.java_name }} {

        private long handle;

        {{ obj.java_name }}Impl(long handle) {
            this.handle = handle;
        }

        // --- Interface methods ---
        {%- for method in obj.methods %}
        @Override
        public {% match method.return_type %}{% when Some with (ret) %}{{ ret|java_type }}{% when None %}void{% endmatch %} {{ method.java_name }}(
            {%- for arg in method.arguments -%}
            {{ arg.ty|java_type }} {{ arg.java_name }}{% if !loop.last %}, {% endif %}
            {%- endfor -%}
        ) {
            {{ method.body }}
        }
        {%- endfor %}

        @Override
        public void close() {
            if (handle != 0) {
                {{ name }}.{{ obj.ffi_func_free.name() }}(handle);
                handle = 0;
            }
        }

        @Override
        @SuppressWarnings("deprecation")
        protected void finalize() throws Throwable {
            try { close(); } finally { super.finalize(); }
        }
    }
{%- endif -%}

