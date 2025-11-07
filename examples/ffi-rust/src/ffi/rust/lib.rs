// Rust FFI Example: Simple Math Operations
//
// This file demonstrates the #[ffi] attribute for exposing Rust functions
// to Whitehall/Kotlin code. No JNI boilerplate needed!

#[ffi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[ffi]
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[ffi]
pub fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        return 0.0; // Simple error handling
    }
    a / b
}

#[ffi]
pub fn is_positive(n: i32) -> bool {
    n > 0
}

// This function is NOT exported (no #[ffi] attribute)
fn helper(x: i32) -> i32 {
    x * 2
}
