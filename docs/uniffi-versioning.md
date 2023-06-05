# The UniFFI Versioning System

UniFFI versions are tricky since both libraries and applications depend on UniFFI.  This can lead to situations where `app_foo` depends on `lib_bar` and `lib_baz`, and all of those depend on UniFFI.  In that situation, each of those crates must depend on a UniFFI version that's compatible with all the others.

All of this means that breaking changes to UniFFI are costly for our consumers.  In the situation above, if `lib_bar` upgrades UniFFI with a breaking change, then both `lib_baz` and `app_foo` would also need to upgrade UniFFI and all of these changes would need to be coordinated together.

Therefore, we want to have a system which minimizes the amount of breaking changes for consumers.

## Breaking changes and SemVer

UniFFI follows the [SemVer rules from the Cargo Book](https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility) which states "Versions are considered compatible if their left-most non-zero major/minor/patch component is the same".  Since all crates are currently on major version `0`, this means a breaking change will result in a bump of the minor version.  Once we are on major version `1` or later, a breaking change will result in a major version bump.  In the text below, these are referred to as a "breaking version bump".

## How consumers should depend on UniFFI

Crates that use UniFFI to generate scaffolding or bindings should only have a direct dependency to the `uniffi` crate, which re-exports the top-level functionality from other crates:

* Generating the scaffolding via a build script
* Generating the bindings via a CLI
* Generating the scaffolding or bindings programmatically

Because the crates only directly depend on `uniffi`, they only need to care about the `uniffi` version and can ignore the versions of sub-dependencies.  This means that breaking changes in `uniffi_bindgen` won't be a breaking change for consumers, as long as it doesn't affect the functionality listed above.

## How binding generators should depend on UniFFI

Crates that use uniffi_bindgen to implement bindings for 3rd party languages should always have
a direct dependency to a specific uniffi-bindgen version. This version will be incremented to
indicate a breaking change whenever the implementation of bindings might need to change, even if
these changes would not be noticed by the user (ie, even when we did not bump the major "uniffi"
version.)

Unfortunately, this might well mean that in some cases the users of some bindings might be
unable to update to what the UniFFI project considers a minor (ie, non breaking) change.
For example:

* UniFFI version X ships with uniffi_bindgen version Y
* UniFFI implements internal changes to uniffi_bindgen - nothing which changes the FFI or how types
  are named, but enough that binding generators require some work to be compatible.
* The next UniFFI release is likely ship with trhe version for `uniffi` indicating a semver compatible
  change to that crate, but a semver breaking change for `uniffi_bindgen`

The end result is that:
* Most users depend only on the top-level `uniffi` version, so see a semver compatible change.
* Users who depends on 3rd party bindings will find that those bindings declare they need
  uniffi_bindgen version Y - but the new release comes with a semver incompatible version.
* Users find themselves unable to update UniFFI to what appears to be semver compatible version.

Note that this behaviour is a feature of how we are using versioning.


## What is a breaking change?

To expand on the previous point, here are the scenarios where `uniffi` should get a breaking version bump:

* Backward incompatible changes to the UDL/proc-macro parsing:
  * Removing a feature.
  * Changing how existing UDL/proc-macro code is handled -- for example if we changed UniFFI functions to return a `Result<>` enum rather than throwing exceptions.
  * Note: Adding new UDL or proc-macro features is not a breaking change.
* Backward incompatible changes to the FFI contract between the scaffolding and bindings code:
  * Changing how FFI functions are named.
  * Changing how FFI functions are called
  * Changing how types are represented.
(XXX - is the above correct? Shouldn't that just be the bindgen version? See comments above?)

## How to handle breaking changes

* Increment the minor version of `uniffi`
  * Once we get to `1.0` then this will change to be a major version bump.
* Update the `uniffi_meta::UNIFFI_CONTRACT_VERSION` value
