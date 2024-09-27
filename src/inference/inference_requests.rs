//! Inference requests module for the Module Validator application.
//!
//! This module defines structures for handling inference requests and responses.

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// Represents a dynamic inference request with arbitrary data.
#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicInferenceRequest {
    pub data: HashMap<String, serde_json::Value>,
}

impl DynamicInferenceRequest {
    /// Creates a new DynamicInferenceRequest instance.
    ///
    /// # Arguments
    ///
    /// * `data` - The initial data string (currently unused).
    ///
    /// # Returns
    ///
    /// A new DynamicInferenceRequest instance.
    pub fn new(data: String) -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Adds a parameter to the inference request.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the parameter.
    /// * `value` - The value of the parameter.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of adding the parameter.
    pub fn add_param<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.params.insert(key.to_string(), json_value);
        Ok(())
    }
}

/// Represents an inference request.
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub data: HashMap<String, serde_json::Value>,
}

/// Represents an inference response.
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub data: String,
}
