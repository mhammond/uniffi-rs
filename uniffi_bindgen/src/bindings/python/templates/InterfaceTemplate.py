{%- let ffi_converter_name = self_type.ffi_converter_name %}
{%- include "Protocol.py" %}

class {{ name }}({{ base_classes|join(", ") }}):
    {{ docstring|docstring(4) }}
    _handle: ctypes.c_uint64

{%- for cons in constructors %}
{%-     let callable = cons.callable %}
{%-     if callable.is_primary_constructor() && callable.is_async %}
    def __init__(self, *args, **kw):
        raise ValueError("async constructors not supported.")
{%-     elif callable.is_primary_constructor() %}
    def __init__(self, {% include "CallableArgs.py" %}):
        {{ cons.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
{%-     else %}
    @classmethod
    {% if callable.is_async %}async {% endif %}def {{ callable.name }}(cls, {% include "CallableArgs.py" %}):
        {{ cons.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
{%-     endif %}
{%- endfor %}

{%- if !has_primary_constructor %}
    {# Define __init__ to prevent construction without a handle, which can confuse #}
    def __init__(self, *args, **kwargs):
        raise ValueError("This class has no default constructor")
{%- endif %}

    def __del__(self):
        # In case of partial initialization of instances.
        handle = getattr(self, "_handle", None)
        if handle is not None:
            _uniffi_rust_call(_UniffiLib.{{ ffi_func_free.0 }}, handle)

    def _uniffi_clone_handle(self):
        return _uniffi_rust_call(_UniffiLib.{{ ffi_func_clone.0 }}, self._handle)

    # Used by alternative constructors or any methods which return this type.
    @classmethod
    def _uniffi_make_instance(cls, handle):
        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required handle.
        inst = cls.__new__(cls)
        inst._handle = handle
        return inst

{%- for meth in methods -%}
{%-     let callable = meth.callable %}
    {% if callable.is_async %}async {% endif %}def {{ callable.name }}(self, {% include "CallableArgs.py" %}):
        {{ meth.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
{%- endfor %}

{%- if let Some(fmt) = uniffi_trait_methods.debug_fmt %}
{%-    let callable = fmt.callable %}
    # The Rust `Debug::fmt`` implementation.
    def __repr__(self) -> {{ callable.return_type.type_name }}:
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}
{%- endif %}
{%- if let Some(fmt) = uniffi_trait_methods.display_fmt %}
{%-    let callable = fmt.callable %}
    # The Rust `Display::fmt`` implementation.
    def __str__(self) -> {{ callable.return_type.type_name }}:
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}

{%- endif %}
{%- if let Some(eq) = uniffi_trait_methods.eq_eq %}
{%-    let callable = eq.callable %}
    # The Rust `Eq::eq`` implementation.
    def __eq__(self, other: object) -> {{ callable.return_type.type_name }}:
        if not isinstance(other, {{ self_type.type_name }}):
            return NotImplemented

        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}
{%- endif %}
{%- if let Some(ne) = uniffi_trait_methods.eq_ne %}
{%-    let callable = ne.callable %}
    # The Rust `Eq::ne`` implementation.
    def __ne__(self, other: object) -> {{ callable.return_type.type_name }}:
        if not isinstance(other, {{ self_type.type_name }}):
            return NotImplemented
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}
{%- endif %}
{%- if let Some(hash) = uniffi_trait_methods.hash_hash %}
{%-    let callable = hash.callable %}
    # The Rust `Hash::hash`` implementation.
    def __hash__(self) -> {{ callable.return_type.type_name }}:
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}

{%- endif %}
{%- if let Some(cmp) = uniffi_trait_methods.ord_cmp %}
{%-    let callable = cmp.callable %}
    # The Rust `Ord::cmp`` implementation.
    # lol/sob, python3 got rid of the perfect python2 `.__cmp__()` :(
    def __rust_cmp__(self, other) -> {{ callable.return_type.type_name }}:
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter %}

    def __lt__(self, other) -> bool:
        return self.__rust_cmp__(other) < 0

    def __le__(self, other) -> bool:
        return self.__rust_cmp__(other) <= 0

    def __gt__(self, other) -> bool:
        return self.__rust_cmp__(other) > 0

    def __ge__(self, other) -> bool:
        return self.__rust_cmp__(other) >= 0
{%- endif %}

{# Objects as error #}
{%- if self_type.is_used_as_error %}
{# Due to some mismatches in the ffi converter mechanisms, errors are forced to be a RustBuffer #}
class {{ ffi_converter_name }}__as_error(_UniffiConverterRustBuffer):
    @classmethod
    def read(cls, buf):
        raise NotImplementedError()

    @classmethod
    def write(cls, value, buf):
        raise NotImplementedError()

    @staticmethod
    def lift(value):
        # Errors are always a rust buffer holding a handle - which is a "read"
        with value.consume_with_stream() as stream:
            return {{ ffi_converter_name }}.read(stream)

    @staticmethod
    def lower(value):
        raise NotImplementedError()

{%- endif %}

{%- match vtable %}
{%- when None %}
{# simple case: the interface can only be implemented in Rust #}
class {{ ffi_converter_name }}:
    @staticmethod
    def lift(value: int) -> {{ name }}:
        return {{ name }}._uniffi_make_instance(value)

    @staticmethod
    def check_lower(value: {{ name }}):
        if not isinstance(value, {{ name }}):
            raise TypeError("Expected {{ name }} instance, {} found".format(type(value).__name__))

    @staticmethod
    def lower(value: {{ name }}) -> ctypes.c_uint64:
        return value._uniffi_clone_handle()

    @classmethod
    def read(cls, buf: _UniffiRustBuffer) -> {{ name }}:
        ptr = buf.read_u64()
        if ptr == 0:
            raise InternalError("Raw handle value was null")
        return cls.lift(ptr)

    @classmethod
    def write(cls, value: {{ name }}, buf: _UniffiRustBuffer):
        buf.write_u64(cls.lower(value))

{%- when Some(vtable) %}
{#
 # The interface can be implemented in Rust or Python

 # * Generate a callback interface implementation to handle the Python side
 # * In the FfiConverter, check which side a handle came from to know how to handle correctly.
 #}

{%- let trait_impl=format!("_UniffiTraitImpl{}", self.name) %}
{%- include "CallbackInterfaceImpl.py" %}

class {{ ffi_converter_name }}:
    _handle_map = _UniffiHandleMap()

    @staticmethod
    def lift(value: int):
        if (value & 1) == 0:
            # Rust-generated handle, construct a new class that uses the handle to implement the
            # interface
            return {{ name }}._uniffi_make_instance(value)
        else:
            # Python-generated handle, get the object from the handle map
            return {{ ffi_converter_name }}._handle_map.remove(value)

    @staticmethod
    def check_lower(value: {{ protocol.name }}):
        if not isinstance(value, {{ protocol.name }}):
            raise TypeError("Expected {{ protocol.name }} subclass, {} found".format(type(value).__name__))

    @staticmethod
    def lower(value: {{ protocol.name }}):
         if isinstance(value, {{ name }}):
            # Rust-implementated object.  Clone the handle and return it
            return value._uniffi_clone_handle()
         else:
            # Python-implementated object, generate a new vtable handle and return that.
            return {{ ffi_converter_name }}._handle_map.insert(value)

    @classmethod
    def read(cls, buf: _UniffiRustBuffer):
        ptr = buf.read_u64()
        if ptr == 0:
            raise InternalError("Raw handle value was null")
        return cls.lift(ptr)

    @classmethod
    def write(cls, value: {{ protocol.name }}, buf: _UniffiRustBuffer):
        buf.write_u64(cls.lower(value))
{%- endmatch %}
