{# Included from wrapper.java inside a function loop #}
{%- if func is defined -%}
    /**
     {%- if func.docstring.is_some() %}
     * {{ func.docstring.as_ref().unwrap() }}
     *
     {%- endif %}
     */
    public static {% match func.return_type %}{% when Some with (ret) %}{{ ret|java_type }}{% when None %}void{% endmatch %} {{ func.java_name }}(
        {%- for arg in func.arguments -%}
        {{ arg.ty|java_type }} {{ arg.java_name }}{% if !loop.last %}, {% endif %}
        {%- endfor -%}
    ) {
        {{ func.body }}
    }
{%- endif -%}

