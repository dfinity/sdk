use std::{borrow::Cow, time::Duration};

use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, Schema, SchemaObject, StringValidation},
    JsonSchema,
};

pub fn network_to_pathcompat(network_name: &str) -> String {
    network_name.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}

pub fn expiry_duration() -> Duration {
    // 5 minutes is max ingress timeout
    // 4 minutes accounts for possible replica drift
    Duration::from_secs(60 * 4)
}

pub struct ByteSchema;

impl JsonSchema for ByteSchema {
    fn schema_name() -> String {
        "Byte".to_string()
    }
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("byte_unit::Byte")
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(vec![InstanceType::Integer, InstanceType::String].into()),
            number: None,
            string: Some(Box::new(StringValidation {
                pattern: Some("^[0-9]+( *([KkMmGgTtPpEeZzYy]i?)?[Bb])?$".to_string()),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                title: Some("Byte Count".to_string()),
                description: Some("A quantity of bytes. Representable either as an integer, or as an SI unit string".to_string()),
                examples: vec![72.into(), "2KB".into(), "4 MiB".into()],
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
