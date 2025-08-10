/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use camino::Utf8Path;
use fs_err as fs;
use std::any::Any;
use std::collections::HashMap;

pub mod filters;
mod pipeline;
pub use pipeline::pipeline;

#[cfg(feature = "bindgen-tests")]
pub mod test;

pub fn run_pipeline(initial_root: pipeline::initial::Root, out_dir: &Utf8Path) -> Result<()> {
    let python_root = pipeline().execute(initial_root)?;
    println!("writing out {out_dir}");
    if !out_dir.exists() {
        fs::create_dir_all(out_dir)?;
    }
    for namespace in python_root.namespaces.values() {
        //let mut values: BTreeMap<&str, Box<dyn Any>> = BTreeMap::new();
        //values.insert("string_ffi_converter_name", Box::new(module.string_type_node.ffi_converter_name.clone()));
        // let values: (&str, &dyn Any) = ("string_ffi_converter_name", &module.string_type_node.ffi_converter_name);
        let mut values: HashMap<&str, Box<dyn Any>> = HashMap::new();
        //values.insert("string_ffi_converter_name", Box::new("wtf_ffi_conv"));

        let mut content = namespace.render_with_values(&values)?;
        for t in &namespace.type_definitions {
            content += &t.render_with_values(&values)?;
        }
        // types, functions, string
        let path = out_dir.join(format!("{}.py", namespace.name));
        println!("writing {path}");
        fs::write(path, content)?;
    }
    Ok(())
}
