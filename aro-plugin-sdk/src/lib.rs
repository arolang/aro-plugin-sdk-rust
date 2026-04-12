//! # aro-plugin-sdk
//!
//! Rust SDK for building [ARO](https://github.com/arolang/aro) plugins.
//!
//! ## Quick start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! aro-plugin-sdk = "0.1"
//! serde_json = "1.0"
//!
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! Implement the four required C ABI exports:
//!
//! ```rust,no_run
//! use std::os::raw::c_char;
//! use aro_plugin_sdk::{ffi, Input, Output, PluginError, PluginResult};
//! use serde_json::json;
//!
//! #[no_mangle]
//! pub extern "C" fn aro_plugin_info() -> *mut c_char {
//!     ffi::to_c_string(json!({
//!         "name": "my-plugin",
//!         "version": "1.0.0",
//!         "handle": "MyPlugin",
//!         "actions": [
//!             {
//!                 "name": "greet",
//!                 "verbs": ["greet"],
//!                 "role": "own",
//!                 "prepositions": ["from"],
//!                 "description": "Greet a person"
//!             }
//!         ]
//!     }).to_string())
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn aro_plugin_execute(
//!     action: *const c_char,
//!     input_json: *const c_char,
//! ) -> *mut c_char {
//!     ffi::wrap_execute(action, input_json, |action, input| match action {
//!         "greet" => {
//!             let name = input.string("name").unwrap_or("World");
//!             Ok(Output::new().set("greeting", json!(format!("Hello, {name}!"))))
//!         }
//!         _ => Err(PluginError::internal(format!("Unknown action: {action}"))),
//!     })
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn aro_plugin_qualifier(
//!     qualifier: *const c_char,
//!     input_json: *const c_char,
//! ) -> *mut c_char {
//!     ffi::wrap_qualifier(qualifier, input_json, |_, _| {
//!         Err(PluginError::internal("No qualifiers registered"))
//!     })
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn aro_plugin_free(ptr: *mut c_char) {
//!     ffi::free_c_string(ptr);
//! }
//! ```

pub mod error;
pub mod event;
pub mod ffi;
pub mod input;
pub mod output;
pub mod qualifier;
pub mod testing;

// Re-export macros from the companion crate
pub use aro_plugin_sdk_macros::{action, aro_plugin, init, on_event, qualifier as qualifier_macro, shutdown, system_object};

// Flat re-exports for convenience
pub use error::{PluginError, PluginErrorCode, PluginResult};
pub use event::EventData;
pub use input::Input;
pub use output::Output;
pub use qualifier::Params;

/// The SDK prelude — import everything commonly needed in plugin code.
///
/// ```rust
/// use aro_plugin_sdk::prelude::*;
/// ```
pub mod prelude {
    pub use crate::error::{PluginError, PluginErrorCode, PluginResult};
    pub use crate::event::EventData;
    pub use crate::ffi::{free_c_string, to_c_string, wrap_event, wrap_execute, wrap_qualifier};
    pub use crate::input::Input;
    pub use crate::output::Output;
    pub use crate::qualifier::Params;
    pub use serde_json::{json, Value};
}
