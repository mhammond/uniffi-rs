/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_meta::{MethodMetadata, ObjectImplMetadata};

use super::convert::{fn_param_metadata, return_type_metadata, type_as_type_path};
use crate::export::{ExportItem, Method};

pub(super) fn gen_impl_metadata(
    item: syn::ItemImpl,
    mod_path: &[String],
) -> syn::Result<ExportItem> {
    if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "generic impls are not currently supported by uniffi::export",
        ));
    }

    let type_path = type_as_type_path(&item.self_ty)?;

    if type_path.qself.is_some() {
        return Err(syn::Error::new_spanned(
            type_path,
            "qualified self types are not currently supported by uniffi::export",
        ));
    }

    let self_impl = match type_path.path.get_ident() {
        Some(id) => ObjectImplMetadata::Struct(id.to_string()), // XXXXXXX???
        None => {
            return Err(syn::Error::new_spanned(
                type_path,
                "qualified paths in self-types are not currently supported by uniffi::export",
            ));
        }
    };

    let methods = item
        .items
        .into_iter()
        .map(|it| gen_method_metadata(it, &self_impl, mod_path))
        .collect();

    Ok(ExportItem::Impl { methods, self_impl })
}

fn gen_method_metadata(
    it: syn::ImplItem,
    self_impl: &ObjectImplMetadata,
    mod_path: &[String],
) -> syn::Result<Method> {
    let item = match it {
        syn::ImplItem::Method(m) => m,
        _ => {
            return Err(syn::Error::new_spanned(
                it,
                "only methods are supported in impl blocks annotated with uniffi::export",
            ));
        }
    };

    let metadata = method_metadata(self_impl, &item, mod_path)?;

    Ok(Method { item, metadata })
}

fn method_metadata(
    self_impl: &ObjectImplMetadata,
    f: &syn::ImplItemMethod,
    mod_path: &[String],
) -> syn::Result<MethodMetadata> {
    Ok(MethodMetadata {
        module_path: mod_path.to_owned(),
        self_impl: self_impl.clone(),
        name: f.sig.ident.to_string(),
        inputs: fn_param_metadata(&f.sig.inputs)?,
        return_type: return_type_metadata(&f.sig.output)?,
    })
}
