# Foreign-language bindings

As stated in the [Overview](../Overview.md), this library and tutorial does not cover *how* to ship a Rust library on mobile, but how to generate bindings for it, so this section will only cover that.

## Creating the bindgen binary

First, make sure you have installed all the [prerequisites](./Prerequisites.md).

Ideally you would then run the `uniffi-bindgen` binary from the `uniffi` crate to generate your bindings,
but if not on [Cargo nightly](https://doc.rust-lang.org/cargo/reference/unstable.html#artifact-dependencies),
you need to create a binary in your project that does the same thing.

Add the following to your `Cargo.toml`:

```toml
[[bin]]
# This can be whatever name makes sense for your project, but the rest of this tutorial assumes uniffi-bindgen.
name = "uniffi-bindgen"
path = "uniffi-bindgen.rs"
```

Create `uniffi-bindgen.rs`:
```rust
fn main() {
    uniffi::uniffi_bindgen_main()
}
```

You can now run `uniffi-bindgen` from your project using `cargo run --features=uniffi/cli --bin uniffi-bindgen [args]`

### Multi-crate workspaces

In a multiple crates workspace, you can create a separate crate for running `uniffi-bindgen`:
  - Name the crate `uniffi-bindgen`, add it to your workspace.
  - Add this dependency to `Cargo.toml`: `uniffi = {version = "0.XX.0", features = ["cli"] }`
  - As above, add the `uniffi-bindgen` binary target

Then your can run `uniffi-bindgen` from any create in your project using `cargo run -p uniffi-bindgen [args]`

`unnif-bindgen` can take as input either a UDL file or a built library file.

A library file is recommended as it supports more scenarios.

A UDL file means you don't need access to the built binary, but it doesn't support multi-crate
environments or procmacros. This facility is likely to eventually be deprecated.


## Running uniffi-bindgen using a library file

Use `generate --library` and specify the path to the cdylib file built for your library.
All crates built into the library that use UniFFI will have bindings generated for them.

```
cargo build --release
cargo run --bin uniffi-bindgen generate --library target/release/libarithmetical.so --language kotlin --out-dir out
```

(try also `python` or `swift` for `--language`)

Then look in the `out` directory; there will be one file per crate (ie, one file in this example)

This must be run from within your Cargo workspace so it has access to Cargo metadata.

## Running uniffi-bindgen with a single UDL file

As above, not recommended and may be deprecated.

Use the `generate` command to generate bindings by specifying a UDL file.

```
cargo run --bin uniffi-bindgen generate src/arithmetic.udl --language kotlin
```

then have a look at `src/arithmetic.kts`

## Done

These commands can be integrated as part of your gradle/Xcode build process.

This is it, you have an MVP integration of UniFFI in your project.
