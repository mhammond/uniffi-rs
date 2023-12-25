/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

// A struct exposed to foreign bindings as an Error interface.
#[derive(Debug, thiserror::Error)]
#[error("{e:?}")]
pub struct ErrorInterface {
    e: anyhow::Error,
}

impl ErrorInterface {
    fn chain(&self) -> Vec<String> {
        self.e.chain().map(ToString::to_string).collect()
    }
    fn link(&self, ndx: u64) -> Option<String> {
        self.e.chain().nth(ndx as usize).map(ToString::to_string)
    }
}

// A conversion into our ErrorInterface from anyhow::Error.
impl From<anyhow::Error> for ErrorInterface {
    fn from(e: anyhow::Error) -> Self {
        Self { e }
    }
}

// Defined in UDL as throwing EnumError.
fn simple() -> Result<(), EnumError> {
    Err(EnumError::Oops)
}

// Defined in UDL as throwing ErrorInterface.
fn oops() -> anyhow::Result<()> {
    anyhow::bail!("oops");
}

// Procmacros need to be told the type.
#[uniffi::export(E = Arc<ErrorInterface>)]
// Explicit result used here but `anyhow::Result<()>` is fine too.
fn poops() -> Result<(), anyhow::Error> {
    Err(anyhow::Error::msg("poops").context("via a procmacro"))
}

#[cfg(feature = "async")] // async broken too.
#[uniffi::export]
async fn asimple() -> Result<(), EnumError> {
    Err(EnumError::Oops)
}

#[cfg(feature = "async")] // async broken too.
#[uniffi::export(E = Arc<ErrorInterface>)]
pub async fn apoops() -> anyhow::Result<()> {
    unreachable!()
}

// The error interface can still be used as a regular interface.
fn get_error(message: String) -> Arc<ErrorInterface> {
    std::sync::Arc::new(ErrorInterface {
        e: anyhow::Error::msg(message),
    })
}

// Exercise constructors and methods.
#[derive(uniffi::Object, Debug)]
pub struct ErrorThrower {}

#[uniffi::export(E = Arc<ErrorInterface>)]
impl ErrorThrower {
    #[uniffi::constructor]
    fn new(ok: bool) -> anyhow::Result<Arc<Self>> {
        if ok {
            Ok(Arc::new(Self {}))
        } else {
            anyhow::bail!("oops")
        }
    }

    fn throw(&self) -> anyhow::Result<()> {
        anyhow::bail!("threw");
    }
}

#[cfg(feature = "trait-result")]
mod trait_result {
    use super::*;

    // doesn't work - broken as we try and implement the trait for callbacks
    // which doesn't make sense in this model.
    #[uniffi::export(E = Arc<ErrorInterface>)]
    trait TraitThrower: Send + Sync {
        fn throw(&self) -> anyhow::Result<()> {
            anyhow::bail!("threw");
        }
    }
}

#[cfg(feature = "trait-impl")]
mod trait_impl {
    // foreign traits might use another trait to represent error values.
    #[uniffi::export]
    pub trait ErrorTrait: Send + Sync + std::error::Error {
        fn message(&self) -> String;
    }

    // Then, as above, we need:
    // #[uniffi::export(E = Arc<dyn ErrorTrait>)] ???
    // trait {
    //  ... normal anyhow::Result<> impl
    //  }
}

// Enum errors.
#[derive(Debug, thiserror::Error)]
pub enum EnumError {
    #[error("oops")]
    Oops,
}

// Any module can work like `anyhow` above. Eg, `EnumError` can be
// specifically for the foreign error interface, but internally
// the Rust code uses a different Error:
// (XXX - the code below is *not* demonstrating that - to demonstrate that
// we'd need a function *outside* the module to return `enums::Result<>` AND
// the error to be an interface...)
mod enums {
    use super::*;
    pub enum Error {
        BadThing1,
    }
    pub type Result<T> = std::result::Result<T, Error>;

    // public functions returning the local error type via `?`
    #[uniffi::export(E = EnumError)]
    fn poopse() -> Result<()> {
        Err(Error::BadThing1)
    }

    // Convert the local errors to public errors.
    impl From<Error> for EnumError {
        fn from(_: Error) -> Self {
            super::EnumError::Oops
        }
    }
}

uniffi::include_scaffolding!("error_types");
