# The `uniffi::Error` derive

The `Error` derive registers a type as an error and can be used on any enum that the `Enum` derive also accepts.
By default, it exposes any variant fields to the foreign code.
This type can then be used as the `E` in a `Result<T, E>` return type of an exported function or method.
The generated foreign function for an exported function with a `Result<T, E>` return type
will have the result's `T` as its return type and throw the error in case the Rust call returns `Err(e)`.

```rust
#[derive(uniffi::Error)]
pub enum MyError {
    MissingInput,
    IndexOutOfBounds {
        index: u32,
        size: u32,
    }
    // tuple-enums work.
    Generic(String),
}

#[uniffi::export]
fn do_thing() -> Result<(), MyError> {
    // ...
}
```

You can also use the helper attribute `#[uniffi(flat_error)]` to expose just the variants but none of the fields.
In this case the error will be serialized using Rust's `ToString` trait
and will be accessible as the only field on each of the variants.
The types of the fields can be any UniFFI supported type and don't need to implement any special traits.

```rust
#[derive(uniffi::Error)]
#[uniffi(flat_error)]
pub enum MyApiError {
    Http(reqwest::Error),
    Json(serde_json::Error),
}

// ToString is not usually implemented directly, but you get it for free by implementing Display.
// This impl could also be generated by a proc-macro, for example thiserror::Error.
impl std::fmt::Display for MyApiError {
    // ...
}

#[uniffi::export]
fn do_http_request() -> Result<(), MyApiError> {
    // ...
}
```