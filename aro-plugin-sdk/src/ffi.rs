//! C ABI helpers for the ARO plugin interface.
//!
//! Every ARO native plugin (C, C++, Rust) must export these four symbols:
//!
//! | Symbol | Purpose |
//! |--------|---------|
//! | `aro_plugin_info` | Return plugin metadata as a JSON C string |
//! | `aro_plugin_execute` | Dispatch an action, return JSON C string |
//! | `aro_plugin_qualifier` | Transform a value via a qualifier |
//! | `aro_plugin_free` | Release a C string allocated by the plugin |
//!
//! This module provides helpers that make it easy to implement those
//! symbols without boilerplate:
//!
//! - [`to_c_string`] / [`free_c_string`] for memory management
//! - [`wrap_execute`] for building an `aro_plugin_execute` body
//! - [`wrap_qualifier`] for building an `aro_plugin_qualifier` body
//! - [`wrap_event`] for building an `aro_plugin_event` body (optional)

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde_json::{json, Value};

use crate::error::{PluginError, PluginErrorCode, PluginResult};
use crate::input::Input;
use crate::output::Output;

// ---------------------------------------------------------------------------
// Raw C-string helpers
// ---------------------------------------------------------------------------

/// Convert a Rust `String` into a heap-allocated, NUL-terminated C string.
///
/// The caller (i.e. the ARO runtime) **must** release the memory by calling
/// `aro_plugin_free` on the returned pointer.
///
/// Returns `null` if the string contains interior NUL bytes (should not
/// happen for valid JSON).
pub fn to_c_string(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Reclaim memory allocated by [`to_c_string`].
///
/// This is the implementation of `aro_plugin_free`.  Call it like:
///
/// ```ignore
/// #[no_mangle]
/// pub extern "C" fn aro_plugin_free(ptr: *mut c_char) {
///     aro_plugin_sdk::ffi::free_c_string(ptr);
/// }
/// ```
pub fn free_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Read a `*const c_char` into a Rust `&str`, returning an error JSON
/// pointer on failure.
///
/// # Safety
/// `ptr` must be a valid, NUL-terminated C string for the duration of the
/// call.
pub unsafe fn read_c_str<'a>(ptr: *const c_char, field: &str) -> Result<&'a str, *mut c_char> {
    if ptr.is_null() {
        return Err(error_json(&format!("{field} pointer is null")));
    }
    CStr::from_ptr(ptr).to_str().map_err(|_| error_json(&format!("{field} is not valid UTF-8")))
}

// ---------------------------------------------------------------------------
// Dispatch wrappers
// ---------------------------------------------------------------------------

/// Dispatch wrapper for `aro_plugin_execute`.
///
/// Parses the input JSON string, calls `handler(action, input)`, and
/// serialises the result (or error) back to a C string.
///
/// # Example
/// ```ignore
/// #[no_mangle]
/// pub extern "C" fn aro_plugin_execute(
///     action: *const c_char,
///     input_json: *const c_char,
/// ) -> *mut c_char {
///     aro_plugin_sdk::ffi::wrap_execute(action, input_json, |action, input| {
///         match action {
///             "my-action" => handle_my_action(input),
///             _ => Err(PluginError::new(PluginErrorCode::Unknown,
///                      format!("Unknown action: {action}"))),
///         }
///     })
/// }
/// ```
pub fn wrap_execute<F>(
    action_ptr: *const c_char,
    input_ptr: *const c_char,
    handler: F,
) -> *mut c_char
where
    F: FnOnce(&str, Input) -> PluginResult<Output>,
{
    // Safety: ARO runtime guarantees valid C strings
    let action = unsafe {
        match read_c_str(action_ptr, "action") {
            Ok(s) => s,
            Err(ptr) => return ptr,
        }
    };

    let input_str = unsafe {
        match read_c_str(input_ptr, "input_json") {
            Ok(s) => s,
            Err(ptr) => return ptr,
        }
    };

    let input = match Input::from_str(input_str) {
        Ok(i) => i,
        Err(e) => return to_c_string(error_string(&format!("Invalid JSON input: {e}"))),
    };

    match handler(action, input) {
        Ok(output) => to_c_string(output.to_json_string()),
        Err(e) => to_c_string(error_string(&e.to_string())),
    }
}

/// Dispatch wrapper for `aro_plugin_qualifier`.
///
/// Similar to [`wrap_execute`] but the handler signature reflects that
/// qualifiers receive `(qualifier_name, input)` and return an `Output`.
pub fn wrap_qualifier<F>(
    qualifier_ptr: *const c_char,
    input_ptr: *const c_char,
    handler: F,
) -> *mut c_char
where
    F: FnOnce(&str, Input) -> PluginResult<Output>,
{
    wrap_execute(qualifier_ptr, input_ptr, handler)
}

/// Dispatch wrapper for an optional `aro_plugin_event` export.
///
/// Event handlers follow the same input/output pattern as actions, but
/// are invoked asynchronously by the runtime's event bus.
pub fn wrap_event<F>(
    event_ptr: *const c_char,
    input_ptr: *const c_char,
    handler: F,
) -> *mut c_char
where
    F: FnOnce(&str, Input) -> PluginResult<Output>,
{
    wrap_execute(event_ptr, input_ptr, handler)
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn error_string(message: &str) -> String {
    json!({ "error": message }).to_string()
}

fn error_json(message: &str) -> *mut c_char {
    to_c_string(error_string(message))
}

/// Build a JSON error response value (useful in `aro_plugin_info` bodies).
pub fn make_error_value(code: PluginErrorCode, message: &str) -> Value {
    json!({
        "error": message,
        "code": code as u8,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::ffi::CString;

    fn make_c_str(s: &str) -> *const c_char {
        CString::new(s).unwrap().into_raw()
    }

    #[test]
    fn test_wrap_execute_ok() {
        let action = make_c_str("ping");
        let input = make_c_str(r#"{"data":"hello"}"#);

        let result_ptr = wrap_execute(action, input, |action, inp| {
            assert_eq!(action, "ping");
            assert_eq!(inp.string("data"), Some("hello"));
            Ok(Output::new().set("pong", json!(true)))
        });

        assert!(!result_ptr.is_null());
        let result = unsafe { CString::from_raw(result_ptr) };
        let v: Value = serde_json::from_str(result.to_str().unwrap()).unwrap();
        assert_eq!(v["pong"], true);
    }

    #[test]
    fn test_wrap_execute_error() {
        let action = make_c_str("fail");
        let input = make_c_str("{}");

        let result_ptr = wrap_execute(action, input, |_, _| {
            Err(PluginError::missing("data"))
        });

        assert!(!result_ptr.is_null());
        let result = unsafe { CString::from_raw(result_ptr) };
        let v: Value = serde_json::from_str(result.to_str().unwrap()).unwrap();
        assert!(v.get("error").is_some());
    }
}
