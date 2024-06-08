# Remote types: Extending remote interfaces.

* Status: proposed
* Deciders: Uniffi developers
* Date: 2024-06-08

## Context and Problem Statement

[ADR-0009](./0009-remote-types-interfaces-and-proc-macros.md) discusses remote types.

Remote interfaces, as described in that ADR, allow proc-macros to express:

```
uniffi::remote!(
    impl ExtInterface {
        fn existing_method(&self) -> String;
    }
);
```
to expose an existing, externally declared method to foreign bindings.

This ADR explores how we might choose to locally extend these types - ie, how a crate might expose to foreign bindings locally defined Rust methods. For example:

```
#[uniffi::remote]
impl ExtInterface {
    fn local_method(&self) -> String {
        "hello".to_string()
    }
}
```

As a concrete example we will explore `anyhow::Error`

## Etc

```rust
type AnyhowError = anyhow::Error;

uniffi::remote!(
    impl AnyhowError {
        // Expose the `to_string` method (technically, `to_string` comes from the `Display` trait, but that
        // doesn't matter for foreign consumers.  Since the item definition is not used for the
        // scaffolding code and will not be present in the expanded code, it can be left empty.
        pub fn to_string(&self) -> String { }
    }
);
```

One issue with this approach is that we can only export methods that are compatible with UniFFI.
However, users could add an extension trait to create adapter methods that are UniFFI compatible:

```rust
type AnyhowError = anyhow::Error;

pub trait AnyhowErrorExt {
    // [anyhow::Error::is] is a generic method, which can't be exported by UniFFI,
    // but we can export specialized versions for specific types.
    fn is_foo_error(&self) -> bool;
    fn is_bar_error(&self) -> bool;

    // `to_string` is not the best name for the foreign code, let's rename it.
    fn message(&self) -> String;
}

impl AnyhowErrorExt for anyhow::Error {
    fn is_foo_error(&self) -> bool {
        self.is::<foo::Error>()
    }

    fn is_bar_error(&self) -> bool {
        self.is::<bar::Error>()
    }

    fn message(&self) -> String {
        self.to_string()
    }
}

uniffi::remote!(
    impl AnyhowError {
        pub fn is_foo_error(&self) -> bool { }
        pub fn is_bar_error(&self) -> bool { }
        pub fn message(&self) -> String { }
    }
);
```

The above code could be shortened using the [extend](https://crates.io/crates/extend) crate.
UniFFI could also offer syntactic sugar:


```rust
type AnyhowError = anyhow::Error;

// This expands to the equivelent code as the above block
uniffi::remote_extend!(
    impl AnyhowError {
        fn is_foo_error(&self) -> bool {
            self.is::<foo::Error>()
        }

        fn is_bar_error(&self) -> bool {
            self.is::<bar::Error>()
        }

        fn message(&self) -> String {
            self.to_string()
        }
    }
);
```

