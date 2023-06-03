# Stuck!

So here is:

```rust
fn oops() -> anyhow::Result<()> {
    anyhow::bail!("oops");
}
```

which uniffi expands to:

```rust
extern "C" fn uniffi_uniffi_error_types_fn_func_oops(
    call_status: &mut ::uniffi::RustCallStatus,
) -> <::std::result::Result<
    (),
    std::sync::Arc<ErrorInterface>,
> as ::uniffi::LowerReturn<crate::UniFfiTag>>::ReturnType {
...
    ::uniffi::rust_call(
        call_status,
        || {
            <::std::result::Result<(), std::sync::Arc<ErrorInterface>> as ::uniffi::LowerReturn ...>::lower_return(
                match uniffi_lift_args() {
                    Ok(uniffi_args) => oops().map_err(::std::convert::Into::into),
                    Err((arg_name, anyhow_error)) => ...
                },
            )
        },
    )
}
```

# Problem is:
```rust
                    Ok(uniffi_args) => oops().map_err(::std::convert::Into::into),
```

map_err has `anyhow::Error<>`, we want `Arc<ErrorInterface>`, `::into` can't do that.

but - for enum, all `Arc<ErrorInterface>`s above are `ErrorEnum`. So above call is more like:
map_err has `CrateInternalError`, want `CratePublicError`, `::into` can do that.

# Why not emit different code for enum/error

Because procmacros don't know how to tell. UDL uses procmacros here.

procmacros do have simple `looks_like_result`, so maybe we could try and each it to know
the error ident, but that's not going to work still right?