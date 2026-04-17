# ARO Plugin SDK for Rust

Build ARO plugins in Rust with proc macros that eliminate C ABI boilerplate. Annotate your functions with `#[action]` and `#[qualifier]`, then call `aro_export!` to generate all exports automatically.

## Installation

```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
aro-plugin-sdk = { git = "https://github.com/arolang/aro-plugin-sdk-rust.git", branch = "main" }
serde_json = "1.0"
```

## Quick Start

```rust
use aro_plugin_sdk::prelude::*;

#[action(name = "Greet", verbs = ["greet"], role = "own",
         prepositions = ["with"], description = "Greet someone by name")]
fn greet(input: &Input) -> PluginResult<Output> {
    let name = input.string("name").unwrap_or("World");
    Ok(Output::new().set("greeting", json!(format!("Hello, {name}!"))))
}

aro_export! {
    name: "my-plugin",
    version: "1.0.0",
    handle: "Greeting",
    actions: [greet],
    qualifiers: [],
}
```

The `aro_export!` macro generates `aro_plugin_info`, `aro_plugin_execute`, `aro_plugin_qualifier`, `aro_plugin_free`, `aro_plugin_init`, and `aro_plugin_shutdown` — all the C ABI exports the ARO runtime needs.

## Actions

Actions handle verbs in ARO statements. Annotate functions with `#[action]`:

```rust
#[action(name = "ParseCSV", verbs = ["parsecsv", "readcsv"], role = "own",
         prepositions = ["from", "with"],
         description = "Parse a CSV string into rows")]
fn parse_csv(input: &Input) -> PluginResult<Output> {
    let data = input.string("data")
        .ok_or_else(|| PluginError::missing("data"))?;
    let rows = parse(data);
    Ok(Output::new()
        .set("rows", json!(rows))
        .set("count", json!(rows.len())))
}
```

The function name is used for dispatch: `parse_csv` matches action names `"parse-csv"` and `"parse_csv"`.

**Roles**: `"request"`, `"own"`, `"response"`, `"export"`

## Qualifiers

Qualifiers transform values using `<value: Handle.qualifier>` syntax in ARO:

```rust
#[qualifier(name = "reverse", input_types = ["List", "String"],
            description = "Reverse elements or characters")]
fn qualifier_reverse(input: &Input) -> PluginResult<Output> {
    if let Some(arr) = input.array("value") {
        let reversed: Vec<Value> = arr.iter().rev().cloned().collect();
        return Ok(Output::value(json!(reversed)));
    }
    if let Some(s) = input.string("value") {
        return Ok(Output::value(json!(s.chars().rev().collect::<String>())));
    }
    Err(PluginError::invalid_type("value", "a list or string"))
}
```

Qualifier function names are mapped by stripping `qualifier_` prefix: `qualifier_reverse` → `"reverse"`.

## The `aro_export!` Macro

Ties everything together — lists all action and qualifier functions:

```rust
aro_export! {
    name: "my-plugin",
    version: "1.0.0",
    handle: "MyHandle",
    actions: [greet, parse_csv, format_csv],
    qualifiers: [qualifier_reverse, qualifier_sort],
}
```

This generates:
- `aro_plugin_info()` → JSON with plugin metadata and all action/qualifier definitions
- `aro_plugin_execute()` → dispatches to the right action function
- `aro_plugin_qualifier()` → dispatches to the right qualifier function (only if qualifiers are listed)
- `aro_plugin_free()` → frees C strings allocated by the plugin
- `aro_plugin_init()` / `aro_plugin_shutdown()` → lifecycle hooks (no-op by default)

## Input API

`Input` provides type-safe access to the JSON envelope from the ARO runtime:

```rust
// Primary data (top-level keys take precedence over _with)
input.string("name")              // Option<&str>
input.int("count")                // Option<i64>
input.float("price")              // Option<f64>
input.bool("enabled")             // Option<bool>
input.array("items")              // Option<&Vec<Value>>
input.get("key")                  // Option<&Value>
input.raw()                       // &Value

// With-clause parameters: with { order: "asc", limit: 10 }
let params = input.with_params();  // Params
params.string("order")             // Option<&str>
params.string_or("order", "asc")   // &str (with default)
params.int("limit")                // Option<i64>
params.int_or("limit", 10)         // i64 (with default)
params.bool_or("verbose", false)   // bool (with default)
params.contains("key")             // bool

// ARO statement descriptors
input.result_identifier()          // Option<&str>  — e.g. "greeting"
input.result_qualifier()           // Option<&str>  — e.g. "formal"
input.source_identifier()          // Option<&str>  — e.g. "user-data"
input.preposition()                // Option<&str>  — e.g. "with"

// Execution context
input.context()                    // Option<&Value>
input.context_get("requestId")     // Option<&Value>
```

## Output API

`Output` is a fluent builder for the JSON response:

```rust
// Simple key-value output
Output::new()
    .set("result", json!("value"))
    .set("count", json!(42))

// Single-value output (for qualifiers)
Output::value(json!("reversed string"))

// With event emission
Output::new()
    .set("user", json!(user))
    .emit("UserCreated", json!({"id": user_id}))
```

## Error Handling

Use `PluginError` with standard error codes:

```rust
// Convenience constructors
PluginError::missing("data")                       // code 1: MissingInput
PluginError::invalid_type("count", "an integer")   // code 2: InvalidType
PluginError::not_found("user/42")                  // code 7: NotFound
PluginError::internal("unexpected state")           // code 10: InternalError

// Custom error code
PluginError::new(PluginErrorCode::Timeout, "database query timed out")
```

| Code | Name | Description |
|------|------|-------------|
| 0 | `Unknown` | Generic error |
| 1 | `MissingInput` | Required field missing |
| 2 | `InvalidType` | Type mismatch |
| 3 | `OutOfRange` | Value out of range |
| 4 | `IoError` | I/O operation failed |
| 5 | `NetworkError` | Network/connection error |
| 6 | `SerializationError` | JSON encoding error |
| 7 | `NotFound` | Resource not found |
| 8 | `Unauthorized` | Access denied |
| 9 | `Timeout` | Operation timed out |
| 10 | `InternalError` | Plugin bug |

## Event Emission

Actions can emit events that trigger other ARO feature sets:

```rust
Ok(Output::new()
    .set("order", json!(order))
    .emit("OrderCreated", json!({"orderId": order.id}))
    .emit("InventoryReserved", json!({"items": order.items})))
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use aro_plugin_sdk::testing::mock_input;

    #[test]
    fn test_greet() {
        let input = mock_input(json!({"name": "Alice"}));
        let result = greet(&input).unwrap().to_value();
        assert_eq!(result["greeting"], "Hello, Alice!");
    }

    #[test]
    fn test_greet_default() {
        let input = mock_input(json!({}));
        let result = greet(&input).unwrap().to_value();
        assert_eq!(result["greeting"], "Hello, World!");
    }

    #[test]
    fn test_with_params() {
        let input = mock_input(json!({
            "data": "hello",
            "_with": {"format": "uppercase"}
        }));
        let params = input.with_params();
        assert_eq!(params.string("format"), Some("uppercase"));
    }
}
```

## Building

```bash
cargo build --release
# Output: target/release/libmy_plugin.dylib (macOS) or .so (Linux)
```

## Complete Example

```rust
use aro_plugin_sdk::prelude::*;

#[action(name = "ParseCSV", verbs = ["parsecsv", "readcsv"], role = "own",
         prepositions = ["from", "with"],
         description = "Parse a CSV string into an array of rows")]
fn parse_csv(input: &Input) -> PluginResult<Output> {
    let data = input.string("data")
        .ok_or_else(|| PluginError::missing("data"))?;
    let has_headers = input.bool("headers").unwrap_or(true);
    // ... parse CSV ...
    Ok(Output::new().set("rows", json!(rows)).set("count", json!(rows.len())))
}

#[action(name = "FormatCSV", verbs = ["formatcsv"], role = "own",
         prepositions = ["from", "with"],
         description = "Format rows as a CSV string")]
fn format_csv(input: &Input) -> PluginResult<Output> {
    let rows = input.array("rows")
        .ok_or_else(|| PluginError::missing("rows"))?;
    // ... format CSV ...
    Ok(Output::new().set("csv", json!(csv_string)))
}

aro_export! {
    name: "plugin-rust-csv",
    version: "1.0.0",
    handle: "CSV",
    actions: [parse_csv, format_csv],
    qualifiers: [],
}
```

## License

MIT
