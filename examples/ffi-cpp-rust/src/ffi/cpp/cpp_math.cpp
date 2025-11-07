// C++ FFI Implementation
//
// This demonstrates C++ FFI in a mixed C++/Rust project.
// We implement addition and multiplication in C++.

// @ffi
int add(int a, int b) {
    return a + b;
}

// @ffi
int multiply(int a, int b) {
    return a * b;
}

// @ffi
double power(double base, double exponent) {
    double result = 1.0;
    for (int i = 0; i < static_cast<int>(exponent); i++) {
        result *= base;
    }
    return result;
}
