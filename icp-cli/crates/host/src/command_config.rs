// src/config.rs

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub parameters: HashMap<String, Parameter>,
    pub nodes: HashMap<String, Node>,
}

#[derive(Debug, Deserialize)]
pub struct Parameter {
    pub r#type: String, // e.g., "string", "int", "path"
    #[serde(default)]
    pub positional: bool,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub r#type: String, // e.g., "print", "sign", "verify"
    #[serde(default)]
    pub inputs: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_simple_inputs() {
        let yaml = r#"
parameters:
  message:
    type: string
    positional: true

nodes:
  printer:
    type: print
    inputs:
      value: message  # Simple reference
"#;

        // Use a cursor to simulate reading from a file
        let config: Config =
            serde_yaml::from_reader(Cursor::new(yaml)).expect("unable to parse yaml");

        // Test parameter loading
        assert_eq!(config.parameters.len(), 1);
        let message_param = config
            .parameters
            .get("message")
            .expect("message parameter missing");
        assert_eq!(message_param.r#type, "string");
        assert!(message_param.positional);

        // Test nodes loading
        assert_eq!(config.nodes.len(), 1);

        let printer_node = config.nodes.get("printer").expect("printer node missing");
        assert_eq!(printer_node.r#type, "print");
        let input = printer_node
            .inputs
            .get("value")
            .expect("value input missing");
        assert_eq!(input, "message");
    }

    #[test]
    fn test_qualified_inputs() {
        let yaml = r#"
parameters:
  message:
    type: string
    positional: true

nodes:
  signer:
    type: sign
    inputs:
      data: message.value  # Qualified reference (message.value is an output)
"#;

        // Use a cursor to simulate reading from a file
        let config: Config =
            serde_yaml::from_reader(Cursor::new(yaml)).expect("unable to parse yaml");

        // Test parameter loading
        assert_eq!(config.parameters.len(), 1);
        let message_param = config
            .parameters
            .get("message")
            .expect("message parameter missing");
        assert_eq!(message_param.r#type, "string");
        assert!(message_param.positional);

        // Test nodes loading
        assert_eq!(config.nodes.len(), 1);

        let signer_node = config.nodes.get("signer").expect("signer node missing");
        assert_eq!(signer_node.r#type, "sign");
        let input = signer_node.inputs.get("data").expect("data input missing");
        assert_eq!(input, "message.value");
    }
}
