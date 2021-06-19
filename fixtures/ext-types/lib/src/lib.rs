use ext_types_guid::{Guid, GuidHelper};
use ext_types_json::{JSONHelper, JSONObject};

include!(concat!(env!("OUT_DIR"), "/lib.uniffi.rs"));

struct LibHelper {
    guid_helper: GuidHelper,
    json_helper: JSONHelper,
}

fn get_lib_helper(vals: Option<LibHelper>) -> LibHelper {
    match vals {
        None => LibHelper {
            guid_helper: GuidHelper {
                guid: Guid("first-guid".to_string()),
                guids: vec![
                    Guid("second-guid".to_string()),
                    Guid("third-guid".to_string()),
                ],
            },
            json_helper: JSONHelper {
                json: JSONObject(serde_json::json!({"foo": "bar"})),
                jsons: vec![
                    JSONObject(serde_json::json!(["an", "array"])),
                    JSONObject(serde_json::json!(3)),
                ],
            },
        },
        Some(vals) => vals,
    }
}
