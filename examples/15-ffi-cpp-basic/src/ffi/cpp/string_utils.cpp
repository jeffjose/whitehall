// Simple C++ FFI: String Utilities
// Demonstrates @ffi annotation for exposing C++ functions to Whitehall

#include <string>
#include <algorithm>
#include <cctype>

// @ffi
std::string to_uppercase(std::string str) {
    std::transform(str.begin(), str.end(), str.begin(), ::toupper);
    return str;
}

// @ffi
std::string to_lowercase(std::string str) {
    std::transform(str.begin(), str.end(), str.begin(), ::tolower);
    return str;
}

// @ffi
std::string reverse_string(std::string str) {
    std::reverse(str.begin(), str.end());
    return str;
}

// @ffi
int count_vowels(std::string str) {
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
bool is_palindrome(std::string str) {
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
std::string repeat_string(std::string str, int times) {
    std::string result;
    for (int i = 0; i < times; i++) {
        result += str;
    }
    return result;
}
