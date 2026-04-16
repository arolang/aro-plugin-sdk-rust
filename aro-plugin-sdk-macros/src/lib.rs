//! Proc-macro crate for the ARO plugin Rust SDK.
//!
//! # Macros
//!
//! | Macro | Purpose |
//! |-------|---------|
//! | `#[aro_plugin]` | Annotate a module as the plugin root; generates all C ABI exports |
//! | `#[action]` | Mark a function as an action handler |
//! | `#[qualifier]` | Mark a function as a qualifier handler |
//! | `#[init]` | Mark a function as the plugin init hook |
//! | `#[shutdown]` | Mark a function as the plugin shutdown hook |

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, ExprArray, ExprLit, Ident, ItemFn, Lit, LitStr, Token};

// ---------------------------------------------------------------------------
// Helper: parse `key = value` pairs from attribute arguments
// ---------------------------------------------------------------------------

struct KeyValue {
    key: Ident,
    _eq: Token![=],
    value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(KeyValue {
            key: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct AttrArgs {
    pairs: Punctuated<KeyValue, Token![,]>,
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AttrArgs {
            pairs: Punctuated::parse_terminated(input)?,
        })
    }
}

fn get_str(args: &AttrArgs, key: &str) -> Option<String> {
    args.pairs.iter().find(|kv| kv.key == key).and_then(|kv| {
        if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &kv.value {
            Some(s.value())
        } else {
            None
        }
    })
}

fn get_str_array(args: &AttrArgs, key: &str) -> Vec<String> {
    args.pairs
        .iter()
        .find(|kv| kv.key == key)
        .map(|kv| {
            if let Expr::Array(ExprArray { elems, .. }) = &kv.value {
                elems
                    .iter()
                    .filter_map(|e| {
                        if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = e {
                            Some(s.value())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                vec![]
            }
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Attribute storage via hidden constants
// ---------------------------------------------------------------------------

// Actions and qualifiers annotated with #[action] / #[qualifier] store their
// metadata in a hidden const so the #[aro_plugin] module-level macro can
// collect them.  The naming convention is:
//
//   const __ARO_ACTION_<FN_NAME>: &str = r#"{"name":"...", ...}"#;
//   const __ARO_QUALIFIER_<FN_NAME>: &str = r#"{"name":"...", ...}"#;

/// Mark a function as an ARO action handler.
///
/// # Example
/// ```ignore
/// #[action(name = "ParseCSV", verbs = ["parsecsv"], role = "own",
///          prepositions = ["from", "with"], description = "Parse CSV data")]
/// fn parse_csv(input: &Input) -> PluginResult<Output> { ... }
/// ```
#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttrArgs);
    let func = parse_macro_input!(item as ItemFn);
    let fn_name = &func.sig.ident;

    let name = get_str(&args, "name").unwrap_or_else(|| fn_name.to_string());
    let verbs = get_str_array(&args, "verbs");
    let role = get_str(&args, "role").unwrap_or_else(|| "own".into());
    let preps = get_str_array(&args, "prepositions");
    let desc = get_str(&args, "description").unwrap_or_default();

    let verbs_json: Vec<String> = verbs.iter().map(|v| format!("\"{}\"", v)).collect();
    let preps_json: Vec<String> = preps.iter().map(|p| format!("\"{}\"", p)).collect();
    let json_lit = format!(
        r#"{{"name":"{}","verbs":[{}],"role":"{}","prepositions":[{}],"description":"{}"}}"#,
        name,
        verbs_json.join(","),
        role,
        preps_json.join(","),
        desc,
    );

    let const_name = format_ident!("__ARO_ACTION_{}", fn_name.to_string().to_uppercase());

    let expanded = quote! {
        #[doc(hidden)]
        const #const_name: &str = #json_lit;

        #func
    };
    expanded.into()
}

/// Mark a function as an ARO qualifier handler.
///
/// # Example
/// ```ignore
/// #[qualifier(name = "reverse", input_types = ["List", "String"],
///             description = "Reverse elements")]
/// fn qualifier_reverse(input: &Input) -> PluginResult<Output> { ... }
/// ```
#[proc_macro_attribute]
pub fn qualifier(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttrArgs);
    let func = parse_macro_input!(item as ItemFn);
    let fn_name = &func.sig.ident;

    let name = get_str(&args, "name").unwrap_or_else(|| {
        fn_name.to_string().strip_prefix("qualifier_").unwrap_or(&fn_name.to_string()).to_string()
    });
    let input_types = get_str_array(&args, "input_types");
    let desc = get_str(&args, "description").unwrap_or_default();

    let types_json: Vec<String> = input_types.iter().map(|t| format!("\"{}\"", t)).collect();
    let json_lit = format!(
        r#"{{"name":"{}","inputTypes":[{}],"description":"{}"}}"#,
        name,
        types_json.join(","),
        desc,
    );

    let const_name = format_ident!("__ARO_QUALIFIER_{}", fn_name.to_string().to_uppercase());

    let expanded = quote! {
        #[doc(hidden)]
        const #const_name: &str = #json_lit;

        #func
    };
    expanded.into()
}

/// Mark a function as the plugin init hook.
#[proc_macro_attribute]
pub fn init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let fn_name = &func.sig.ident;
    let marker = format_ident!("__ARO_INIT_FN");

    let expanded = quote! {
        #func

        #[doc(hidden)]
        const #marker: fn() = #fn_name;
    };
    expanded.into()
}

/// Mark a function as the plugin shutdown hook.
#[proc_macro_attribute]
pub fn shutdown(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let fn_name = &func.sig.ident;
    let marker = format_ident!("__ARO_SHUTDOWN_FN");

    let expanded = quote! {
        #func

        #[doc(hidden)]
        const #marker: fn() = #fn_name;
    };
    expanded.into()
}

/// Annotate a module or struct as the ARO plugin root.
///
/// Not needed for stub purposes but reserved for future use.
#[proc_macro_attribute]
pub fn aro_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a struct as an ARO system object provider.
#[proc_macro_attribute]
pub fn system_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Mark a function as an ARO event handler.
#[proc_macro_attribute]
pub fn on_event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Generate all C ABI exports for the plugin.
///
/// Place this at crate root after all `#[action]` and `#[qualifier]` functions.
///
/// # Example
/// ```ignore
/// use aro_plugin_sdk::prelude::*;
///
/// #[action(name = "Greet", verbs = ["greet"], role = "own", prepositions = ["with"])]
/// fn greet(input: &Input) -> PluginResult<Output> { ... }
///
/// aro_export! {
///     name: "my-plugin",
///     version: "1.0.0",
///     handle: "My",
///     actions: [greet],
///     qualifiers: [],
/// }
/// ```
#[proc_macro]
pub fn aro_export(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as ExportConfig);

    let name = &config.name;
    let version = &config.version;
    let handle = &config.handle;

    // Build action metadata JSON fragments
    let action_fns = &config.actions;
    let action_consts: Vec<Ident> = action_fns
        .iter()
        .map(|f| format_ident!("__ARO_ACTION_{}", f.to_string().to_uppercase()))
        .collect();

    // Build qualifier metadata JSON fragments
    let qualifier_fns = &config.qualifiers;
    let qualifier_consts: Vec<Ident> = qualifier_fns
        .iter()
        .map(|f| format_ident!("__ARO_QUALIFIER_{}", f.to_string().to_uppercase()))
        .collect();

    // Build action dispatch arms
    let action_dispatches: Vec<TokenStream2> = action_fns
        .iter()
        .map(|f| {
            let fn_ident = f;
            let kebab = f.to_string().replace('_', "-");
            let raw = f.to_string();
            quote! {
                #kebab | #raw => #fn_ident(&input),
            }
        })
        .collect();

    // Build qualifier dispatch arms
    let qualifier_dispatches: Vec<TokenStream2> = qualifier_fns
        .iter()
        .map(|f| {
            let fn_ident = f;
            let name_str = f.to_string().strip_prefix("qualifier_").unwrap_or(&f.to_string()).to_string();
            let kebab = name_str.replace('_', "-");
            let raw = name_str.clone();
            quote! {
                #kebab | #raw => #fn_ident(&input),
            }
        })
        .collect();

    let has_qualifiers = !qualifier_fns.is_empty();

    // Generate aro_plugin_qualifier only if there are qualifiers
    let qualifier_export = if has_qualifiers {
        quote! {
            #[no_mangle]
            pub extern "C" fn aro_plugin_qualifier(
                qualifier: *const ::std::os::raw::c_char,
                input_json: *const ::std::os::raw::c_char,
            ) -> *mut ::std::os::raw::c_char {
                ::aro_plugin_sdk::ffi::wrap_qualifier(qualifier, input_json, |name, input| {
                    match name {
                        #(#qualifier_dispatches)*
                        _ => Err(::aro_plugin_sdk::PluginError::internal(
                            format!("Unknown qualifier: {name}")
                        )),
                    }
                })
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn aro_plugin_info() -> *mut ::std::os::raw::c_char {
            let actions_json = vec![#(#action_consts),*];
            let qualifiers_json = vec![#(#qualifier_consts),*];

            let actions_str = actions_json.join(",");
            let qualifiers_str = qualifiers_json.join(",");

            let info = format!(
                r#"{{"name":"{}","version":"{}","handle":"{}","actions":[{}],"qualifiers":[{}]}}"#,
                #name, #version, #handle, actions_str, qualifiers_str,
            );
            ::aro_plugin_sdk::ffi::to_c_string(info)
        }

        #[no_mangle]
        pub extern "C" fn aro_plugin_execute(
            action: *const ::std::os::raw::c_char,
            input_json: *const ::std::os::raw::c_char,
        ) -> *mut ::std::os::raw::c_char {
            ::aro_plugin_sdk::ffi::wrap_execute(action, input_json, |action, input| {
                match action {
                    #(#action_dispatches)*
                    _ => Err(::aro_plugin_sdk::PluginError::internal(
                        format!("Unknown action: {action}")
                    )),
                }
            })
        }

        #qualifier_export

        #[no_mangle]
        pub extern "C" fn aro_plugin_free(ptr: *mut ::std::os::raw::c_char) {
            ::aro_plugin_sdk::ffi::free_c_string(ptr);
        }

        #[no_mangle]
        pub extern "C" fn aro_plugin_init() {}

        #[no_mangle]
        pub extern "C" fn aro_plugin_shutdown() {}
    };

    expanded.into()
}

// ---------------------------------------------------------------------------
// Parser for aro_export! { ... }
// ---------------------------------------------------------------------------

struct ExportConfig {
    name: LitStr,
    version: LitStr,
    handle: LitStr,
    actions: Vec<Ident>,
    qualifiers: Vec<Ident>,
}

impl Parse for ExportConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut version = None;
        let mut handle = None;
        let mut actions = vec![];
        let mut qualifiers = vec![];

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _: Token![:] = input.parse()?;

            match key.to_string().as_str() {
                "name" => { name = Some(input.parse::<LitStr>()?); }
                "version" => { version = Some(input.parse::<LitStr>()?); }
                "handle" => { handle = Some(input.parse::<LitStr>()?); }
                "actions" => {
                    let content;
                    syn::bracketed!(content in input);
                    actions = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "qualifiers" => {
                    let content;
                    syn::bracketed!(content in input);
                    qualifiers = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                _ => return Err(syn::Error::new(key.span(), format!("Unknown key: {key}"))),
            }

            // Optional trailing comma
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(ExportConfig {
            name: name.ok_or_else(|| input.error("missing `name`"))?,
            version: version.ok_or_else(|| input.error("missing `version`"))?,
            handle: handle.ok_or_else(|| input.error("missing `handle`"))?,
            actions,
            qualifiers,
        })
    }
}
