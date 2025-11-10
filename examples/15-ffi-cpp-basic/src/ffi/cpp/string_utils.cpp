// Simple C++ FFI: String Utilities
// Demonstrates @ffi annotation for exposing C++ functions to Whitehall

#include <string>
#include <algorithm>
#include <cctype>

// @ffi
std::string to_uppercase(const std::string& str) {
    std::string result = str;
    std::transform(result.begin(), result.end(), result.begin(), ::toupper);
    return result;
}

// @ffi
std::string to_lowercase(const std::string& str) {
    std::string result = str;
    std::transform(result.begin(), result.end(), result.begin(), ::tolower);
    return result;
}

// @ffi
std::string reverse_string(const std::string& str) {
    std::string result = str;
    std::reverse(result.begin(), result.end());
    return result;
}

// @ffi
int count_vowels(const std::string& str) {
    int count = 0;
    for (char c : str) {
        char lower = std::tolower(c);
        if (lower == 'a' || lower == 'e' || lower == 'i' ||
            lower == 'o' || lower == 'u') {
            count++;
        }
    }
    return count;
}

// @ffi
bool is_palindrome(const std::string& str) {
    std::string cleaned;
    for (char c : str) {
        if (std::isalnum(c)) {
            cleaned += std::tolower(c);
        }
    }

    std::string reversed = cleaned;
    std::reverse(reversed.begin(), reversed.end());

    return cleaned == reversed;
}

// @ffi
std::string repeat_string(const std::string& str, int times) {
    std::string result;
    for (int i = 0; i < times; i++) {
        result += str;
    }
    return result;
}
