use crate::config::model::dfinity::BUILTIN_CANISTER_TYPES;
use crate::error::config::{
    AppendMetadataError, ApplyExtensionCanisterTypeDefaultsError, ApplyExtensionCanisterTypeError,
    ApplyExtensionCanisterTypesError, MergeTechStackError, RenderErrorWithContext,
};
use crate::extension::manager::ExtensionManager;
use crate::extension::manifest::{extension::ExtensionCanisterType, ExtensionManifest};
use handlebars::{Handlebars, RenderError};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub fn apply_extension_canister_types(
    mut json: Value,
    extension_manager: Option<&ExtensionManager>,
) -> Result<Value, ApplyExtensionCanisterTypesError> {
    let Some(extension_manager) = extension_manager else {
        return Ok(json);
    };
    let Some(canisters) = json.get_mut("canisters") else {
        return Ok(json);
    };

    let canisters: &mut Map<String, Value> = canisters
        .as_object_mut()
        .ok_or(ApplyExtensionCanisterTypesError::CanistersFieldIsNotAnObject())?;
    for (canister_name, v) in canisters.iter_mut() {
        let canister_json =
            v.as_object_mut()
                .ok_or(ApplyExtensionCanisterTypesError::CanisterIsNotAnObject(
                    canister_name.to_string(),
                ))?;
        apply_extension_canister_type(canister_name, canister_json, extension_manager)?
    }
    Ok(json)
}

fn apply_extension_canister_type(
    canister_name: &str,
    fields: &mut Map<String, Value>,
    extension_manager: &ExtensionManager,
) -> Result<(), ApplyExtensionCanisterTypeError> {
    let canister_type = fields
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("custom")
        .to_string();

    if BUILTIN_CANISTER_TYPES.contains(&canister_type.as_str()) {
        return Ok(());
    }

    let extension = canister_type;
    if !ExtensionManifest::exists(&extension, &extension_manager.dir) {
        return Err(
            ApplyExtensionCanisterTypeError::NoExtensionForUnknownCanisterType {
                canister: canister_name.to_string(),
                extension,
            },
        );
    }
    let manifest = ExtensionManifest::load(&extension, &extension_manager.dir)
        .map_err(ApplyExtensionCanisterTypeError::LoadExtensionManifest)?;
    let extension_canister_type = manifest.canister_type.ok_or(
        ApplyExtensionCanisterTypeError::ExtensionDoesNotDefineCanisterType {
            canister: canister_name.to_string(),
            extension: extension.clone(),
        },
    )?;

    apply_defaults(canister_name, fields, &extension_canister_type).map_err(|source| {
        ApplyExtensionCanisterTypeError::ApplyDefaults {
            canister: Box::new(canister_name.to_string()),
            extension: Box::new(extension),
            source,
        }
    })?;
    fields.insert("type".to_string(), Value::String("custom".to_string()));
    Ok(())
}

fn apply_defaults(
    canister_name: &str,
    canister_json: &mut Map<String, Value>,
    extension_canister_type: &ExtensionCanisterType,
) -> Result<(), ApplyExtensionCanisterTypeDefaultsError> {
    let evaluate_keys = keys_in_evaluation_order(extension_canister_type);

    let handlebars = Handlebars::new();

    for k in evaluate_keys {
        let data = build_render_data(canister_name, canister_json, extension_canister_type);
        let v = extension_canister_type.defaults.get(&k).unwrap();
        let v = recursive_render_templates(v.clone(), &handlebars, &data).map_err(|source| {
            ApplyExtensionCanisterTypeDefaultsError::Render(Box::new(RenderErrorWithContext {
                field: k.to_string(),
                value: v.to_string(),
                source,
            }))
        })?;

        let handled_metadata = k == "metadata" && append_metadata(canister_json, &v)?;
        let handled_tech_stack = k == "tech_stack" && merge_tech_stack(canister_json, &v)?;
        let handled = handled_metadata || handled_tech_stack;
        let already_in_dfx_json = canister_json.contains_key(&k);
        if !handled && !already_in_dfx_json {
            canister_json.insert(k.clone(), v);
        }
    }
    Ok(())
}

fn keys_in_evaluation_order(extension_canister_type: &ExtensionCanisterType) -> Vec<String> {
    let mut remaining_keys = extension_canister_type
        .defaults
        .keys()
        .cloned()
        .collect::<BTreeSet<String>>();
    let mut evaluate_keys = extension_canister_type.evaluation_order.clone();
    for k in evaluate_keys.iter() {
        remaining_keys.remove(k);
    }
    evaluate_keys.extend(remaining_keys);
    evaluate_keys
}

fn recursive_render_templates(
    v: Value,
    handlebars: &Handlebars,
    data: &BTreeMap<String, Value>,
) -> Result<Value, RenderError> {
    match v {
        Value::String(s) => {
            let s = handlebars.render_template(&s, data)?;
            Ok(Value::String(s))
        }
        Value::Array(arr) => {
            let arr = arr
                .into_iter()
                .map(|v| recursive_render_templates(v, handlebars, data))
                .collect::<Result<Vec<Value>, _>>()?;
            Ok(Value::Array(arr))
        }
        Value::Object(obj) => {
            let obj: Result<Map<String, Value>, RenderError> = obj
                .into_iter()
                .map(|(k, v)| {
                    let v: Value = recursive_render_templates(v, handlebars, data)?;
                    Ok((k, v))
                })
                .collect();
            Ok(Value::Object(obj?))
        }
        _ => Ok(v),
    }
}

fn build_render_data(
    canister_name: &str,
    canister_json: &Map<String, Value>,
    extension_canister_type: &ExtensionCanisterType,
) -> BTreeMap<String, Value> {
    let mut data = BTreeMap::new();
    data.insert(
        "canister_name".to_string(),
        Value::String(canister_name.to_string()),
    );
    let mut canister = Map::new();
    canister.extend(extension_canister_type.defaults.clone());
    canister.extend(canister_json.clone());
    data.insert("canister".to_string(), Value::Object(canister));
    data
}

fn append_metadata(
    canister_json: &mut Map<String, Value>,
    extension_metadata: &Value,
) -> Result<bool, AppendMetadataError> {
    let Some(metadata) = canister_json.get_mut("metadata") else {
        return Ok(false);
    };

    let metadata = metadata
        .as_array_mut()
        .ok_or(AppendMetadataError::ExpectedCanisterMetadataArray)?;
    let mut extension_metadata = extension_metadata
        .as_array()
        .ok_or(AppendMetadataError::ExpectedExtensionCanisterTypeMetadataArray)?
        .clone();
    metadata.append(&mut extension_metadata);
    Ok(true)
}

fn merge_tech_stack(
    canister_json: &mut Map<String, Value>,
    extension_tech_stack: &Value,
) -> Result<bool, MergeTechStackError> {
    let Some(canister_tech_stack) = canister_json.get_mut("tech_stack") else {
        return Ok(false);
    };

    merge_tech_stack_maps(canister_tech_stack, extension_tech_stack)?;

    Ok(true)
}

fn merge_tech_stack_maps(
    canister_tech_stack: &mut Value,
    extension_tech_stack: &Value,
) -> Result<(), MergeTechStackError> {
    let canister_tech_stack = canister_tech_stack
        .as_object_mut()
        .ok_or(MergeTechStackError::ExpectedCanisterTechStackObject)?;
    let extension_tech_stack = extension_tech_stack
        .as_object()
        .ok_or(MergeTechStackError::ExpectedExtensionCanisterTypeTechStackObject)?;
    for (k, v) in extension_tech_stack.iter() {
        if let Some(canister_value) = canister_tech_stack.get_mut(k) {
            if canister_value.is_object() {
                merge_tech_stack_maps(canister_value, v)?;
            }
        } else {
            canister_tech_stack.insert(k.clone(), v.clone());
        }
    }
    Ok(())
}
