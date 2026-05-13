{# Included as part of wrapper.java for each Enum type definition #}
{%- if e is defined -%}
    /**
     * {{ e.name }} — a Rust-style enum.
     *
     {%- if e.docstring.is_some() %}
     * {{ e.docstring.as_ref().unwrap() }}
     {%- endif %}
     */
    public static sealed class {{ e.java_name }}
        {%- for variant in e.variants %}
        permits {{ e.java_name }}.{{ variant.java_name }}{% if !loop.last %}, {% endif %}
        {%- endfor %} {

        {%- for variant in e.variants %}
        /**
         * Variant {@code {{ variant.name }}}.
         */
        public static final class {{ variant.java_name }} extends {{ e.java_name }} {
            {%- for field in variant.fields %}
            private final {{ field.ty|java_type }} {{ field.java_name }};
            {%- endfor %}

            public {{ variant.java_name }}(
                {%- for field in variant.fields -%}
                {{ field.ty|java_type }} {{ field.java_name }}{% if !loop.last %}, {% endif %}
                {%- endfor -%}
            ) {
                {%- for field in variant.fields %}
                this.{{ field.java_name }} = {{ field.java_name }};
                {%- endfor %}
            }

            {%- for field in variant.fields %}
            public {{ field.ty|java_type }} get{{ field.java_name|capitalize }}() {
                return {{ field.java_name }};
            }
            {%- endfor %}

            @Override
            public String toString() {
                return "{{ variant.name }}(" +
                    {%- for field in variant.fields -%}
                    {{ field.java_name }}{% if !loop.last %} + ", " + {% endif %}
                    {%- endfor -%}
                    ")";
            }
        }
        {%- endfor %}

        // --- Serialization ---
        public ByteBuffer write() {
            {{ e.write_body }}
        }

        public static {{ e.java_name }} read(ByteBuffer buf) {
            {{ e.read_body }}
        }
    }
{%- endif -%}

