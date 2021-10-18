// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

@file:Suppress("NAME_SHADOWING")

package {{ config.package_name() }};

// Common helper code.
//
// Ideally this would live in a separate .kt file where it can be unittested etc
// in isolation, and perhaps even published as a re-useable package.
//
// However, it's important that the detils of how this helper code works (e.g. the
// way that different builtin types are passed across the FFI) exactly match what's
// expected by the Rust code on the other side of the interface. In practice right
// now that means coming from the exact some version of `uniffi` that was used to
// compile the Rust component. The easiest way to ensure this is to bundle the Kotlin
// helpers directly inline like we're doing here.

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import java.nio.ByteBuffer
import java.nio.ByteOrder

{%- for imported_class in self.imports() %}
import {{ imported_class }}
{%- endfor %}

// The Rust Buffer and 3 templated methods (alloc, free, reserve).
{% include "RustBufferTemplate.kt" %}

{% include "Helpers.kt" %}

// Contains loading, initialization code,
// and the FFI Function declarations in a com.sun.jna.Library.
{% include "NamespaceLibraryTemplate.kt" %}

// Public interface members begin here.
{% for code in self.declaration_code() %}
{{ code }}
{%- endfor %}

{% import "macros.kt" as kt %}