/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    env,
    collections::HashMap,
    convert::TryFrom, convert::TryInto,
    fs::File,
    iter::IntoIterator,
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::bail;
use anyhow::Result;
use askama::Template;

use crate::interface::*;

// Some config options for it the caller wants to customize the generated Kotlin.
// Note that this can only be used to control details of the Kotlin *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.
pub struct Config {
    pub package_name: String
}

impl Config {
    pub fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: format!("uniffi.{}", ci.namespace())
        }
    }
}

#[derive(Template)]
#[template(ext="kt", escape="none", source=r#"
// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

package {{ config.package_name }};

// Common helper code.
//
// Ideally this would live in a separate .kt file where it can be unittested etc
// in isolation, and perhaps even published as a re-useable package.
//
// However, it's important that the detils of how this helper code works (e.g. the
// way that different builtin types are passed across the FFI) exactly match what's
// expected by the rust code on the other side of the interface. In practice right
// now that means coming from the exact some version of `uniffi` that was used to
// compile the rust component. The easiest way to ensure this is to bundle the Kotlin
// helpers directly inline like we're doing here.

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.atomic.AtomicLong

// This is how we find and load the dynamic library provided by the component.
// For now we just look it up by name.
//
// XXX TODO: This will probably grow some magic for resolving megazording in future.
// E.g. we might start by looking for the named component in `libuniffi.so` and if
// that fails, fall back to loading it separately from `lib${componentName}.so`.

inline fun <reified Lib : Library> loadIndirect(
    componentName: String
): Lib {
    return Native.load<Lib>("uniffi_${componentName}", Lib::class.java)
}

// This is a helper for safely working with byte buffers returned from the Rust code.
// It's basically a wrapper around a length and a data pointer, corresponding to the
// `ffi_support::ByteBuffer` struct on the rust side.
//
// It's lightly modified from the version we use in application-services.

@Structure.FieldOrder("len", "data")
open class RustBuffer : Structure() {
    @JvmField var len: Long = 0
    @JvmField var data: Pointer? = null

    class ByValue : RustBuffer(), Structure.ByValue

    companion object {
        internal fun alloc(size: Int): RustBuffer.ByValue {
            return _UniFFILib.INSTANCE.{{ ci.ffi_bytebuffer_alloc().name() }}(size)
        }

        internal fun free(buf: RustBuffer.ByValue) {
            return _UniFFILib.INSTANCE.{{ ci.ffi_bytebuffer_free().name() }}(buf)
        }
    }

    @Suppress("TooGenericExceptionThrown")
    fun asByteBuffer(): ByteBuffer? {
        return this.data?.let {
            val buf = it.getByteBuffer(0, this.len)
            buf.order(ByteOrder.BIG_ENDIAN)
            return buf
        }
    }
}


// Helpers for lifting primitive data types from a bytebuffer.

fun<T> liftFromRustBuffer(rbuf: RustBuffer.ByValue, liftFrom: (ByteBuffer) -> T): T {
    val buf = rbuf.asByteBuffer()!!
    try {
       val item = liftFrom(buf)
       if (buf.hasRemaining()) {
           throw RuntimeException("junk remaining in buffer after lifting, something is very wrong!!")
       }
       return item
    } finally {
        RustBuffer.free(rbuf)
    }
}

fun Boolean.Companion.lift(v: Byte): Boolean {
    return v.toInt() != 0
}

fun Boolean.Companion.liftFrom(buf: ByteBuffer): Boolean {
    return Boolean.lift(buf.get())
}

fun Byte.Companion.lift(v: Byte): Byte {
    return v
}

fun Byte.Companion.liftFrom(buf: ByteBuffer): Byte {
    return buf.get()
}

fun Int.Companion.lift(v: Int): Int {
    return v
}

fun Int.Companion.liftFrom(buf: ByteBuffer): Int {
    return buf.getInt()
}


fun Long.Companion.lift(v: Long): Long {
    return v
}

fun Long.Companion.liftFrom(buf: ByteBuffer): Long {
    return buf.getLong()
}

fun Float.Companion.lift(v: Float): Float {
    return v
}

fun Float.Companion.liftFrom(buf: ByteBuffer): Float {
    return buf.getFloat()
}

fun Double.Companion.lift(v: Double): Double {
    return v
}

fun Double.Companion.liftFrom(buf: ByteBuffer): Double {
    val v = buf.getDouble()
    return v
}

// I can't figure out how to make a generic implementation of (Any?).liftFrom, and IIUC there are some
// restrictions on generics in Kotlin (inherited from the JVM) that make it impossible to write in the
// style I want here. So, we use a standalone helper.

fun<T> liftOptional(rbuf: RustBuffer.ByValue, liftFrom: (ByteBuffer) -> T): T? {
    return liftFromRustBuffer(rbuf) { buf -> liftFromOptional(buf, liftFrom) }
}

fun<T> liftFromOptional(buf: ByteBuffer, liftFrom: (ByteBuffer) -> T): T? {
    if (! Boolean.liftFrom(buf)) {
        return null
    }
    return liftFrom(buf)
}

// Helpers for lowering primitive data types into a bytebuffer.
// Since we need to allocate buffers from rust, the lowering process needs to be
// able to calculate ahead-of-time what the required size of the buffer will be.

fun<T> lowerIntoRustBuffer(v: T, lowersIntoSize: (T) -> Int, lowerInto: (T, ByteBuffer) -> Unit): RustBuffer.ByValue {
    val buf = RustBuffer.alloc(lowersIntoSize(v))
    try {
        lowerInto(v, buf.asByteBuffer()!!)
        return buf
    } catch (e: Throwable) {
        RustBuffer.free(buf)
        throw e
    }
}

fun Int.lower(): Int {
    return this
}

fun Int.lowersIntoSize(): Int {
    return 4
}

fun Int.lowerInto(buf: ByteBuffer) {
    buf.putInt(this)
}

fun Long.lower(): Long {
    return this
}

fun Long.lowersIntoSize(): Long {
    return 4
}

fun Long.lowerInto(buf: ByteBuffer) {
    buf.putLong(this)
}

fun Float.lower(): Float {
    return this
}

fun Float.lowersIntoSize(): Int {
    return 4
}

fun Float.lowerInto(buf: ByteBuffer) {
    buf.putFloat(this)
}

fun Double.lower(): Double {
    return this
}

fun Double.lowersIntoSize(): Int {
    return 8
}

fun Double.lowerInto(buf: ByteBuffer) {
    buf.putDouble(this)
}

fun<T> lowerOptional(v: T?, lowersIntoSize: (T) -> Int, lowerInto: (T, ByteBuffer) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v, { v -> lowersIntoSizeOptional(v, lowersIntoSize) }, { v, buf -> lowerIntoOptional(v, buf, lowerInto) })
}

fun <T> lowersIntoSizeOptional(v: T?, lowersIntoSize: (T) -> Int): Int {
    if (v === null) return 1
    return 1 + lowersIntoSize(v)
}

fun<T> lowerIntoOptional(v: T?, buf: ByteBuffer, lowerInto: (T, ByteBuffer) -> Unit) {
    if (v === null) {
        buf.put(0)
    } else {
        buf.put(1)
        lowerInto(v, buf)
    }
}

// A JNA Library to expose the extern-C FFI definitions.
// This is an implementation detail which will be called internally by the public API.

internal interface _UniFFILib : Library {
    companion object {
        internal var INSTANCE: _UniFFILib = loadIndirect(componentName = "{{ ci.namespace() }}")
    }

    {% for func in ci.iter_ffi_function_definitions() -%}
        fun {{ func.name() }}(
        {%- for arg in func.arguments() %}
            {{ arg.name() }}: {{ arg.type_()|decl_c }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
        // TODO: When we implement error handling, there will be an out error param here.
        ): {%- match func.return_type() -%}{%- when Some with (type_) %}{{ type_|decl_c }}{% when None %}Unit{% endmatch %}
    {% endfor -%}
}

// Public interface members begin here.

{% for e in ci.iter_enum_definitions() %}
    enum class {{ e.name() }} {
        {% for value in e.values() %}
        {{ value }}{% if loop.last %};{% else %},{% endif %}
        {% endfor %}

        companion object {
            internal fun lift(n: Int): {{ e.name() }} {
                return when (n) {
                  {% for value in e.values() %}
                  {{ loop.index }} -> {{ value }}
                  {% endfor %}
                  else -> {
                      throw RuntimeException("invalid enum value, something is very wrong!!")
                  }
                }
            }

            internal fun liftFrom(buf: ByteBuffer): {{ e.name() }} {
                return {{ e.name() }}.lift(Int.liftFrom(buf))
            }
        }

        internal fun lower(): Int {
            return this.ordinal
        }

        internal fun lowersIntoSize(): Int {
            return 4
        }

        internal fun lowerInto(buf: ByteBuffer) {
            this.ordinal.lowerInto(buf)
        }
    }
{%- endfor -%}

{%- for rec in ci.iter_record_definitions() %}
    data class {{ rec.name() }} (
      {%- for field in rec.fields() %}
        val {{ field.name() }}: {{ field.type_()|decl_kt }}{% if loop.last %}{% else %},{% endif %}
      {%- endfor %}
    ) {
      companion object {
          // XXX TODO: put this in a superclass maybe?
          internal fun lift(rbuf: RustBuffer.ByValue): {{ rec.name() }} {
              return liftFromRustBuffer(rbuf) { buf -> {{ rec.name() }}.liftFrom(buf) }
          }

          internal fun liftFrom(buf: ByteBuffer): {{ rec.name() }} {
              return {{ rec.name() }}(
                {%- for field in rec.fields() %}
                {{ "buf"|lift_from_kt(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
                {%- endfor %}
              )
          }
      }

      internal fun lower(): RustBuffer.ByValue {
          return lowerIntoRustBuffer(this, {v -> v.lowersIntoSize()}, {v, buf -> v.lowerInto(buf)})
      }

      internal fun lowersIntoSize(): Int {
          return 0 +
            {%- for field in rec.fields() %}
            {{ "(this.{})"|format(field.name())|lowers_into_size_kt(field.type_()) }}{% if loop.last %}{% else %} +{% endif %}
            {%- endfor %}
      }

      internal fun lowerInto(buf: ByteBuffer) {
          {%- for field in rec.fields() %}
          {{ "(this.{})"|format(field.name())|lower_into_kt("buf", field.type_()) }}
          {%- endfor %}
      }
    }

{% endfor %}

{% for func in ci.iter_function_definitions() %}

    {%- match func.return_type() -%}
    {%- when Some with (return_type) %}

        fun {{ func.name() }}(
            {%- for arg in func.arguments() %}
                {{ arg.name() }}: {{ arg.type_()|decl_kt }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        ): {{ return_type|decl_kt }} {
            val _retval = _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
                {%- for arg in func.arguments() %}
                {{ arg.name()|lower_kt(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
                {%- endfor %}
            )
            return {{ "_retval"|lift_kt(return_type) }}
        }

    {% when None -%}

        fun {{ func.name() }}(
            {%- for arg in func.arguments() %}
                {{ arg.name() }}: {{ arg.type_()|decl_kt }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        ) {
            UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
                {%- for arg in func.arguments() %}
                {{ arg.name()|lower_kt(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
                {%- endfor %}
            )
        }

    {%- endmatch %}
{% endfor %}

{% for obj in ci.iter_object_definitions() %}
class {{ obj.name() }}(handle: Long) {
    private var handle: AtomicLong = AtomicLong(handle)
    {%- for cons in obj.constructors() %}
    constructor({% for arg in cons.arguments() %}{{ arg.name() }}: {{ arg.type_()|decl_kt }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}) :
        this(
            _UniFFILib.INSTANCE.{{ cons.ffi_func().name() }}(
                {%- for arg in cons.arguments() %}
                {{ arg.name()|lower_kt(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
                {%- endfor %}
            )
        )
    {%- endfor %}

    // XXX TODO: destructors or equivalent.

    {%- for meth in obj.methods() %}
    fun {{ meth.name() }}(
        {% for arg in meth.arguments() %}
        {{ arg.name() }}: {{ arg.type_()|decl_kt }}{% if loop.last %}{% else %}, {% endif %}
        {% endfor %}
    ): {% match meth.return_type() %}{% when Some with (type_) %}{{ type_|decl_kt }}{% when None %}Unit{% endmatch %} {
        val _retval = _UniFFILib.INSTANCE.{{ meth.ffi_func().name() }}(
            this.handle.get(){% if meth.arguments().len() > 0 %},{% endif %}
            {%- for arg in meth.arguments() %}
            {{ arg.name()|lower_kt(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
        {% match meth.return_type() %}{% when Some with (return_type) %}return {{ "_retval"|lift_kt(return_type) }}{% else %}{% endmatch %}
    }
    {%- endfor %}
}
{% endfor %}
"#)]
pub struct KotlinWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
}
impl<'a> KotlinWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        Self { config, ci }
    }
}

mod filters {
    use std::fmt;
    use super::*;

    pub fn decl_kt(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native Kotlin types map nicely to the FFI without conversion.
            TypeReference::U32 => "Int".to_string(),
            TypeReference::U64 => "Long".to_string(),
            TypeReference::Float => "Float".to_string(),
            TypeReference::Double => "Double".to_string(),
            TypeReference::Bytes => "RustBuffer.ByValue".to_string(),
            // These types need conversation, and special handling for lifting/lowering.
            TypeReference::Boolean => "Boolean".to_string(),
            TypeReference::Enum(name) => name.clone(),
            TypeReference::Record(name) => name.clone(),
            TypeReference::Optional(t) => format!("{}?", decl_kt(t)?),
            _ => panic!("[TODO: decl_kt({:?})]", type_),
        })
    }

    pub fn decl_c(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::Boolean => "Byte".to_string(),
            TypeReference::Enum(_) => "Int".to_string(),
            TypeReference::Record(_) => "RustBuffer.ByValue".to_string(),
            TypeReference::Optional(_) => "RustBuffer.ByValue".to_string(),
            TypeReference::Object(_) => "Long".to_string(),
            _ => decl_kt(type_)?,
        })
    }

    pub fn lower_kt(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(_) => {
                format!("(lowerOptional({}, {{ v -> {} }}, {{ (v, buf) -> {} }})", nm, lowers_into_size_kt(&"v", type_)?, lower_into_kt(&"v", &"buf", type_)?)
            },
            _ => format!("({} as {}).lower()", nm, decl_kt(type_)?),
        })
    }

    pub fn lower_into_kt(nm: &dyn fmt::Display, target: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(_) => {
                format!("(lowerIntoOptional({}, {}, {{ (v, buf) -> {} }})", nm, target, lower_into_kt(&"v", &"buf", type_)?)
            },
            _ => format!("({} as {}).lowerInto({})", nm, decl_kt(type_)?, target),
        })
    }

    pub fn lowers_into_size_kt(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(_) => {
                format!("(lowersIntoSizeOptional({}, {{ v -> {} }})", nm, lowers_into_size_kt(&"v", type_)?)
            },
            _ => format!("({} as {}).lowersIntoSize()", nm, decl_kt(type_)?),
        })
    }

    pub fn lift_kt( nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(t) => {
                format!("liftOptional({}, {{ buf -> {} }})", nm, lift_from_kt(&"buf", t)?)
            },
            _ => format!("{}.lift({})", decl_kt(type_)?, nm),
        })
    }

    pub fn lift_from_kt(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(t) => {
                format!("liftFromOptional({}, {{ buf -> {} }})", nm, lift_from_kt(&"buf", t)?)
            },
            _ => format!("{}.liftFrom({})", decl_kt(type_)?, nm),
        })
    }
}