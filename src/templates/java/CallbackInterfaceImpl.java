{# Included as part of wrapper.java for each CallbackInterface type definition. #}
{# VTable creation, callback handler classes, and registration logic. #}
{%- if cbi is defined -%}
    // --- {{ cbi.java_name }} registration helpers ---

    private static final HandleMap<{{ cbi.java_name }}> handleMap{{ cbi.java_name }}
        = new HandleMap<>();

    public static long register{{ cbi.java_name }}({{ cbi.java_name }} impl) {
        long handle = handleMap{{ cbi.java_name }}.insert(impl);
        // VTable is initialized automatically by JNI_OnLoad via register_callbacks()
        return handle;
    }

    /**
     * Native callback handler methods for {@link {{ cbi.java_name }}}.
     * These are called from Rust (via JNI) when the Rust side invokes
     * a method on the callback interface.
     */
    {%- for method in cbi.methods %}
    public static {% match method.return_type %}{% when Some with (ret) %}{{ ret|java_type }}{% when None %}void{% endmatch %} callback{{ cbi.java_name }}_{{ method.java_name }}(
            long handle
            {%- for arg in method.arguments -%}
            , {{ arg.ty|java_type }} {{ arg.java_name }}
            {%- endfor -%}
    ) {
        {{ cbi.java_name }} impl = handleMap{{ cbi.java_name }}.get(handle);
        if (impl == null) {
            throw new IllegalStateException("{{ cbi.java_name }} callback handle " + handle + " not found");
        }
        try {
            {%- match method.return_type %}
            {%- when Some with (_) %}
            return impl.{{ method.java_name }}(
                {%- for arg in method.arguments -%}
                {{ arg.java_name }}{% if !loop.last %}, {% endif %}
                {%- endfor -%}
            );
            {%- when None %}
            impl.{{ method.java_name }}(
                {%- for arg in method.arguments -%}
                {{ arg.java_name }}{% if !loop.last %}, {% endif %}
                {%- endfor -%}
            );
            {%- endmatch %}
        } catch (Exception ex) {
            throw new RuntimeException("Callback {{ cbi.name }}.{{ method.name }} failed", ex);
        }
    }
    {%- endfor %}
{%- endif -%}

