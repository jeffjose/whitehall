// C++ FFI Example: Simple Math Operations
//
// This file demonstrates the @ffi annotation for exposing C++ functions
// to Whitehall/Kotlin code. No JNI boilerplate needed!

// @ffi
int add(int a, int b) {
    return a + b;
}

// @ffi
int multiply(int a, int b) {
    return a * b;
}

// @ffi
double divide(double a, double b) {
    if (b == 0.0) {
        return 0.0; // Simple error handling
    }
    return a / b;
}

// This function is NOT exported (no @ffi annotation)
int helper(int x) {
    return x * 2;
}
