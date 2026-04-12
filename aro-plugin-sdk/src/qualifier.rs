//! Qualifier parameter types.
//!
//! Qualifiers are invoked via the `<value: Handle.qualifier>` syntax and
//! optionally receive a `with`-clause: `<value: Handle.qualifier with <params>>`.

use serde_json::Value;

/// Parameters supplied to a qualifier via the `with`-clause.
///
/// # Example (ARO source)
/// ```aro
/// Compute the <sorted: Stats.sort with <order: "desc">> from the <numbers>.
/// ```
///
/// The `Params` struct gives typed access to the key-value pairs in that
/// `with`-clause object.
#[derive(Debug, Clone, Default)]
pub struct Params {
    inner: serde_json::Map<String, Value>,
}

impl Params {
    /// Create a `Params` from a JSON object value.
    ///
    /// Returns `None` if `value` is not a JSON object.
    pub fn from_value(value: &Value) -> Option<Self> {
        value.as_object().map(|m| Self { inner: m.clone() })
    }

    /// Create an empty `Params`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Return the raw JSON value for `key`.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.inner.get(key)
    }

    /// Return a string parameter.
    pub fn string(&self, key: &str) -> Option<&str> {
        self.inner.get(key)?.as_str()
    }

    /// Return a string parameter, falling back to `default` if absent.
    pub fn string_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.string(key).unwrap_or(default)
    }

    /// Return an integer parameter.
    pub fn int(&self, key: &str) -> Option<i64> {
        self.inner.get(key)?.as_i64()
    }

    /// Return an integer parameter, falling back to `default` if absent.
    pub fn int_or(&self, key: &str, default: i64) -> i64 {
        self.int(key).unwrap_or(default)
    }

    /// Return a float parameter.
    pub fn float(&self, key: &str) -> Option<f64> {
        self.inner.get(key)?.as_f64()
    }

    /// Return a boolean parameter.
    pub fn bool(&self, key: &str) -> Option<bool> {
        self.inner.get(key)?.as_bool()
    }

    /// Return a boolean parameter, falling back to `default` if absent.
    pub fn bool_or(&self, key: &str, default: bool) -> bool {
        self.bool(key).unwrap_or(default)
    }

    /// Return an array parameter as a slice of `Value`s.
    pub fn array(&self, key: &str) -> Option<&Vec<Value>> {
        self.inner.get(key)?.as_array()
    }

    /// Check whether `key` is present in the params.
    pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }
}
