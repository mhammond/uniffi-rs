# Remote types: proc-macros and interfaces

* Status: proposed
* Deciders: Uniffi developers
* Date: 2024-05-24

## Context and Problem Statement

We want to expose APIs which are described naturally in our Rust implementation. For example,

```rust
fn set_level(level: log::Level) { ... }
```

Where `log::Level` is a "remote type", defined in a 3rd-party crate.
[UDL supports enums, records and interfaces](https://github.com/mozilla/uniffi-rs/blob/main/fixtures/ext-types/external-crate/src/lib.rs).

This works in UDL because it re-describes the type. ie:

```rust
type LogLevel = log::Level;
use ExternalCrate::Caller;
```
[our UDL re-describes it](https://github.com/mozilla/uniffi-rs/blob/ce178e9fefcbe9cd5ead92e7dc3c1469dd2c393a/fixtures/ext-types/lib/src/ext-types-lib.udl#L58-L72):
```idl
enum LogLevel {
    "Error",
    ...,
};

interface Caller {
    string get();
};

dictionary Etc { ... };
```

Our proc-macros similarly need some way to learn learn the shape of the types.

This ADR is concerned with how we describe these remote types to proc-macros, in a way that achieves parity with UDL.
Future ADRs may look to extend capabilities, but this does not.

## Considered Options

### [Option 1] expose remote types directly

We could continue to re-describe remote types directly, similar to how it currently works in UDL, using our macros.

```rust
type LogLevel = log::Level;

uniffi::remote!(
    pub enum LogLevel {
        Error = 1,
    }
);

type ExtInterface = external::Interface;

uniffi::remote!(
    impl ExtInterface {
        pub fn existing_method(&self) -> String;
    }
);
```

The `remote!` macro would generate all scaffolding code needed to handle these types.
The `enum LogLevel` item would not end up in the expanded code.

### [Option 1a] use an attribute macro

The same idea could also be spelled out using an attribute macro rather than a function-like macro:

```rust
#[uniffi::remote]
pub enum LogLevel {
    Error = 1,
}

#[uniffi::remote]
impl ExtInterface {
    pub fn existing_method(&self) -> String;
}
```

### [Option 2] use custom-type conversion to expose the type

An alternate strategy would be to use a custom-type conversion from that type into a local type that does implement the UniFFI traits.
These examples will use the custom type syntax from #2087, since I think it looks nicer than the current `UniffiCustomTypeConverter` based code.

```rust
/// Define a type that mirrors `Log::Level`
#[derive(uniffi::Enum)]
pub enum LogLevel {
    Error = 1,
}

/// Define a custom type conversion from `log::Level` to the above type.
uniffi::custom_type!(log::Level, LogLevel, {
  from_custom: |l| match l {
    log::Level::Error => LogLevel::Error,
  },
  try_into_custom: |l| Ok(match l ({
    LogLevel::Error => log::Level::Error,
  })
})

/// Interfaces can use the newtype pattern
#[derive(uniffi::Object)]
pub struct AnyhowError(anyhow::Error);

uniffi::custom_newtype!(anyhow::Error, AnyhowError).

// In the context of ADR-0010, We could define methods directly with this approach, no need for extension traits.
#[uniffi::export]
impl AnyhowError {
    fn message(&self) -> String {
        self.0.to_string()
    }
}
```

[Note that we already support custom types wrapping interfaces](https://github.com/mozilla/uniffi-rs/blob/ce178e9fefcbe9cd5ead92e7dc3c1469dd2c393a/fixtures/ext-types/custom-types/src/lib.rs#L158-L168), so we could probably extend the existing support in this way regardless.

#### Two types

One drawback of this approach is that we have to equivalent, but different types.
Rust code would need to use `anyhow::Error` in their signatures, while foreign code would use `AnyhowError`.
Since the types are almost exactly the same, but named slightly different and with slightly different methods, it can be awkward to document this distinction -- both by UniFFI for library authors and by library authors for their consumers.

### [Option 3] hybrid approach

We could try to combine the best of both worlds by implementing the FFI traits directly for records/structs and using the converter approach for interfaces.

XXX - what does this mean?

## Pros and Cons of the Options

### [Option 1] expose remote types directly

* Good, because both the foreign code and Rust code can use the same type names.
* Good, because it has a low amount of boilerplate code (assuming we provide the `remote_extend!` macro).
* Bad, because we need to define extension traits for remote interfaces types.
* Bad, because it can be confusing to see a type declaration that the `uniffi::remote!` macro will eventually throw away.

### [Option 1a] use an attribute macro

(compared to option 1)

* Good, because the item declaration looks more natural.
* Bad, since the natural looking item declaration is thrown away, there is even more possibility for confusion.

### [Option 2] use custom-type conversion to expose the type

* Good, because adding methods to remote interface types is natural.
* Bad, because having two equivalent but different types could cause confusion.
* Bad, because users have to write out the trivial struct/enum conversions.

### [Option 3] hybrid approach

* Good, because adding methods to remote interface types is natural.
* Good, because both the foreign code and Rust code can use the same type names for struct/record types.
* Bad, because there will be two types for interface types.
* Good, because it has a low amount of boilerplate code.
* Bad, because mixing the two systems increases the overall complexity and risk of confusion.

## Decision Outcome

???

