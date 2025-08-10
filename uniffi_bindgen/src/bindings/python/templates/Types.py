{#
 # Map `Type` instances to an include statement for that type.
 #
 # There is a companion match in `PythonCodeOracle::create_code_type()` which performs a similar function for the
 # Rust code.
 #
 #   - When adding additional types here, make sure to also add a match arm to that function.
 #   - To keep things manageable, let's try to limit ourselves to these 2 mega-matches
 #}
{%- let type_node = self.0 %}
{%- match self.0.ty %}

{%- when Type::Boolean %}
{%- include "BooleanHelper.py" %}

{%- when Type::Int8 %}
{%- include "Int8Helper.py" %}

{%- when Type::Int16 %}
{%- include "Int16Helper.py" %}

{%- when Type::Int32 %}
{%- include "Int32Helper.py" %}

{%- when Type::Int64 %}
{%- include "Int64Helper.py" %}

{%- when Type::UInt8 %}
{%- include "UInt8Helper.py" %}

{%- when Type::UInt16 %}
{%- include "UInt16Helper.py" %}

{%- when Type::UInt32 %}
{%- include "UInt32Helper.py" %}

{%- when Type::UInt64 %}
{%- include "UInt64Helper.py" %}

{%- when Type::Float32 %}
{%- include "Float32Helper.py" %}

{%- when Type::Float64 %}
{%- include "Float64Helper.py" %}

{%- when Type::String %}
{%- include "StringHelper.py" %}

{%- when Type::Bytes %}
{%- include "BytesHelper.py" %}

{%- when Type::Timestamp %}
{%- include "TimestampHelper.py" %}

{%- when Type::Duration %}
{%- include "DurationHelper.py" %}

{%- else %}

{%- endmatch %}
