use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;
use anyhow::{Result, Context, bail};

#[derive(Debug, Clone, PartialEq)]
pub enum CppType {
    Void,
    Int,
    Long,
    Float,
    Double,
    Bool,
    String,  // Phase 2: String support
}

impl CppType {
    /// Parse C++ type from string
    pub fn from_str(type_str: &str) -> Result<Self> {
        let type_str = type_str.trim();
        match type_str {
            "void" => Ok(CppType::Void),
            "int" | "int32_t" => Ok(CppType::Int),
            "long" | "long long" | "int64_t" => Ok(CppType::Long),
            "float" => Ok(CppType::Float),
            "double" => Ok(CppType::Double),
            "bool" => Ok(CppType::Bool),
            "std::string" => Ok(CppType::String),
            "string" => Ok(CppType::String),
            _ => bail!("Unsupported C++ type: '{}'. Phase 2 supports: int, long, float, double, bool, std::string", type_str),
        }
    }

    /// Get the corresponding JNI type name
    pub fn to_jni_type(&self) -> &'static str {
        match self {
            CppType::Void => "void",
            CppType::Int => "jint",
            CppType::Long => "jlong",
            CppType::Float => "jfloat",
            CppType::Double => "jdouble",
            CppType::Bool => "jboolean",
            CppType::String => "jstring",
        }
    }

    /// Get the corresponding Kotlin type name
    pub fn to_kotlin_type(&self) -> &'static str {
        match self {
            CppType::Void => "Unit",
            CppType::Int => "Int",
            CppType::Long => "Long",
            CppType::Float => "Float",
            CppType::Double => "Double",
            CppType::Bool => "Boolean",
            CppType::String => "String",
        }
    }

    /// Get the C++ type string for forward declarations
    pub fn to_cpp_type(&self) -> &'static str {
        match self {
            CppType::Void => "void",
            CppType::Int => "int",
            CppType::Long => "long long",
            CppType::Float => "float",
            CppType::Double => "double",
            CppType::Bool => "bool",
            CppType::String => "std::string",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CppFfiFunction {
    pub name: String,
    pub params: Vec<(String, CppType)>,
    pub return_type: CppType,
    pub source_file: PathBuf,
}

/// Discover all @ffi annotated functions in C++ files
pub fn discover_cpp_ffi(ffi_dir: &Path) -> Result<Vec<CppFfiFunction>> {
    let cpp_dir = ffi_dir.join("cpp");

    if !cpp_dir.exists() {
        return Ok(Vec::new());
    }

    let mut functions = Vec::new();

    // Find all .cpp files
    let entries = fs::read_dir(&cpp_dir)
        .context(format!("Failed to read directory: {}", cpp_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("cpp") {
            let file_functions = parse_cpp_file(&path)?;
            functions.extend(file_functions);
        }
    }

    Ok(functions)
}

/// Parse a single C++ file for @ffi annotations
fn parse_cpp_file(path: &Path) -> Result<Vec<CppFfiFunction>> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;

    parse_cpp_ffi_from_string(&content, path)
}

/// Parse C++ source code for @ffi annotations (testable)
pub fn parse_cpp_ffi_from_string(content: &str, source_file: &Path) -> Result<Vec<CppFfiFunction>> {
    let mut functions = Vec::new();

    // Regex to match: // @ffi followed by a function signature
    // Pattern: return_type function_name(params) {
    // Note: Handles multi-word types like "long long" and namespaced types like "std::string"
    let ffi_regex = Regex::new(
        r"(?m)^\s*//\s*@ffi\s*\n\s*([\w:]+(?:\s+[\w:]+)*)\s+(\w+)\s*\(([^)]*)\)"
    ).unwrap();

    for cap in ffi_regex.captures_iter(content) {
        let return_type_str = cap.get(1).unwrap().as_str().trim();
        let function_name = cap.get(2).unwrap().as_str().trim();
        let params_str = cap.get(3).unwrap().as_str().trim();

        // Parse return type
        let return_type = CppType::from_str(return_type_str)
            .context(format!("In function '{}' at {}", function_name, source_file.display()))?;

        // Parse parameters
        let params = parse_parameters(params_str, function_name, source_file)?;

        functions.push(CppFfiFunction {
            name: function_name.to_string(),
            params,
            return_type,
            source_file: source_file.to_path_buf(),
        });
    }

    Ok(functions)
}

/// Parse parameter list: "int a, float b" -> [("a", Int), ("b", Float)]
/// Handles multi-word types like "long long"
fn parse_parameters(params_str: &str, function_name: &str, source_file: &Path) -> Result<Vec<(String, CppType)>> {
    if params_str.is_empty() {
        return Ok(Vec::new());
    }

    let mut params = Vec::new();

    for param in params_str.split(',') {
        let param = param.trim();
        if param.is_empty() {
            continue;
        }

        // Split into type and name
        // For "int a" or "long long a": last word is name, everything before is type
        let parts: Vec<&str> = param.split_whitespace().collect();

        if parts.len() < 2 {
            bail!(
                "Invalid parameter '{}' in function '{}' at {}. Expected format: 'type name'",
                param, function_name, source_file.display()
            );
        }

        // Last part is the parameter name
        let param_name = parts[parts.len() - 1];

        // Everything before is the type (handles "long long", "unsigned int", etc.)
        let param_type_str = parts[..parts.len() - 1].join(" ");

        let param_type = CppType::from_str(&param_type_str)
            .context(format!(
                "In parameter '{}' of function '{}' at {}",
                param_name, function_name, source_file.display()
            ))?;

        params.push((param_name.to_string(), param_type));
    }

    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let cpp = r#"
            // @ffi
            int add(int a, int b) {
                return a + b;
            }
        "#;

        let path = Path::new("test.cpp");
        let functions = parse_cpp_ffi_from_string(cpp, path).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "add");
        assert_eq!(functions[0].return_type, CppType::Int);
        assert_eq!(functions[0].params.len(), 2);
        assert_eq!(functions[0].params[0].0, "a");
        assert_eq!(functions[0].params[0].1, CppType::Int);
        assert_eq!(functions[0].params[1].0, "b");
        assert_eq!(functions[0].params[1].1, CppType::Int);
    }

    #[test]
    fn test_parse_multiple_functions() {
        let cpp = r#"
            // @ffi
            int add(int a, int b) {
                return a + b;
            }

            // Not FFI - should be ignored
            int helper(int x) {
                return x * 2;
            }

            // @ffi
            double multiply(double x, double y) {
                return x * y;
            }
        "#;

        let path = Path::new("test.cpp");
        let functions = parse_cpp_ffi_from_string(cpp, path).unwrap();

        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "add");
        assert_eq!(functions[1].name, "multiply");
        assert_eq!(functions[1].return_type, CppType::Double);
    }

    #[test]
    fn test_parse_no_params() {
        let cpp = r#"
            // @ffi
            int getRandom() {
                return 42;
            }
        "#;

        let path = Path::new("test.cpp");
        let functions = parse_cpp_ffi_from_string(cpp, path).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "getRandom");
        assert_eq!(functions[0].params.len(), 0);
    }

    #[test]
    fn test_parse_all_types() {
        let cpp = r#"
            // @ffi
            bool isPositive(int n) {
                return n > 0;
            }

            // @ffi
            float square(float x) {
                return x * x;
            }

            // @ffi
            long long factorial(long n) {
                return n;
            }
        "#;

        let path = Path::new("test.cpp");
        let functions = parse_cpp_ffi_from_string(cpp, path).unwrap();

        assert_eq!(functions.len(), 3);
        assert_eq!(functions[0].return_type, CppType::Bool);
        assert_eq!(functions[1].return_type, CppType::Float);
        assert_eq!(functions[2].return_type, CppType::Long);
    }

    #[test]
    fn test_string_support() {
        let cpp = r#"
            // @ffi
            std::string greet(std::string name) {
                return "Hello, " + name;
            }

            // @ffi
            std::string toUpper(std::string text) {
                return text;
            }
        "#;

        let path = Path::new("test.cpp");
        let functions = parse_cpp_ffi_from_string(cpp, path).unwrap();

        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "greet");
        assert_eq!(functions[0].return_type, CppType::String);
        assert_eq!(functions[0].params.len(), 1);
        assert_eq!(functions[0].params[0].1, CppType::String);

        assert_eq!(functions[1].name, "toUpper");
        assert_eq!(functions[1].return_type, CppType::String);
    }

    #[test]
    fn test_const_string_ref() {
        let cpp = r#"
            // @ffi
            std::string process(std::string input) {
                return input;
            }
        "#;

        let path = Path::new("test.cpp");
        let functions = parse_cpp_ffi_from_string(cpp, path).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].params[0].1, CppType::String);
    }

    #[test]
    fn test_type_conversions() {
        assert_eq!(CppType::Int.to_jni_type(), "jint");
        assert_eq!(CppType::Long.to_jni_type(), "jlong");
        assert_eq!(CppType::Float.to_jni_type(), "jfloat");
        assert_eq!(CppType::Double.to_jni_type(), "jdouble");
        assert_eq!(CppType::Bool.to_jni_type(), "jboolean");
        assert_eq!(CppType::String.to_jni_type(), "jstring");

        assert_eq!(CppType::Int.to_kotlin_type(), "Int");
        assert_eq!(CppType::Long.to_kotlin_type(), "Long");
        assert_eq!(CppType::Float.to_kotlin_type(), "Float");
        assert_eq!(CppType::Double.to_kotlin_type(), "Double");
        assert_eq!(CppType::Bool.to_kotlin_type(), "Boolean");
        assert_eq!(CppType::String.to_kotlin_type(), "String");

        assert_eq!(CppType::Int.to_cpp_type(), "int");
        assert_eq!(CppType::String.to_cpp_type(), "std::string");
    }
}
