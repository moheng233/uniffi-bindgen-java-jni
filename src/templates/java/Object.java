{# Included as part of wrapper.java for each Object type definition #}
{%- if obj is defined -%}
    /**
     * {{ obj.name }} — a Rust object exposed to Java.
     *
     {%- if obj.docstring.is_some() %}
     * {{ obj.docstring.as_ref().unwrap() }}
     {%- endif %}
     */
    public static final class {{ obj.java_name }} implements AutoCloseable {

        private long handle;

        private {{ obj.java_name }}(long handle) {
            this.handle = handle;
        }

        /**
         * Returns {@code true} if this object has not been closed.
         */
        public boolean isValid() {
            return handle != 0;
        }

        /**
         * Returns the raw FFI handle. Internal use only.
         */
        long getHandle() {
            return handle;
        }

        // --- Constructors ---
        {%- for ctor in obj.constructors %}
        public static {{ obj.java_name }} {{ ctor.java_name }}(
            {%- for arg in ctor.arguments -%}
            {{ arg.ty|java_type }} {{ arg.java_name }}{% if !loop.last %}, {% endif %}
            {%- endfor -%}
        ) {
            {{ ctor.body }}
        }
        {%- endfor %}

        // --- Methods ---
        {%- for method in obj.methods %}
        public {% match method.return_type %}{% when Some with (ret) %}{{ ret|java_type }}{% when None %}void{% endmatch %} {{ method.java_name }}(
            {%- for arg in method.arguments -%}
            {{ arg.ty|java_type }} {{ arg.java_name }}{% if !loop.last %}, {% endif %}
            {%- endfor -%}
        ) {
            {{ method.body }}
        }
        {%- endfor %}

        // --- Destructor ---
        @Override
        public void close() {
            if (handle != 0) {
                {{ name }}.{{ obj.ffi_func_free.name() }}(handle);
                handle = 0;
            }
        }

        /**
         * Creates a copy of this object handle. Useful when passing to Rust
         * functions that take ownership.
         */
        public {{ obj.java_name }} clone() {
            long newHandle = {{ name }}.{{ obj.ffi_func_clone.name() }}(handle);
            return new {{ obj.java_name }}(newHandle);
        }

        @Override
        @SuppressWarnings("deprecation")
        protected void finalize() throws Throwable {
            try {
                close();
            } finally {
                super.finalize();
            }
        }
    }
{%- endif -%}

