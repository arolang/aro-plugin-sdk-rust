//! Output builder for ARO plugin responses.
//!
//! Every action and qualifier returns a JSON object to the ARO runtime.
//! `Output` provides a fluent builder for constructing that object and
//! serialising it to a JSON string.
//!
//! # Example
//! ```rust
//! # use aro_plugin_sdk::Output;
//! # use serde_json::json;
//! let json = Output::new()
//!     .set("result", json!("hello"))
//!     .set("count", json!(42))
//!     .emit("DataProcessed", json!({ "count": 42 }))
//!     .to_json_string();
//! ```

use serde_json::{Map, Value};

/// Builder for the JSON object returned by a plugin action or qualifier.
#[derive(Debug, Default)]
pub struct Output {
    fields: Map<String, Value>,
    events: Vec<Value>,
}

impl Output {
    /// Create an empty `Output`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience: create an `Output` with a single `"value"` field.
    pub fn value(v: impl Into<Value>) -> Self {
        Self::new().set("value", v.into())
    }

    /// Set a top-level field in the output object.
    pub fn set(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Queue an event to be emitted by the ARO runtime after this action
    /// completes.  The runtime reads the `_events` array in the response.
    ///
    /// # Arguments
    /// * `name` – event name, e.g. `"UserCreated"`
    /// * `payload` – JSON payload attached to the event
    pub fn emit(mut self, name: impl Into<String>, payload: impl Into<Value>) -> Self {
        self.events.push(serde_json::json!({
            "event": name.into(),
            "payload": payload.into(),
        }));
        self
    }

    /// Consume the builder and produce a `serde_json::Value`.
    pub fn to_value(mut self) -> Value {
        if !self.events.is_empty() {
            self.fields.insert("_events".to_owned(), Value::Array(self.events));
        }
        Value::Object(self.fields)
    }

    /// Consume the builder and serialise to a JSON string.
    ///
    /// # Panics
    /// Panics if serialisation fails (it should never fail for a well-formed
    /// `serde_json::Value`).
    pub fn to_json_string(self) -> String {
        self.to_value().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_set_fields() {
        let out = Output::new()
            .set("name", json!("Alice"))
            .set("age", json!(30));
        let v = out.to_value();
        assert_eq!(v["name"], "Alice");
        assert_eq!(v["age"], 30);
    }

    #[test]
    fn test_emit_event() {
        let out = Output::new()
            .set("ok", json!(true))
            .emit("UserCreated", json!({ "id": 1 }));
        let v = out.to_value();
        let events = v["_events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event"], "UserCreated");
    }

    #[test]
    fn test_no_events_no_key() {
        let out = Output::new().set("x", json!(1));
        let v = out.to_value();
        assert!(v.get("_events").is_none());
    }
}
