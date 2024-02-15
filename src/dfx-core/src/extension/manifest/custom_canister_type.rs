use crate::error::extension::{ProcessCanisterDeclarationError};
use crate::extension::manager::ExtensionManager;
use crate::extension::manifest::ExtensionManifest;
use handlebars::Handlebars;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::BTreeMap;
use crate::error::extension::ProcessCanisterDeclarationError::{CustomCanisterTypeTemplateError, ExtensionDoesNotSupportAnyCustomCanisterTypes, ExtensionDoesNotSupportSpecificCustomCanisterType};
use crate::error::load_dfx_config::TransformConfigurationError;

pub trait TransformConfiguration {
    fn transform(&mut self, json: &mut serde_json::Value) -> Result<(), TransformConfigurationError>;
}

impl TransformConfiguration for ExtensionManager {
    fn transform(&mut self, json: &mut JsonValue) -> Result<(), TransformConfigurationError> {
        let canisters = match json.get_mut("canisters").and_then(|v| v.as_object_mut()) {
            Some(canisters) => canisters,
            None => return Ok(()),
        };
        for (canister_name, canister_declaration) in canisters.iter_mut() {
            if let Some(canister_type) = get_valid_canister_type(canister_declaration) {
                let canister_declaration = canister_declaration.as_object_mut().unwrap();
                let (extension_name, canister_type) =
                    get_extension_name_and_custom_canister_type(&canister_type);
                let extension_manifest =
                    ExtensionManifest::get_by_extension_name(extension_name, self)?;
                *canister_declaration = process_canister_declaration(
                    canister_declaration,
                    extension_name,
                    &extension_manifest,
                    canister_name,
                    canister_type,
                )?;
            }
        }
        Ok(())
    }
}

fn get_valid_canister_type(canister_declaration: &mut JsonValue) -> Option<String> {
    canister_declaration
        .get("type")
        .and_then(|v| v.as_str())
        .and_then(|s| {
            if !["rust", "motoko", "custom", "assets", "pull"].contains(&s) {
                Some(s.to_owned())
            } else {
                None
            }
        })
}

/// Split the canister type on ':', returning `extension_name` and `canister_type`
/// If there's no ':', `canister_type` is the same as `extension_name`
pub(super) fn get_extension_name_and_custom_canister_type(canister_type: &str) -> (&str, &str) {
    if let Some(i) = canister_type.find(':') {
        (&canister_type[..i], &canister_type[i + 1..])
    } else {
        (canister_type, canister_type)
    }
}

pub(super) fn process_canister_declaration(
    canister_declaration: &mut JsonMap<String, JsonValue>,
    extension_name: &str,
    extension_manifest: &ExtensionManifest,
    canister_name: &str,
    canister_type: &str,
) -> Result<JsonMap<String, JsonValue>, ProcessCanisterDeclarationError> {
    let extension_manifest_canister_type = extension_manifest.canister_types.as_ref().ok_or(
        ExtensionDoesNotSupportAnyCustomCanisterTypes(extension_name.into()),
    )?;
    let extension_manifest_canister_type = extension_manifest_canister_type.get(canister_type);

    let custom_canister_declaration = match extension_manifest_canister_type {
        Some(val) => val,
        None => {
            return Err(
                ExtensionDoesNotSupportSpecificCustomCanisterType(
                    canister_type.into(),
                    extension_name.into(),
                ),
            );
        }
    };
    let mut values: BTreeMap<String, JsonValue> = canister_declaration
        .into_iter()
        .filter_map(|(k, v)| {
            if v.is_array() || v.is_object() {
                None
            } else {
                Some((k.clone(), v.clone()))
            }
        })
        .collect();
    values.insert("canister_name".into(), canister_name.into());

    custom_canister_declaration
        .apply_template(values)
        .map_err(|e| {
            CustomCanisterTypeTemplateError(
                extension_name.to_string(),
                canister_type.to_string(),
                e.to_string(),
            )
        })
}

type FieldName = String;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CustomCanisterTypeDeclaration(BTreeMap<FieldName, Op>);

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Op {
    BoolValue(bool),
    NumberValue(serde_json::Number),
    Remove { remove: bool },
    Replace { replace: Replace },
    Template(String),
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Replace {
    input: String,
    search: String,
    output: String,
}

#[derive(Debug)]
enum OpError {
    InvalidTemplate(handlebars::RenderError),
    InvalidReplace(regex::Error),
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpError::InvalidTemplate(e) => write!(f, "Invalid template: {}", e),
            OpError::InvalidReplace(e) => write!(f, "Invalid regex: {}", e),
        }
    }
}

impl CustomCanisterTypeDeclaration {
    fn apply_template(
        &self,
        values: BTreeMap<FieldName, JsonValue>,
    ) -> Result<JsonMap<FieldName, JsonValue>, OpError> {
        let mut remove_fields = vec![];
        let mut final_fields = JsonMap::new();
        for (field_name, op) in self
            .0
            .clone()
            .into_iter()
            .collect::<Vec<_>>()
            .clone()
            .into_iter()
        {
            match op {
                Op::BoolValue(x) => {
                    final_fields.insert(field_name, x.into());
                }
                Op::NumberValue(x) => {
                    final_fields.insert(field_name, x.into());
                }
                Op::Replace { replace } => {
                    let input = Handlebars::new()
                        .render_template(&replace.input, &values)
                        .map_err(OpError::InvalidTemplate)?;
                    let re = Regex::new(&replace.search).map_err(OpError::InvalidReplace)?;
                    let x = re.replace_all(&input, &replace.output).to_string();
                    final_fields.insert(field_name, x.into());
                }
                Op::Remove { remove } if remove => {
                    remove_fields.push(field_name);
                }
                Op::Template(template) => {
                    let x = Handlebars::new()
                        .render_template(&template, &values)
                        .map_err(OpError::InvalidTemplate)?;
                    final_fields.insert(field_name, x.into());
                }
                _ => {}
            }
        }

        // Removing fields should be done last because of the order of the fields in the map.
        // It's easier to do in second for loop than to sort Ops beforehand, bacause Op would need to implement PartialOrd,
        // which is not possible, because serde_json::Number does not implement it.
        for field_name in remove_fields {
            final_fields.remove(&field_name);
        }

        // Override custom canister declaration values by the real canister_declaration
        // see: https://github.com/dfinity/sdk/pull/3222#issuecomment-1624073606
        let skip_keys = ["type", "canister_name"].map(String::from);
        for (key, value) in values.iter().filter(|(k, _)| !skip_keys.contains(k)) {
            final_fields.insert(key.clone(), value.clone());
        }
        Ok(final_fields)
    }
}

#[cfg(test)]
pub struct NoopTransformConfiguration;
#[cfg(test)]
impl TransformConfiguration for NoopTransformConfiguration {
    fn transform(&mut self, _: &mut serde_json::Value) -> Result<(), TransformConfigurationError> {
        // Do nothing
        Ok(())
    }
}

#[cfg(test)]
mod custom_canister_type_declaration_tests {
    use super::*;

    macro_rules! test_op {
    (
        custom_canister_template = $custom_canister_template:expr,
        dfx_json_canister_values = $dfx_json_canister:expr,
        expected = $expected:expr
    ) => {
        let custom_canister_template = serde_json::from_str::<CustomCanisterTypeDeclaration>($custom_canister_template).unwrap();
        // dfx_json_canister_values is a transformed version of canister declaration from dfx.json.
        // Below is the example of the transformation; FROM:
        // "frontend_canister": {
        //    "type": "custom"
        // }
        // transformed INTO:
        // let values: BTreeMap<String, Value::String> = [
        //   ("canister_name".into(), Value::String("frontend_canister".into()))
        //   ("type".into(),          Value::String("custom".into()))
        // ].into();
        let dfx_json_canister_values = serde_json::from_str($dfx_json_canister).unwrap();
        let expected = serde_json::from_str($expected).unwrap();
        assert_eq!(
            custom_canister_template
                .apply_template(dfx_json_canister_values)
                .unwrap(),
            expected
        );
    };}

    #[test]
    fn test_op_replace_1() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "something.py"
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "gzip": true
        }
        "#,
            expected = r#"
        {
            "gzip": true,
            "main": "something.py"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_2() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "something.py",
            "gzip": false
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "gzip": true
        }
        "#,
            expected = r#"
        {
            "gzip": true,
            "main": "something.py"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_3() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "{{gzip}}.py",
            "gzip": false
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "gzip": true
        }
        "#,
            expected = r#"
        {
            "gzip": true,
            "main": "true.py"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_4() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "path/to/{{main}}"
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "main": "something.py"
        }
        "#,
            expected = r#"
        {
            "main": "path/to/something.py"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_5() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": { "replace": { "input": "{{main}}", "search": ".*/(.*).ts", "output": "thecanister/$1.exe" } }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "main": "src/main.ts"
        }
        "#,
            expected = r#"
        {
            "main": "thecanister/main.exe"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_6() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": { "replace": { "input": "{{canister_name}}", "search": "frontend_(.*)", "output": "thecanister/$1/main.ts" } }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "canister_name": "frontend_xyz"
        }
        "#,
            expected = r#"
        {
            "main": "thecanister/xyz/main.ts"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_and_delete_1() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "something.py",
            "gzip": { "remove": true }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "gzip": true
        }
        "#,
            expected = r#"
        {
            "main": "something.py"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_and_delete_2() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "something.py",
            "gzip": false,
            "gzip": { "remove": true }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "gzip": true
        }
        "#,
            expected = r#"
        {
            "gzip": false,
            "main": "something.py"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_and_delete_3() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "{{gzip}}.py",
            "gzip": false,
            "gzip": { "remove": true }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "gzip": true
        }
        "#,
            expected = r#"
        {
            "gzip": false,
            "main": "true.py"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_and_delete_4() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "path/to/{{main}}",
            "main": { "remove": true }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "main": "something.py"
        }
        "#,
            expected = r#"
        {
            "main": "path/to/something.py"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_and_delete_5() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": { "replace": { "input": "{{main}}", "search": ".*/(.*).ts", "output": "thecanister/$1.exe" } },
            "main": { "remove": true }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "main": "src/main.ts"
        }
        "#,
            expected = r#"
        {
            "main": "thecanister/main.exe"
        }
        "#
        );
    }

    #[test]
    fn test_op_replace_and_delete_6() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": { "replace": { "input": "{{canister_name}}", "search": "frontend_(.*)", "output": "thecanister/$1/main.ts" } }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "canister_name": "frontend_xyz"
        }
        "#,
            expected = r#"
        {
            "main": "thecanister/xyz/main.ts"
        }
        "#
        );
    }

    #[test]
    fn test_op_remove() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": "src/main.ts",
            "main": { "remove": true }
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "main": "thecanister.exe"
        }
        "#,
            expected = r#"
        {
            "main": "thecanister.exe"
        }
        "#
        );
    }

    #[test]
    fn test_op_template() {
        test_op!(
            custom_canister_template = r#"
        {
            "type": "{{canister_name}}"
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "canister_name": "something"
        }
        "#,
            expected = r#"
        {
            "type": "something"
        }
        "#
        );
    }

    #[test]
    fn test_op_bool() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": true
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "type": "my_bool_extension",
            "canister_name": "something"
        }
        "#,
            expected = r#"
        {
            "main": true
        }
        "#
        );
    }

    #[test]
    fn test_op_number() {
        test_op!(
            custom_canister_template = r#"
        {
            "main": 3
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "type": "my_number_extension",
            "canister_name": "something"
        }
        "#,
            expected = r#"
        {
            "main": 3
        }
        "#
        );
    }

    #[test]
    fn test_overwrite() {
        test_op!(
            custom_canister_template = r#"
        {
            "gzip": true
        }
        "#,
            dfx_json_canister_values = r#"
        {
            "gzip": false
        }
        "#,
            expected = r#"
        {
            "gzip": false
        }
        "#
        );
    }

    #[test]
    fn test_ops() {
        test_op!(
            custom_canister_template = r#"
        {
            "type": "custom",
            "main": "src/main.ts",
            "ts": { "replace": { "input": "{{main}}", "search": "(.*).ts", "output": "$1.ts" }},
            "wasm": ".azyl/{{canister_name}}/{{canister_name}}.wasm.gz",
            "build": "npx azyl {{canister_name}}",
            "candid": { "replace": { "input": "{{main}}", "search": "(.*).ts", "output": "$1.did" }},
            "main": { "remove": true },
            "gzip": true
        }"#,
            dfx_json_canister_values = r#"
        {
            "canister_name": "azyl_frontend",
            "main": "src/main.ts",
            "gzip": false
        }"#,
            expected = r#"
        {
            "build": "npx azyl azyl_frontend",
            "candid": "src/main.did",
            "gzip": false,
            "main": "src/main.ts",
            "ts": "src/main.ts",
            "type": "custom",
            "wasm": ".azyl/azyl_frontend/azyl_frontend.wasm.gz"
        }
        "#
        );
    }
}
