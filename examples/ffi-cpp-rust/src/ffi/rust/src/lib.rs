// Rust FFI Implementation
//
// This demonstrates Rust FFI in a mixed C++/Rust project.
// We implement subtraction and division in Rust.

use whitehall_ffi_macro::ffi;

#[ffi]
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[ffi]
pub fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        return 0.0;
    }
    a / b
}

#[ffi]
pub fn modulo(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0;
    }
    a % b
}

#[ffi]
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}


// Auto-generated JNI bridge (Phase 1.6)
mod jni_bridge;
