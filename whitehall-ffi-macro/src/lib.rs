//! Whitehall FFI Macro
//!
//! Provides the #[ffi] attribute for marking functions to be exposed via JNI.
//! This is a marker attribute that is detected by the Whitehall build system
//! during code generation. The attribute itself does nothing at runtime.

extern crate proc_macro;
use proc_macro::TokenStream;

/// Marker attribute for functions to be exposed via FFI/JNI
///
/// # Example
/// ```
/// #[ffi]
/// pub fn add(a: i32, b: i32) -> i32 {
///     a + b
/// }
/// ```
#[proc_macro_attribute]
pub fn ffi(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass through unchanged - this is just a marker for whitehall's parser
    item
}
