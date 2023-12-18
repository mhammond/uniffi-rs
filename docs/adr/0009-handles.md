# Use handle map handles to pass objects across the FFI

* Status: proposed
* Deciders:
* Consulted:

Discussion and approval: 

ADR-0005 discussion: [PR 430](https://github.com/mozilla/uniffi-rs/pull/430).

## Context and Problem Statement

Prior to ADR-0005 being implemented, UniFFI used a handle-map, implemented similarly to what is being described here.

After the implementation of that ADR, UniFFI moved to managing all objects via `Arc`s to help manage lifetimes, and passed the raw addresses over the FFI.

ADR-0005 provided the following justifications for this model
  1. Clearer generated code.
  2. Ability to pass objects as arguments (https://github.com/mozilla/uniffi-rs/issues/40).
     This was deemed difficult to do with the existing codegen + HandleMap.
  3. Ability for bindings to track object identity (https://github.com/mozilla/uniffi-rs/issues/197).  If two function calls return the same object, then this should result in an identical object on the foreign side.
  4. Because the code is all generated, it was felt that certain classes of bugs caused by improper use of raw pointers would be minimized.
  5. Increased performance by avoiding a level of indirection via HANDLEs.

Since that ADR was approved, this approach was extended to work with unsized types (`Arc<dyn Trait>`) and these pointers are used in more contexts (eg, in dictionaries).

Now that we have several years of experience, it's a good time to revisit some of the reasoning in ADR-0005 to see if they still apply, whether we are getting the benefits we wanted, and whether any improvments or changes should be made:

* The code that deals with these isn't so clear now that not everything is a simple `Arc<T>` - in particular, unsized types (eg, traits) and RustFuture
  [allocation](https://github.com/mozilla/uniffi-rs/blob/fbc6631953a889c7af6e5f1af94de9242589b75b/uniffi_core/src/ffi/rustfuture/mod.rs#L56-L63) / [dellocation](https://github.com/mozilla/uniffi-rs/blob/fbc6631953a889c7af6e5f1af94de9242589b75b/uniffi_core/src/ffi/rustfuture/mod.rs#L124-L125).
  In other words, the comments in that ADR about safety being provided by ensuring the generated code is always becoming less
  true as the complexity of the implementation increases.
* The codegen has progressed such that all of the things deemed difficult with handle maps are now relatively easy to support.
* We believe the performance benefits of using pointers directly without the indirection through a handle will not be
  a major factor of the overall performance characteristics of a UniFFI application.

Furthermore, practice has shown that dealing with raw pointers makes debugging difficult, with errors often resulting in segfaults or UB.
Even if these are all worked out before being merged, development of UniFFI will be improved if we can generate better error messages and
correct stack traces instead of simply crashing (or worse, not crashing but experiencing UB)

The characteristics of ADR-0005 we want to keep include:

* The reverse identity map needed for `[3]`. Even though none of our bndings have yet implemented this (ie,
  the `NimbusClient` example given in https://github.com/mozilla/uniffi-rs/issues/419 would still fail today),
  this capability is something we should not give up lightly.

* The performance argument should be be entirely discarded; we need to balance safety and performance.

### Foreign handles

A related question is how to handle handles to foreign objects that are passed into Rust.
Even though we expect that our implementation might offer help to bindings in this area,
that question is orthogonal to this ADR.

## Considered Options

### [Option 1] Continue using raw Arc pointers to pass Rust objects across the FFI

Stay with the current status quo.

### [Option 2] Use the old `HandleMap` to pass Rust objects across the FFI

We could switch back to the old handle map code, which is still around in the [ffi-support crate](https://github.com/mozilla/ffi-support/blob/main/src/handle_map.rs).
This implements a relatively simple handle-map that uses a `RWLock` to manage concurrency.

See [../handles.md] for details on how this would work.

Handles are passed as a `u64` values, but they only actually use 48 bits.
This works better with JS, where the `Value` type only supports integers up to 53-bits wide.

### [Option 3] Use a `HandleMap` with more performant/complex concurrency strategy

We could switch to something like the [handle map implementation from #1808](https://github.com/bendk/uniffi-rs/blob/d305f7e47203b260e2e44009e37e7435fd554eaa/uniffi_core/src/ffi/slab.rs).
The struct in that code was named `Slab` because it was inspired by the `tokio` `slab` crate.
However, it's very similar to the original UniFFI `HandleMap` and this PR will call it a `HandleMap` to follow in that tradition.

See [../handles.md] for details on how this would work.

### [Option 4] Use a 3rd-party crate to pass Rust objects across the FFI

We could also use a 3rd-party crate to handle this.
The `sharded-slab` crate promises lock-free concurrency and supports generation counters.

## Decision Drivers

## Decision Outcome

???

## Pros and Cons of the Options

### [Option 1] Continue using raw Arc pointers to pass Rust objects across the FFI

* Good, because it has the fastest performance, especially for sized types.
* Good, because it doesn't require code changes.
* Bad, because it's hard to debug errors.
* Bad, because it doesn't offer any safety should errors slip through.

### [Option 2] Re-introduce a handle map.

All handle-map solutions offer:
* Good, because it's easier to debug errors.
* Good, general safety
* Bad, new dependency/code needed.

We can see 3 implementation options:

#### [Option 2.1] Use the original handle map to pass Rust objects across the FFI

* Bad, read-write lock means `insert`/`remove` can block `get`.
* Good, because it works better with Javascript
* Good, because it works with any type, not just `Arc<T>`.
  For example, we might want to pass a handle to a [oneshot::Sender](https://docs.rs/oneshot/latest/oneshot/) across the FFI to implement async callback interface methods.

### [Option 2.2] New handle map with a performant concurrency strategy.
* Better: `get` doesn't require a lock; `insert` and `remove` still do.
* Bad because it requires us to implement and maintain a non-trivial implementation.
* Bad, `append-only-vec` dependecy.
* Good, because it works better with Javascript
* Good, because it works with any type, not just `Arc<T>`.

### [Option 2.3] Use a 3rd-party crate to pass Rust objects across the FFI
* Good, because we don't need to implement it.
* Good, lockless?
* Bad, dependecy on ??.
* Bad, because it makes it harder to implement custom functionality.
  For example, supporting clone to fix https://github.com/mozilla/uniffi-rs/issues/1797 or adding a foreign bit to improve trait interface handling.
