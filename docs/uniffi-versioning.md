# The UniFFI Versioning System

UniFFI versions are tricky due to many inter-related dependencies between all the UniFFI components.
Libraries and applications depend on UniFFI, leading to situations where `app_foo` depends on `lib_bar` and `lib_baz`,
all with a common UniFFI dependency. This dependency exists in both the Rust code and foreign bindings - each piece must be compatible.

In that situation, each of those crates must depend on a UniFFI version that's compatible with all the others.
Any semver-incompatible change means all these consumers must coordinate an update.
But most of the version bumps aren't actually breaking to consumers of UniFFI without external dependencies.

To help with this, UniFFI has a top-level `uniffi` crate which re-exports all functionality
required by the most common use-cases - if your needs can be met via the top-level crate, that's
what you should use.

While UniFFI remains at 0.x.x, all minor version bumps will be considered by semver to be "breaking".

See [more about semver](https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility)

## Internal bindings

If you use UniFFI with only bindings in the UniFFI repo (ie, Python, Swift, Kotlin), you can take a dependency
on just `uniffi`, which re-exports the top-level functionality from other crates.

## External bindings

Bindings outside of the repo will probably need to to take a direct dependency on
a specific version of `uniffi_bindgen`.

Unfortunately, this means more breakng changes. Eg, let's say `uniffi` is at version `0.42.6`, which internally has `uniffi_bindgen` at version `0.66.6`
* Your `uniffi-go` dependency must depend on `uniffi_bindgen="0.66"`
* A new `uniffi` version `0.42.7` is released, which is semver compatible with the existing `0.42.6` :tada:
* But `uniiffi_bindgen` was bumped to `0.67.0` in this release :sob:

So this semver-compatible bump of `uniffi` was still breaking for consumers of your `uniffi-go` version.

Note that this behaviour is a feature of how we are using versioning.

## What is a breaking change?

This mean the top-level `uniffi` crate needs guidance as to when it should get a breaking version bump:

* Backward incompatible changes to the UDL/proc-macro parsing:
  * Removing a feature.
  * Changing how existing UDL/proc-macro code is handled -- for example if we changed UniFFI functions to return a `Result<>` enum rather than throwing exceptions.
  * Note: Adding new UDL or proc-macro features is not a breaking change.
* Backward incompatible changes to the FFI contract between the scaffolding and bindings code:
  * Changing how FFI functions are named.
  * Changing how FFI functions are called
  * Changing how types are represented.

# UNIFFI_CONTRACT_VERSION

So yeah, what is this?

## How to handle breaking changes

* Increment the minor version of `uniffi`
  * Once we get to `1.0` then this will change to be a major version bump.
* Update the `uniffi_meta::UNIFFI_CONTRACT_VERSION` value
