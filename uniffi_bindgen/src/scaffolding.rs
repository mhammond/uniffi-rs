/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;

use super::interface::*;

#[derive(Template)]
#[template(syntax = "rs", escape = "none", path = "scaffolding_template.rs")]
pub struct RustScaffolding<'a> {
    ci: &'a ComponentInterface,
    uniffi_version: &'static str,
}
impl<'a> RustScaffolding<'a> {
    pub fn new(ci: &'a ComponentInterface) -> Self {
        Self {
            ci,
            uniffi_version: crate::BINDGEN_VERSION,
        }
    }
}

// A hack while we still support non-threadsafe structs. If the type specifies
// a `Type::Object()` but it's not declared as threadsafe, we wrap it in a
// Mutex.
pub fn make_threadsafe(type_: &Type, already_threadsafe: &bool) -> Type {
    if *already_threadsafe {
        // nothing to do.
        type_.clone()
    } else {
        // not declared threadsafe, so wrap objects in a mutex.
        match type_ {
            Type::Object(name) => Type::Object(format!("std::sync::Mutex<{}>", name)),
            _ => type_.clone(),
        }
    }
}

mod filters {
    use super::*;
    use std::fmt;

    #[allow(dead_code)] // we should kill this if a need for it doesn't come back.
    pub fn choose(
        expr_value: &bool,
        then_value: &dyn fmt::Display,
        else_value: &dyn fmt::Display,
    ) -> Result<String, askama::Error> {
        Ok(if *expr_value {
            then_value.to_string()
        } else {
            else_value.to_string()
        })
    }

    pub fn type_rs(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8 => "i8".into(),
            Type::UInt8 => "u8".into(),
            Type::Int16 => "i16".into(),
            Type::UInt16 => "u16".into(),
            Type::Int32 => "i32".into(),
            Type::UInt32 => "u32".into(),
            Type::Int64 => "i64".into(),
            Type::UInt64 => "u64".into(),
            Type::Float32 => "f32".into(),
            Type::Float64 => "f64".into(),
            Type::Boolean => "bool".into(),
            Type::String => "String".into(),
            Type::Enum(name) | Type::Record(name) | Type::Error(name) => name.clone(),
            Type::Object(name) => format!("std::sync::Arc<{}>", name),
            Type::CallbackInterface(name) => format!("Box<dyn {}>", name),
            Type::Optional(t) => format!("Option<{}>", type_rs(t)?),
            Type::Sequence(t) => format!("Vec<{}>", type_rs(t)?),
            Type::Map(t) => format!("std::collections::HashMap<String, {}>", type_rs(t)?),
        })
    }

    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(match type_ {
            FFIType::Int8 => "i8".into(),
            FFIType::UInt8 => "u8".into(),
            FFIType::Int16 => "i16".into(),
            FFIType::UInt16 => "u16".into(),
            FFIType::Int32 => "i32".into(),
            FFIType::UInt32 => "u32".into(),
            FFIType::Int64 => "i64".into(),
            FFIType::UInt64 => "u64".into(),
            FFIType::Float32 => "f32".into(),
            FFIType::Float64 => "f64".into(),
            FFIType::RustCString => "*mut std::os::raw::c_char".into(),
            FFIType::RustArcPtr => "*const std::os::raw::c_void".into(),
            FFIType::RustBuffer => "uniffi::RustBuffer".into(),
            FFIType::RustError => "uniffi::deps::ffi_support::ExternError".into(),
            FFIType::ForeignBytes => "uniffi::ForeignBytes".into(),
            FFIType::ForeignCallback => "uniffi::ForeignCallback".into(),
        })
    }

    pub fn lower_rs(
        nm: &dyn fmt::Display,
        type_: &Type,
        threadsafe: &bool,
    ) -> Result<String, askama::Error> {
        // By explicitly naming the type here, we help the rust compiler to type-check the user-provided
        // implementations of the functions that we're wrapping (and also to type-check our generated code).
        Ok(match type_ {
            Type::CallbackInterface(type_name) => unimplemented!(
                "uniffi::ViaFfi::lower is not supported for callback interfaces ({})",
                type_name
            ),
            // This special case exists because `IntoFfi` is not implemented for
            // `*const std::os::raw::c_void`. If we fix that, this variant
            // can be removed and the default one below will work.
            Type::Object(_) => format!(
                "uniffi::UniffiVoidPtr(<{} as uniffi::ViaFfi>::lower({}))",
                type_rs(&make_threadsafe(type_, threadsafe))?,
                nm
            ),
            _ => format!(
                "<{} as uniffi::ViaFfi>::lower({})",
                type_rs(&make_threadsafe(type_, threadsafe))?,
                nm
            ),
        })
    }

    pub fn lift_rs(
        nm: &dyn fmt::Display,
        type_: &Type,
        threadsafe: &bool,
    ) -> Result<String, askama::Error> {
        // By explicitly naming the type here, we help the rust compiler to type-check the user-provided
        // implementations of the functions that we're wrapping (and also to type-check our generated code).
        // This will panic if the bindings provide an invalid value over the FFI.
        Ok(match type_ {
            Type::CallbackInterface(type_name) => format!(
                "Box::new(<{}Proxy as uniffi::ViaFfi>::try_lift({}).unwrap())",
                type_name, nm,
            ),
            _ => format!(
                "<{} as uniffi::ViaFfi>::try_lift({}).unwrap()",
                type_rs(&make_threadsafe(type_, threadsafe))?,
                nm
            ),
        })
    }

    /// Get a Rust expression for writing a value into a byte buffer.
    pub fn write_rs(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::CallbackInterface(type_name) => unimplemented!(
                "uniffi::ViaFfi::write is not supported for callback interfaces ({})",
                type_name
            ),
            _ => format!(
                "<{} as uniffi::ViaFfi>::write(&{}, {})",
                type_rs(type_)?,
                nm,
                target
            ),
        })
    }

    /// Get a Rust expression for writing a value into a byte buffer.
    pub fn read_rs(target: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::CallbackInterface(type_name) => unimplemented!(
                "uniffi::ViaFfi::try_read is not supported for callback interfaces ({})",
                type_name
            ),
            _ => format!(
                "<{} as uniffi::ViaFfi>::try_read({}).unwrap()",
                type_rs(type_)?,
                target
            ),
        })
    }
}
