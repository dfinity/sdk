use crate::output_promise::AnyOutputPromise;
use std::collections::HashMap;

pub struct NodeConfig {
    pub params: HashMap<String, String>,
    pub inputs: HashMap<String, AnyOutputPromise>,
    pub outputs: HashMap<String, AnyOutputPromise>,
}
