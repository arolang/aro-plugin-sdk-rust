//! Event data passed to `#[on_event]` handlers.

use serde_json::Value;

/// Payload delivered to an event handler registered with `#[on_event]`.
///
/// The ARO runtime serialises the event as JSON and passes it through the
/// C ABI.  This struct gives typed access to the common envelope fields.
#[derive(Debug, Clone)]
pub struct EventData {
    /// The event name, e.g. `"UserCreated"`.
    pub name: String,
    /// The raw JSON payload of the event.
    pub payload: Value,
}

impl EventData {
    /// Construct an `EventData` from its component parts.
    pub fn new(name: impl Into<String>, payload: Value) -> Self {
        Self { name: name.into(), payload }
    }

    /// Parse an `EventData` from the JSON envelope the runtime sends:
    ///
    /// ```json
    /// { "event": "UserCreated", "payload": { ... } }
    /// ```
    pub fn from_json(json: &Value) -> Option<Self> {
        let name = json.get("event")?.as_str()?.to_owned();
        let payload = json.get("payload").cloned().unwrap_or(Value::Null);
        Some(Self { name, payload })
    }

    /// Return a field from the payload by key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.payload.get(key)
    }

    /// Return a string field from the payload.
    pub fn string(&self, key: &str) -> Option<&str> {
        self.payload.get(key)?.as_str()
    }

    /// Return an integer field from the payload.
    pub fn int(&self, key: &str) -> Option<i64> {
        self.payload.get(key)?.as_i64()
    }

    /// Return a boolean field from the payload.
    pub fn bool(&self, key: &str) -> Option<bool> {
        self.payload.get(key)?.as_bool()
    }
}
