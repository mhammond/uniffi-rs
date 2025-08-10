class {{ self_type.type_name }}:
    {{ docstring|docstring(4) -}}
    {% for field in fields -%}
    {{ field.name }}: {{ field.ty.type_name}}
    {{ docstring|docstring(4) -}}
    {% endfor -%}

    {%- if !fields.is_empty() %}
    def __init__(self, *, {% for field in fields %}
    {{- field.name }}: {{- field.ty.type_name}}
    {%- if field.default.is_some() %} = _DEFAULT{% endif %}
    {%- if !loop.last %}, {% endif %}
    {%- endfor %}):
        {%- for field in fields %}
        {%- match field.default %}
        {%- when None %}
        self.{{ field.name }} = {{ field.name }}
        {%- when Some(default) %}
        if {{ field.name }} is _DEFAULT:
            self.{{ field.name }} = {{ default.py_default }}
        else:
            self.{{ field.name }} = {{ field.name }}
        {%- endmatch %}
        {%- endfor %}
    {%- endif %}

    def __str__(self):
        return "{{ self_type.type_name }}({% for field in fields %}{{ field.name }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in fields %}self.{{ field.name }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in fields %}
        if self.{{ field.name }} != other.{{ field.name }}:
            return False
        {%- endfor %}
        return True

class {{ self_type.ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def read(buf):
        return {{ self_type.type_name }}(
            {%- for field in fields %}
            {{ field.name }}={{ field.ty.ffi_converter_name }}.read(buf),
            {%- endfor %}
        )

    @staticmethod
    def check_lower(value):
        {%- if fields.is_empty() %}
        pass
        {%- else %}
        {%- for field in fields %}
        {{ field.ty.ffi_converter_name }}.check_lower(value.{{ field.name }})
        {%- endfor %}
        {%- endif %}

    @staticmethod
    def write(value, buf):
        {%- if !fields.is_empty() %}
        {%- for field in fields %}
        {{ field.ty.ffi_converter_name }}.write(value.{{ field.name }}, buf)
        {%- endfor %}
        {%- else %}
        pass
        {%- endif %}
