// For each Object definition, we assume the caller has provided an appropriately-shaped `struct T`
// with an `impl` for each method on the object. We create an `Arc<T>` for "safely" handing out
// references to these structs to foreign language code, and we provide a `pub extern "C"` function
// corresponding to each method.
//
// (Note that "safely" is in "scare quotes" - that's because we use functions on an `Arc` that
// that are inherently unsafe, but the code we generate is safe in practice.)
//
// If the caller's implementation of the struct does not match with the methods or types specified
// in the UDL, then the rust compiler will complain with a (hopefully at least somewhat helpful!)
// error message when processing this generated code.

// All Object structs must be `Sync + Send`. The generated scaffolding will fail to compile
// if they are not, but unfortunately it fails with an unactionably obscure error message.
// By asserting the requirement explicitly, we help Rust produce a more scrutable error message
// and thus help the user debug why the requirement isn't being met.
{% if obj.is_trait() -%}
uniffi::deps::static_assertions::assert_impl_all!(Box<dyn r#{{ obj.type_name() }}>: Sync, Send);
{%- else -%}
uniffi::deps::static_assertions::assert_impl_all!(r#{{ obj.type_name() }}: Sync, Send);
{%- endif -%}

{% let ffi_free = obj.ffi_object_free() -%}
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn {{ ffi_free.name() }}(ptr: *const std::os::raw::c_void, call_status: &mut uniffi::RustCallStatus) {
    uniffi::call_with_output(call_status, || {
        assert!(!ptr.is_null());
        {% if obj.is_trait() -%}
        {#- traits are lowered as a Box<T> -#}
        // XXX - not sure this is correct.
        drop(unsafe { Box::from_raw(ptr as *mut Box<dyn r#{{ obj.type_name() }}>) })
        {%- else -%}
        {#- turn it into an Arc and explicitly drop it. #}
        drop(unsafe { std::sync::Arc::from_raw(ptr as *const r#{{ obj.type_name() }}) })
        {%- endif -%}
    })
}

{%- for cons in obj.constructors() %}
    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn r#{{ cons.ffi_func().name() }}(
        {%- call rs::arg_list_ffi_decl(cons.ffi_func()) %}) -> *const std::os::raw::c_void /* *const {{ obj.name() }} */ {
        uniffi::deps::log::debug!("{{ cons.ffi_func().name() }}");

        // If the constructor does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_constructor_call(obj, cons) %}
    }
{%- endfor %}

{%- for meth in obj.methods() %}
    #[doc(hidden)]
    #[no_mangle]
    #[allow(clippy::let_unit_value)] // Sometimes we generate code that binds `_retval` to `()`.
    pub extern "C" fn r#{{ meth.ffi_func().name() }}(
        {%- call rs::arg_list_ffi_decl(meth.ffi_func()) %}
    ) {% call rs::return_signature(meth) %} {
        uniffi::deps::log::debug!("{{ meth.ffi_func().name() }}");
        // If the method does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_method_call(obj, meth) %}
    }
{% endfor %}

{% if obj.is_trait() -%}
// Objects which describe traits are able to be implemented by foreign bindings.
// So we implement a struct implementing the trait to enable this.
// XXX - we should make this conditional? Not all traits actually need that.
{% let foreign_callback_internals = format!("foreign_callback_{}_internals", obj.name())|upper -%}

// Register a foreign callback for getting across the FFI.
#[doc(hidden)]
static {{ foreign_callback_internals }}: uniffi::ForeignCallbackInternals = uniffi::ForeignCallbackInternals::new();

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn {{ obj.ffi_init_callback().name() }}(callback: uniffi::ForeignCallback, _: &mut uniffi::RustCallStatus) {
    {{ foreign_callback_internals }}.set_callback(callback);
    // The call status should be initialized to CALL_SUCCESS, so no need to modify it.
}

#[doc(hidden)]
#[derive(Debug)]
struct Foreign{{ obj.type_name() }} {
    handle: u64 
}

impl Drop for Foreign{{ obj.type_name() }} {
    fn drop(&mut self) {
        println!("FOREIGN drop");
        panic!();
    }
}


impl r#{{ obj.type_name() }} for Foreign{{ obj.type_name() }} {
    {%- for meth in obj.methods() %}

    {#- Method declaration #}
    fn r#{{ meth.name() -}}
    ({% call rs::arg_list_decl_with_prefix("&self", meth) %})
    {%- match meth.return_type() %}
    {%- when Some with (return_type) %} -> {{ return_type.borrow()|type_rs }}
    {% else -%}
    {%- endmatch -%} {
    {#- Method body #}
        uniffi::deps::log::debug!("{{ obj.name() }}.{{ meth.name() }}");

    {#- Packing args into a RustBuffer #}
        {% if meth.arguments().len() == 0 -%}
        let args_buf = Vec::new();
        {% else -%}
        let mut args_buf = Vec::new();
        {% endif -%}
        {%- for arg in meth.arguments() %}
        {{ arg.type_().borrow()|ffi_converter }}::write(r#{{ arg.name() }}, &mut args_buf);
        {%- endfor -%}
        let args_rbuf = uniffi::RustBuffer::from_vec(args_buf);

    {#- Calling into foreign code. #}
        let callback = {{ foreign_callback_internals }}.get_callback().unwrap();

        let ret_rbuf = unsafe {
            // SAFETY:
            // * We're passing in a pointer to an empty buffer.
            //   * Nothing allocated, so nothing to drop.
            // * We expect the callback to write into that a valid allocated instance of a
            //   RustBuffer.
            // * A positive return value signals success.
            let mut ret_rbuf = uniffi::RustBuffer::new();
            let ret = callback(self.handle, {{ loop.index }}, args_rbuf, &mut ret_rbuf);
            match ret {
                0 => uniffi::RustBuffer::new(),
                _ if ret < 0 => panic!("Callback failed"),
                _ => ret_rbuf
            }
        };

    {#- Unpacking the RustBuffer to return to Rust #}
        {% match meth.return_type() -%}
        {% when Some with (return_type) -%}
        let vec = ret_rbuf.destroy_into_vec();
        let mut ret_buf = vec.as_slice();
        {{ return_type|ffi_converter }}::try_read(&mut ret_buf).unwrap()
        {%- else -%}
        uniffi::RustBuffer::destroy(ret_rbuf);
        {%- endmatch %}
    }
    {%- endfor %}
}

// A function for making one of our impls
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn {{ obj.ffi_object_new().name() }}(handle: u64) -> *const std::os::raw::c_void {
    let new = Foreign{{ obj.type_name() }} { handle };
    let arc = std::sync::Arc::new(new);
    {{ obj.type_().borrow()|ffi_converter }}::lower(arc)
}

{%- endif -%}
