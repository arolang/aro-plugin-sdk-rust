//! Typed wrapper around the JSON payload the ARO runtime passes to a plugin.
//!
//! The runtime serialises the current execution context as a JSON object with
//! these top-level keys:
//!
//! | Key | Description |
//! |-----|-------------|
//! | `result` | Result descriptor (identifier, qualifier) |
//! | `object` | Object descriptor (identifier, qualifier) |
//! | `preposition` | Preposition keyword used in the statement |
//! | `context` | Ambient context values (request, path params, …) |
//! | `_with` | Values supplied by the `with`-clause |
//! | *everything else* | Direct binding values |

use serde_json::Value;

use crate::qualifier::Params;

/// Typed accessor for the JSON input passed to a plugin action or qualifier.
///
/// # Example
/// ```rust
/// # use aro_plugin_sdk::Input;
/// # use serde_json::json;
/// let raw = json!({ "data": "hello", "count": 3, "_with": { "flag": true } });
/// let input = Input::new(raw);
/// assert_eq!(input.string("data"), Some("hello"));
/// assert_eq!(input.int("count"), Some(3));
/// assert_eq!(input.with_params().bool("flag"), Some(true));
/// ```
#[derive(Debug, Clone)]
pub struct Input {
    raw: Value,
}

impl Input {
    /// Wrap a raw JSON value.
    pub fn new(raw: Value) -> Self {
        Self { raw }
    }

    /// Parse from a JSON string.
    pub fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        let raw: Value = serde_json::from_str(s)?;
        Ok(Self { raw })
    }

    // -----------------------------------------------------------------------
    // Direct field accessors
    // -----------------------------------------------------------------------

    /// Return the raw `Value` for `key`, checking top-level first then `_with`.
    pub fn get(&self, key: &str) -> Option<&Value> {
        // Top-level keys take precedence over _with
        if let Some(v) = self.raw.get(key) {
            return Some(v);
        }
        self.raw.get("_with")?.get(key)
    }

    /// Return a `&str` for `key`.
    pub fn string(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    /// Return an `i64` for `key`.
    pub fn int(&self, key: &str) -> Option<i64> {
        self.get(key)?.as_i64()
    }

    /// Return an `f64` for `key`.
    pub fn float(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_f64()
    }

    /// Return a `bool` for `key`.
    pub fn bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    /// Return an array for `key`.
    pub fn array(&self, key: &str) -> Option<&Vec<Value>> {
        self.get(key)?.as_array()
    }

    /// Return the entire raw JSON value.
    pub fn raw(&self) -> &Value {
        &self.raw
    }

    // -----------------------------------------------------------------------
    // With-clause
    // -----------------------------------------------------------------------

    /// Return the `_with` object as a [`Params`].
    pub fn with_params(&self) -> Params {
        self.raw
            .get("_with")
            .and_then(|v| Params::from_value(v))
            .unwrap_or_default()
    }

    // -----------------------------------------------------------------------
    // Descriptor accessors
    // -----------------------------------------------------------------------

    /// Return the result descriptor object: `{ "identifier": "…", "qualifier": "…" }`.
    pub fn result_descriptor(&self) -> Option<&Value> {
        self.raw.get("result")
    }

    /// Return the result identifier (the variable name the ARO runtime will bind).
    pub fn result_identifier(&self) -> Option<&str> {
        self.result_descriptor()?.get("identifier")?.as_str()
    }

    /// Return the result qualifier, if any.
    pub fn result_qualifier(&self) -> Option<&str> {
        self.result_descriptor()?.get("qualifier")?.as_str()
    }

    /// Return the source/object descriptor: `{ "identifier": "…", "qualifier": "…" }`.
    pub fn source_descriptor(&self) -> Option<&Value> {
        self.raw.get("object")
    }

    /// Return the source identifier.
    pub fn source_identifier(&self) -> Option<&str> {
        self.source_descriptor()?.get("identifier")?.as_str()
    }

    /// Return the source qualifier, if any.
    pub fn source_qualifier(&self) -> Option<&str> {
        self.source_descriptor()?.get("qualifier")?.as_str()
    }

    /// Return the preposition keyword used in the statement, e.g. `"from"` or `"with"`.
    pub fn preposition(&self) -> Option<&str> {
        self.raw.get("preposition")?.as_str()
    }

    /// Return the ambient execution context object.
    ///
    /// This contains runtime-injected values such as the HTTP request,
    /// path parameters, etc.
    pub fn context(&self) -> Option<&Value> {
        self.raw.get("context")
    }

    /// Convenience: return a field from inside the context object.
    pub fn context_get(&self, key: &str) -> Option<&Value> {
        self.context()?.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_accessor() {
        let input = Input::new(json!({ "name": "Alice" }));
        assert_eq!(input.string("name"), Some("Alice"));
    }

    #[test]
    fn test_with_fallback() {
        let input = Input::new(json!({ "_with": { "flag": true } }));
        assert_eq!(input.bool("flag"), Some(true));
    }

    #[test]
    fn test_top_level_wins_over_with() {
        let input = Input::new(json!({ "x": 1, "_with": { "x": 99 } }));
        assert_eq!(input.int("x"), Some(1));
    }

    #[test]
    fn test_result_descriptor() {
        let input = Input::new(json!({
            "result": { "identifier": "output", "qualifier": "uppercase" }
        }));
        assert_eq!(input.result_identifier(), Some("output"));
        assert_eq!(input.result_qualifier(), Some("uppercase"));
    }

    #[test]
    fn test_missing_key_is_none() {
        let input = Input::new(json!({}));
        assert!(input.string("missing").is_none());
    }
}
