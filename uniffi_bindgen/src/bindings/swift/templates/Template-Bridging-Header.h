// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

#pragma once

#include <stdbool.h>
#include <stdint.h>

typedef struct RustBuffer
{
    int32_t capacity;
    int32_t len;
    uint8_t *_Nullable data;
    // Ref https://github.com/mozilla/uniffi-rs/issues/334 for this weird "padding" field.
    int64_t padding;
} RustBuffer;

typedef struct ForeignBytes
{
    int32_t len;
    const uint8_t *_Nullable data;
    // Ref https://github.com/mozilla/uniffi-rs/issues/334 for these weird "padding" fields.
    int64_t padding;
    int32_t padding2;
} ForeignBytes;

// Error definitions
// Each error has an error code enum, and a struct
typedef struct NativeRustError {
    int32_t code;
    char *_Nullable message;
} NativeRustError;

  
{% for func in ci.iter_ffi_function_definitions() -%}
    {%- match func.return_type() -%}{%- when Some with (type_) %}{{ type_|type_ffi }}{% when None %}void{% endmatch %} {{ func.name() }}(
      {% call swift::arg_list_ffi_decl(func) %}
    );
{% endfor -%}

{% import "macros.swift" as swift %}
