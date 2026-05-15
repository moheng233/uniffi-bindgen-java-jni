{# Included as part of wrapper.java for each CallbackInterface type definition. #}
{# A Callback Interface is a Rust trait that Java code implements. #}
{%- if cbi is defined -%}
    /**
     * {{ cbi.name }} — a callback interface.
     *
     * Implement this interface in Java to pass a callback to Rust.
     *
     {%- if cbi.docstring.is_some() %}
     * {{ cbi.docstring.as_ref().unwrap() }}
     {%- endif %}
     */
    public interface {{ cbi.java_name }} {

        {%- for method in cbi.methods %}
        {% match method.return_type %}{% when Some with (ret) %}{{ ret|java_type }}{% when None %}void{% endmatch %} {{ method.java_name }}(
            {%- for arg in method.arguments -%}
            {{ arg.ty|java_type }} {{ arg.java_name }}{% if !loop.last %}, {% endif %}
            {%- endfor -%}
        );
        {%- endfor %}

        /**
         * Register a Java implementation of this callback interface with Rust.
         *
         * @return a handle that can be used to pass this implementation to Rust functions
         */
        default long register{{ cbi.java_name }}() {
            return {{ name }}.register{{ cbi.java_name }}(this);
        }

        /**
         * Unregister this callback interface.
         */
        default void unregister{{ cbi.java_name }}() {
            // TODO: implement unregister
        }
    }

    // Converter for callback interface
    public static final class FfiConverter{{ cbi.java_name }} implements
            FfiConverter<{{ cbi.java_name }}, Long> {

        public static final FfiConverter{{ cbi.java_name }} INSTANCE =
            new FfiConverter{{ cbi.java_name }}();

        @Override
        public {{ cbi.java_name }} lift(Long handle) {
            return handleMap{{ cbi.java_name }}.get(handle);
        }

        @Override
        public Long lower({{ cbi.java_name }} value) {
            return handleMap{{ cbi.java_name }}.insert(value);
        }

        @Override
        public {{ cbi.java_name }} read(ByteBuffer buf) {
            return lift(buf.getLong());
        }

        @Override
        public void write({{ cbi.java_name }} value, ByteBuffer buf) {
            buf.putLong(lower(value));
        }

        @Override
        public int allocationSize({{ cbi.java_name }} value) {
            return 8;
        }
    }
{%- endif -%}

