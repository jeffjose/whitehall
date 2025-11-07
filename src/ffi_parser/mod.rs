pub mod cpp;
pub mod rust;

pub use cpp::{CppFfiFunction, CppType, discover_cpp_ffi};
pub use rust::{RustFfiFunction, RustType, discover_rust_ffi};
