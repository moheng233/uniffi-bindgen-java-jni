{# Included as part of wrapper.java for each Record type definition #}
{%- if rec is defined -%}
    /**
     * {{ rec.name }} — a data record.
     *
     {%- if rec.docstring.is_some() %}
     * {{ rec.docstring.as_ref().unwrap() }}
     {%- endif %}
     */
    public static final class {{ rec.java_name }} {

        {%- for field in rec.fields %}
        private final {{ field.ty|java_type }} {{ field.java_name }};
        {%- endfor %}

        public {{ rec.java_name }}(
            {%- for field in rec.fields -%}
            {{ field.ty|java_type }} {{ field.java_name }}{% if !loop.last %}, {% endif %}
            {%- endfor -%}
        ) {
            {%- for field in rec.fields %}
            this.{{ field.java_name }} = {{ field.java_name }};
            {%- endfor %}
        }

        // --- Getters ---
        {%- for field in rec.fields %}
        public {{ field.ty|java_type }} get{{ field.java_name|capitalize }}() {
            return {{ field.java_name }};
        }
        {%- endfor %}

        // --- Serialization ---
        /**
         * Serialize this record into a {@link ByteBuffer}.
         */
        public ByteBuffer write() {
            {{ rec.write_body }}
        }

        /**
         * Deserialize a record from a {@link ByteBuffer}.
         */
        public static {{ rec.java_name }} read(ByteBuffer buf) {
            {{ rec.read_body }}
        }

        // --- Standard methods ---
        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (!(o instanceof {{ rec.java_name }} that)) return false;
            {%- for field in rec.fields %}
            if (!java.util.Objects.equals(this.{{ field.java_name }}, that.{{ field.java_name }})) return false;
            {%- endfor %}
            return true;
        }

        @Override
        public int hashCode() {
            return java.util.Objects.hash(
                {%- for field in rec.fields -%}
                {{ field.java_name }}{% if !loop.last %}, {% endif %}
                {%- endfor -%}
            );
        }

        @Override
        public String toString() {
            return "{{ rec.java_name }}(" +
                {%- for field in rec.fields -%}
                "{{ field.java_name }}=" + {{ field.java_name }}{% if !loop.last %} + ", " + {% endif %}
                {%- endfor -%}
                + ")";
        }
    }
{%- endif -%}

