/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::CodeType;
use crate::{bail, interface::Literal, Result};

#[derive(Debug)]
pub struct EnumCodeType {
    id: String,
}

impl EnumCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for EnumCodeType {
    fn type_label(&self) -> String {
        super::SwiftCodeOracle.class_name(&self.id)
    }

    fn canonical_name(&self) -> String {
        format!("Type{}", self.id)
    }

    fn literal(&self, literal: &Literal) -> Result<String> {
        if let Literal::Enum(v, _) = literal {
            Ok(format!(".{}", super::SwiftCodeOracle.enum_variant_name(v)))
        } else {
            bail!("Invalid literal for enum type: {literal:?}")
        }
    }
}
