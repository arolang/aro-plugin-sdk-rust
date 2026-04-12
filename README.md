# aro-plugin-sdk (Rust)

Rust SDK for building [ARO](https://github.com/arolang/aro) plugins.

## Workspace layout

```
aro-plugin-sdk-rust/
├── Cargo.toml                  # workspace root
├── aro-plugin-sdk/             # main library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # re-exports and prelude
│       ├── input.rs            # Input — typed JSON accessor
│       ├── output.rs           # Output — response builder
│       ├── error.rs            # PluginError, PluginErrorCode
│       ├── ffi.rs              # C-string helpers, wrap_execute/qualifier/event
│       ├── event.rs            # EventData struct
│       ├── qualifier.rs        # Params struct
│       └── testing.rs          # mock_input helper
└── aro-plugin-sdk-macros/      # proc-macro crate
    ├── Cargo.toml
    └── src/
        └── lib.rs              # #[aro_plugin], #[action], #[qualifier], …
```

## Quick start

**`Cargo.toml`**
```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
aro-plugin-sdk = { path = "../aro-plugin-sdk" }   # or version = "0.1" from crates.io
serde_json = "1.0"
```

**`src/lib.rs`**
```rust
use std::os::raw::c_char;
use aro_plugin_sdk::prelude::*;

#[no_mangle]
pub extern "C" fn aro_plugin_info() -> *mut c_char {
    to_c_string(json!({
        "name": "my-plugin",
        "version": "1.0.0",
        "handle": "MyPlugin",
        "actions": [
            {
                "name": "greet",
                "verbs": ["greet"],
                "role": "own",
                "prepositions": ["from"],
                "description": "Greet a person by name"
            }
        ]
    }).to_string())
}

#[no_mangle]
pub extern "C" fn aro_plugin_execute(
    action: *const c_char,
    input_json: *const c_char,
) -> *mut c_char {
    wrap_execute(action, input_json, |action, input| match action {
        "greet" => {
            let name = input.string("name").unwrap_or("World");
            Ok(Output::new().set("greeting", json!(format!("Hello, {name}!"))))
        }
        _ => Err(PluginError::internal(format!("Unknown action: {action}"))),
    })
}

#[no_mangle]
pub extern "C" fn aro_plugin_qualifier(
    qualifier: *const c_char,
    input_json: *const c_char,
) -> *mut c_char {
    wrap_qualifier(qualifier, input_json, |qualifier, input| {
        match qualifier {
            "shout" => {
                let value = input.string("value").unwrap_or("").to_uppercase();
                Ok(Output::value(json!(value)))
            }
            _ => Err(PluginError::internal(format!("Unknown qualifier: {qualifier}"))),
        }
    })
}

#[no_mangle]
pub extern "C" fn aro_plugin_free(ptr: *mut c_char) {
    free_c_string(ptr);
}
```

## Key types

### `Input`

Typed accessor for the JSON payload the ARO runtime sends:

```rust
// Direct field lookup (top-level keys take precedence over _with)
input.string("name")       // Option<&str>
input.int("count")         // Option<i64>
input.float("price")       // Option<f64>
input.bool("enabled")      // Option<bool>
input.array("items")       // Option<&Vec<Value>>
input.get("key")           // Option<&Value>
input.raw()                // &Value

// With-clause parameters
let params = input.with_params();    // Params
params.string("order")               // Option<&str>
params.string_or("order", "asc")     // &str

// Descriptor accessors
input.result_identifier()  // Option<&str>
input.result_qualifier()   // Option<&str>
input.source_identifier()  // Option<&str>
input.preposition()        // Option<&str>
input.context()            // Option<&Value>
```

### `Output`

Fluent builder for the JSON response:

```rust
Output::new()
    .set("result", json!("value"))
    .set("count", json!(42))
    .emit("UserCreated", json!({ "id": 1 }))
    .to_json_string()
```

### `PluginError` and `PluginErrorCode`

```rust
PluginError::missing("data")                         // code 1
PluginError::invalid_type("count", "an integer")     // code 2
PluginError::not_found("user/42")                    // code 7
PluginError::internal("unexpected state")            // code 10
PluginError::new(PluginErrorCode::Timeout, "…")      // custom
```

## Proc macros (stub — code generation is planned)

The `aro-plugin-sdk-macros` crate provides attribute macros that will
eventually auto-generate boilerplate.  They are **currently pass-through
stubs** and do not modify the annotated item.

| Macro | Planned purpose |
|-------|-----------------|
| `#[aro_plugin]` | Generate `aro_plugin_info` and the dispatch table |
| `#[action]` | Register an action handler |
| `#[qualifier]` | Register a qualifier handler |
| `#[system_object]` | Register a system object |
| `#[on_event]` | Register an event handler |
| `#[init]` | Register the plugin init hook |
| `#[shutdown]` | Register the plugin shutdown hook |

## Testing

```rust
#[cfg(test)]
mod tests {
    use aro_plugin_sdk::testing::{json, mock_input};

    #[test]
    fn test_greet() {
        let input = mock_input(json!({ "name": "Alice" }));
        assert_eq!(input.string("name"), Some("Alice"));
    }
}
```

## Building

```bash
cargo build --release
# Output: target/release/libmy_plugin.dylib (macOS) or .so (Linux)
```

## License

MIT
