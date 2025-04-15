use crate::output_promise::OutputPromise;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NodeConfig {
    pub params: HashMap<String, String>,
    pub inputs: HashMap<String, Arc<OutputPromise>>,
}
