{%- let obj = ci.get_object_definition(name).unwrap() %}

class {{ type_name }}(object):
    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    def __init__(self, {% call py::arg_list_decl(cons) -%}):
        {%- call py::setup_args_extra_indent(cons) %}
        self._pointer = {% call py::to_ffi_call(cons) %}
    {%- when None %}
    {%- endmatch %}

    def __del__(self):
        # In case of partial initialization of instances.
        pointer = getattr(self, "_pointer", None)
        if pointer is not None:
            rust_call(_UniFFILib.{{ obj.ffi_object_free().name() }}, pointer)

    # Used by alternative constructors or any methods which return this type.
    @classmethod
    def _make_instance_(cls, pointer):
        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required pointer.
        inst = cls.__new__(cls)
        inst._pointer = pointer
        return inst

    {% for cons in obj.alternate_constructors() -%}
    @classmethod
    def {{ cons.name()|fn_name }}(cls, {% call py::arg_list_decl(cons) %}):
        {%- call py::setup_args_extra_indent(cons) %}
        # Call the (fallible) function before creating any half-baked object instances.
        pointer = {% call py::to_ffi_call(cons) %}
        return cls._make_instance_(pointer)
    {% endfor %}

    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    def {{ meth.name()|fn_name }}(self, {% call py::arg_list_decl(meth) %}):
        {%- call py::setup_args_extra_indent(meth) %}
        return {{ return_type|lift_fn }}(
            {% call py::to_ffi_call_with_prefix("self._pointer", meth) %}
        )

    {%- when None -%}
    def {{ meth.name()|fn_name }}(self, {% call py::arg_list_decl(meth) %}):
        {%- call py::setup_args_extra_indent(meth) %}
        {% call py::to_ffi_call_with_prefix("self._pointer", meth) %}
    {% endmatch %}
    {% endfor %}

{%- match obj.foreign_impl_name() %}
{%- when Some(name) %}
{% if self.include_once_check("CallbackInterfaceRuntime.py") %}{% include "CallbackInterfaceRuntime.py" %}{% endif %}
{%- let ffi_converter_name_trait = format!("{}Trait", ffi_converter_name) %}
# The handle map to manage Python implemented traits
{{ ffi_converter_name_trait }}HandleMap = ConcurrentArcMap()

# This is very hacky!
# * probably should use RustCallStatus?
# * should use iter_ffi_function_definitions to get them automatically emitted.
_UniFFILib.{{ obj.ffi_object_new().name()}}.argtypes = (
    ctypes.c_uint64,
)
_UniFFILib.{{ obj.ffi_object_new().name()}}.restype = ctypes.c_void_p

{%- let foreign_callback = format!("foreignCallback{}", canonical_type_name) %}

# Very very similar to callbacks
def py_{{ foreign_callback }}(handle, method, args, buf_ptr):
    {% for meth in obj.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name %}
    def {{ method_name }}(python_callback, args):
        {#- Unpacking args from the RustBuffer #}
        rval = None
        {%- if meth.arguments().len() != 0 -%}
        {#- Calling the concrete callback object #}
        with args.consumeWithStream() as buf:
            rval = python_callback.{{ meth.name()|fn_name }}(
                {% for arg in meth.arguments() -%}
                {{ arg|read_fn }}(buf)
                {%- if !loop.last %}, {% endif %}
                {% endfor -%}
            )
        {% else %}
        rval = python_callback.{{ meth.name()|fn_name }}()
        {% endif -%}

        {#- Packing up the return value into a RustBuffer #}
        {%- match meth.return_type() -%}
        {%- when Some with (return_type) -%}
        with RustBuffer.allocWithBuilder() as builder:
            {{ return_type|write_fn }}(rval, builder)
            return builder.finalize()
        {%- else -%}
        return RustBuffer.alloc(0)
        {% endmatch -%}
        # TODO catch errors and report them back to Rust.
        # https://github.com/mozilla/uniffi-rs/issues/351
    {% endfor %}

    cb = {{ ffi_converter_name_trait }}HandleMap.get_ob_for_handle(handle)
    if not cb:
        raise InternalError("No callback in handlemap; this is a Uniffi bug")

    if method == IDX_CALLBACK_FREE:
        # XXXXXX - drop it from the map with explicit drop of the arc?
        print("TODO: Free!")
#        {{ ffi_converter_name }}.drop(handle)
        # No return value.
        # See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
        return 0


    {% for meth in obj.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name -%}
    if method == {{ loop.index }}:
        buf_ptr[0] = {{ method_name }}(cb, args)
        # Value written to out buffer.
        # See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
        return 1
    {% endfor %}

    # This should never happen, because an out of bounds method index won't
    # ever be used. Once we can catch errors, we should return an InternalException.
    # https://github.com/mozilla/uniffi-rs/issues/351

    # An unexpected error happened.
    # See docs of ForeignCallback in `uniffi/src/ffi/foreigncallbacks.rs`
    return -1

# We need to keep this function reference alive:
# if they get GC'd while in use then UniFFI internals could attempt to call a function
# that is in freed memory.
# That would be...uh...bad. Yeah, that's the word. Bad.
{{ foreign_callback }} = FOREIGN_CALLBACK_T(py_{{ foreign_callback }})

# The FfiConverter which transforms the Callbacks in to Handles to pass to Rust.
rust_call(lambda err: _UniFFILib.{{ obj.ffi_init_callback().name() }}({{ foreign_callback }}, err))
{{ ffi_converter_name }} = FfiConverterCallbackInterface({{ foreign_callback }})


{%- else %}
{%- endmatch %}

class {{ ffi_converter_name }}:
    @classmethod
    def read(cls, buf):
        ptr = buf.readU64()
        if ptr == 0:
            raise InternalError("Raw pointer value was null")
        return cls.lift(ptr)

    @classmethod
    def write(cls, value, buf):
        if not isinstance(value, {{ type_name }}):
            raise TypeError("Expected {{ type_name }} instance, {} found".format(value.__class__.__name__))
        buf.writeU64(cls.lower(value))

    @staticmethod
    def lift(value):
        return {{ type_name }}._make_instance_(value)

    @staticmethod
    def lower(value):
        {%- match obj.foreign_impl_name() %}
        {%- when Some(name) %}
        {%- let ffi_converter_name_trait = format!("{}Trait", ffi_converter_name) %}
        # If the instance we have is a Python implemented version, convert it to a Rust version.
        if isinstance(value, {{ name }}):
            def inst_getter(handle):
                pointer = _UniFFILib.{{ obj.ffi_object_new().name()}}(handle)
                return {{ type_name }}._make_instance_(pointer)

            value = {{ ffi_converter_name_trait }}HandleMap.insert(value, inst_getter)
        {%- else %}
        {%- endmatch %}
        if not isinstance(value, {{ type_name }}):
            raise TypeError("Expected {{ type_name }} instance, {} found".format(value.__class__.__name__))
        return value._pointer
