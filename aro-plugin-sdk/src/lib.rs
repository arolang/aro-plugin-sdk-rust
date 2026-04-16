//! # aro-plugin-sdk
//!
//! Rust SDK for building [ARO](https://github.com/arolang/aro) plugins.
//!
//! ## Quick start
//!
//! ```toml
//! [dependencies]
//! aro-plugin-sdk = { git = "https://github.com/arolang/aro-plugin-sdk-rust.git", branch = "main" }
//! serde_json = "1.0"
//!
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! ```rust,ignore
//! use aro_plugin_sdk::prelude::*;
//!
//! #[action(name = "Greet", verbs = ["greet"], role = "own",
//!          prepositions = ["with"], description = "Greet someone")]
//! fn greet(input: &Input) -> PluginResult<Output> {
//!     let name = input.string("name").unwrap_or("World");
//!     Ok(Output::new().set("greeting", json!(format!("Hello, {name}!"))))
//! }
//!
//! aro_export! {
//!     name: "my-plugin",
//!     version: "1.0.0",
//!     handle: "My",
//!     actions: [greet],
//!     qualifiers: [],
//! }
//! ```

pub mod error;
pub mod event;
pub mod ffi;
pub mod input;
pub mod output;
pub mod qualifier;
pub mod testing;

// Re-export proc macros
pub use aro_plugin_sdk_macros::{
    action, aro_plugin, init, on_event, qualifier as qualifier_attr,
    shutdown, system_object, aro_export,
};

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
    pub use aro_plugin_sdk_macros::{action, aro_export, qualifier as qualifier_attr};
    pub use serde_json::{json, Value};
}
