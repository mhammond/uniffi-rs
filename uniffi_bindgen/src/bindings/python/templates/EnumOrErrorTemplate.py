{# For enums, there are either an error *or* an enum, they can't be both. #}
{%- if self_type.is_used_as_error %}
{%- include "ErrorTemplate.py" %}
{%- else %}
{%- include "EnumTemplate.py" %}
{% endif %}
