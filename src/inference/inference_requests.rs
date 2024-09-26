use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicInferenceRequest {
    pub data: HashMap<String, serde_json::Value>,
}

impl DynamicInferenceRequest {
    pub fn new(data: String) -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn add_param<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.params.insert(key.to_string(), json_value);
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub data: String,
}
