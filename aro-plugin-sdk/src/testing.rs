//! Test helpers for ARO plugin unit tests.

pub use serde_json::json;

use serde_json::Value;

use crate::input::Input;

/// Build an [`Input`] from a `serde_json::json!` literal.
///
/// This is the primary test helper.  Pass a JSON object literal for the
/// full input payload (top-level fields plus optional `_with`).
///
/// # Example
/// ```rust
/// use aro_plugin_sdk::testing::mock_input;
///
/// let input = mock_input(serde_json::json!({
///     "data": "hello",
///     "_with": { "flag": true }
/// }));
///
/// assert_eq!(input.string("data"), Some("hello"));
/// assert_eq!(input.with_params().bool("flag"), Some(true));
/// ```
pub fn mock_input(value: Value) -> Input {
    Input::new(value)
}

/// Build an [`Input`] that simulates an HTTP request context.
///
/// `path_params` is merged into `context.pathParameters` and `body` is
/// placed in `context.body`.
pub fn mock_http_input(path_params: Value, body: Value, extra: Value) -> Input {
    let mut obj = extra
        .as_object()
        .cloned()
        .unwrap_or_default();

    obj.insert(
        "context".to_owned(),
        serde_json::json!({
            "pathParameters": path_params,
            "body": body,
        }),
    );

    Input::new(Value::Object(obj))
}
