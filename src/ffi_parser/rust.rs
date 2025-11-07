use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context, bail};
use syn::{self, Item, ItemFn, FnArg, Pat, Type, ReturnType, Attribute};

#[derive(Debug, Clone, PartialEq)]
pub enum RustType {
    Void,
    Int,
    Long,
    Float,
    Double,
    Bool,
    String,
    // Phase 3/4: Array support
    IntArray,
    LongArray,
    FloatArray,
    DoubleArray,
    BoolArray,
    StringArray,
}

/// Wrapper that indicates whether a type is a Result<T, E>
#[derive(Debug, Clone, PartialEq)]
pub struct RustFunctionReturn {
    pub base_type: RustType,
    pub is_result: bool,  // true if this is Result<T, E>
}

impl RustFunctionReturn {
    pub fn plain(base_type: RustType) -> Self {
        RustFunctionReturn {
            base_type,
            is_result: false,
        }
    }

    pub fn result(base_type: RustType) -> Self {
        RustFunctionReturn {
            base_type,
            is_result: true,
        }
    }

    /// Get the Kotlin type (same whether Result or not)
    pub fn to_kotlin_type(&self) -> &'static str {
        self.base_type.to_kotlin_type()
    }

    /// Get the JNI type (same whether Result or not)
    pub fn to_jni_type(&self) -> &'static str {
        self.base_type.to_jni_type()
    }

    /// Get the Rust type string
    pub fn to_rust_type(&self) -> String {
        if self.is_result {
            format!("Result<{}, String>", self.base_type.to_rust_type())
        } else {
            self.base_type.to_rust_type().to_string()
        }
    }
}

impl RustType {
    /// Parse Rust type from syn::Type
    pub fn from_syn_type(ty: &Type) -> Result<Self> {
        match ty {
            Type::Path(type_path) => {
                let segments = &type_path.path.segments;

                if segments.is_empty() {
                    bail!("Empty type path");
                }

                let last_segment = &segments[segments.len() - 1];
                let type_name = last_segment.ident.to_string();

                // Check for Vec<T> (array types)
                if type_name == "Vec" {
                    if let syn::PathArguments::AngleBracketed(ref args) = last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return Self::vec_element_type(inner_ty);
                        }
                    }
                    bail!("Vec must have a type parameter");
                }

                // Simple types
                match type_name.as_str() {
                    "i32" => Ok(RustType::Int),
                    "i64" => Ok(RustType::Long),
                    "f32" => Ok(RustType::Float),
                    "f64" => Ok(RustType::Double),
                    "bool" => Ok(RustType::Bool),
                    "String" => Ok(RustType::String),
                    _ => bail!("Unsupported Rust type: '{}'. Phase 4 supports: i32, i64, f32, f64, bool, String, Vec<T>", type_name),
                }
            }
            Type::Tuple(tuple) => {
                if tuple.elems.is_empty() {
                    Ok(RustType::Void)
                } else {
                    bail!("Tuple types not supported (except unit type ())")
                }
            }
            _ => bail!("Unsupported Rust type structure"),
        }
    }

    /// Get the RustType for a Vec element type
    fn vec_element_type(ty: &Type) -> Result<Self> {
        match ty {
            Type::Path(type_path) => {
                let segments = &type_path.path.segments;
                if segments.is_empty() {
                    bail!("Empty type path in Vec");
                }

                let last_segment = &segments[segments.len() - 1];
                let type_name = last_segment.ident.to_string();

                match type_name.as_str() {
                    "i32" => Ok(RustType::IntArray),
                    "i64" => Ok(RustType::LongArray),
                    "f32" => Ok(RustType::FloatArray),
                    "f64" => Ok(RustType::DoubleArray),
                    "bool" => Ok(RustType::BoolArray),
                    "String" => Ok(RustType::StringArray),
                    _ => bail!("Unsupported Vec element type: '{}'. Supported: i32, i64, f32, f64, bool, String", type_name),
                }
            }
            _ => bail!("Unsupported Vec element type structure"),
        }
    }

    /// Get the corresponding JNI type name
    pub fn to_jni_type(&self) -> &'static str {
        match self {
            RustType::Void => "void",
            RustType::Int => "jint",
            RustType::Long => "jlong",
            RustType::Float => "jfloat",
            RustType::Double => "jdouble",
            RustType::Bool => "jboolean",
            RustType::String => "jstring",
            RustType::IntArray => "jintArray",
            RustType::LongArray => "jlongArray",
            RustType::FloatArray => "jfloatArray",
            RustType::DoubleArray => "jdoubleArray",
            RustType::BoolArray => "jbooleanArray",
            RustType::StringArray => "jobjectArray",
        }
    }

    /// Get the corresponding Kotlin type name
    pub fn to_kotlin_type(&self) -> &'static str {
        match self {
            RustType::Void => "Unit",
            RustType::Int => "Int",
            RustType::Long => "Long",
            RustType::Float => "Float",
            RustType::Double => "Double",
            RustType::Bool => "Boolean",
            RustType::String => "String",
            RustType::IntArray => "IntArray",
            RustType::LongArray => "LongArray",
            RustType::FloatArray => "FloatArray",
            RustType::DoubleArray => "DoubleArray",
            RustType::BoolArray => "BooleanArray",
            RustType::StringArray => "Array<String>",
        }
    }

    /// Get the Rust type string for forward declarations
    pub fn to_rust_type(&self) -> &'static str {
        match self {
            RustType::Void => "()",
            RustType::Int => "i32",
            RustType::Long => "i64",
            RustType::Float => "f32",
            RustType::Double => "f64",
            RustType::Bool => "bool",
            RustType::String => "String",
            RustType::IntArray => "Vec<i32>",
            RustType::LongArray => "Vec<i64>",
            RustType::FloatArray => "Vec<f32>",
            RustType::DoubleArray => "Vec<f64>",
            RustType::BoolArray => "Vec<bool>",
            RustType::StringArray => "Vec<String>",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RustFfiFunction {
    pub name: String,
    pub params: Vec<(String, RustType)>,
    pub return_type: RustFunctionReturn,  // Phase 5: Support Result<T, E>
    pub source_file: PathBuf,
}

/// Discover all #[ffi] annotated functions in Rust files
pub fn discover_rust_ffi(ffi_dir: &Path) -> Result<Vec<RustFfiFunction>> {
    let rust_dir = ffi_dir.join("rust");

    if !rust_dir.exists() {
        return Ok(Vec::new());
    }

    let mut functions = Vec::new();

    // Find all .rs files
    let entries = fs::read_dir(&rust_dir)
        .context(format!("Failed to read directory: {}", rust_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let file_functions = parse_rust_file(&path)?;
            functions.extend(file_functions);
        }
    }

    // Also check for src/lib.rs inside rust directory
    let lib_path = rust_dir.join("src").join("lib.rs");
    if lib_path.exists() {
        let file_functions = parse_rust_file(&lib_path)?;
        functions.extend(file_functions);
    }

    Ok(functions)
}

/// Parse a single Rust file for #[ffi] annotations
fn parse_rust_file(path: &Path) -> Result<Vec<RustFfiFunction>> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;

    parse_rust_ffi_from_string(&content, path)
}

/// Parse Rust source code for #[ffi] annotations (testable)
pub fn parse_rust_ffi_from_string(content: &str, source_file: &Path) -> Result<Vec<RustFfiFunction>> {
    let syntax = syn::parse_file(content)
        .context(format!("Failed to parse Rust file: {}", source_file.display()))?;

    let mut functions = Vec::new();

    for item in syntax.items {
        if let Item::Fn(func) = item {
            if has_ffi_attribute(&func.attrs) {
                let function = parse_ffi_function(func, source_file)?;
                functions.push(function);
            }
        }
    }

    Ok(functions)
}

/// Check if function has #[ffi] attribute
fn has_ffi_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("ffi")
    })
}

/// Parse a function with #[ffi] attribute
fn parse_ffi_function(func: ItemFn, source_file: &Path) -> Result<RustFfiFunction> {
    let function_name = func.sig.ident.to_string();

    // Parse parameters
    let mut params = Vec::new();
    for input in &func.sig.inputs {
        match input {
            FnArg::Typed(pat_type) => {
                // Get parameter name
                let param_name = match &*pat_type.pat {
                    Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                    _ => bail!("Unsupported parameter pattern in function '{}' at {}",
                              function_name, source_file.display()),
                };

                // Get parameter type
                let param_type = RustType::from_syn_type(&pat_type.ty)
                    .context(format!(
                        "In parameter '{}' of function '{}' at {}",
                        param_name, function_name, source_file.display()
                    ))?;

                params.push((param_name, param_type));
            }
            FnArg::Receiver(_) => {
                bail!("FFI functions cannot have 'self' parameter in function '{}' at {}",
                      function_name, source_file.display());
            }
        }
    }

    // Parse return type (Phase 5: Support Result<T, E>)
    let return_type = match &func.sig.output {
        ReturnType::Default => RustFunctionReturn::plain(RustType::Void),
        ReturnType::Type(_, ty) => parse_return_type(ty, &function_name, source_file)?,
    };

    Ok(RustFfiFunction {
        name: function_name,
        params,
        return_type,
        source_file: source_file.to_path_buf(),
    })
}

/// Parse return type, detecting Result<T, E> wrappers
fn parse_return_type(ty: &Type, function_name: &str, source_file: &Path) -> Result<RustFunctionReturn> {
    match ty {
        Type::Path(type_path) => {
            let segments = &type_path.path.segments;

            if segments.is_empty() {
                bail!("Empty type path in return type of '{}' at {}", function_name, source_file.display());
            }

            let last_segment = &segments[segments.len() - 1];
            let type_name = last_segment.ident.to_string();

            // Phase 5: Check for Result<T, E>
            if type_name == "Result" {
                if let syn::PathArguments::AngleBracketed(ref args) = last_segment.arguments {
                    if args.args.len() >= 1 {
                        // Extract T from Result<T, E>
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let base_type = RustType::from_syn_type(inner_ty)
                                .context(format!("In Result<T, E> return type of '{}' at {}",
                                               function_name, source_file.display()))?;
                            return Ok(RustFunctionReturn::result(base_type));
                        }
                    }
                }
                bail!("Result must have type parameters: Result<T, E> in function '{}' at {}",
                      function_name, source_file.display());
            }

            // Not a Result - parse as regular type
            let base_type = RustType::from_syn_type(ty)
                .context(format!("In return type of '{}' at {}", function_name, source_file.display()))?;
            Ok(RustFunctionReturn::plain(base_type))
        }
        _ => {
            let base_type = RustType::from_syn_type(ty)
                .context(format!("In return type of '{}' at {}", function_name, source_file.display()))?;
            Ok(RustFunctionReturn::plain(base_type))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let rust = r#"
            #[ffi]
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let path = Path::new("test.rs");
        let functions = parse_rust_ffi_from_string(rust, path).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "add");
        assert_eq!(functions[0].return_type, RustFunctionReturn::plain(RustType::Int));
        assert_eq!(functions[0].params.len(), 2);
        assert_eq!(functions[0].params[0].0, "a");
        assert_eq!(functions[0].params[0].1, RustType::Int);
        assert_eq!(functions[0].params[1].0, "b");
        assert_eq!(functions[0].params[1].1, RustType::Int);
    }

    #[test]
    fn test_parse_multiple_functions() {
        let rust = r#"
            #[ffi]
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            // Not FFI - should be ignored
            pub fn helper(x: i32) -> i32 {
                x * 2
            }

            #[ffi]
            pub fn multiply(x: f64, y: f64) -> f64 {
                x * y
            }
        "#;

        let path = Path::new("test.rs");
        let functions = parse_rust_ffi_from_string(rust, path).unwrap();

        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "add");
        assert_eq!(functions[1].name, "multiply");
        assert_eq!(functions[1].return_type, RustFunctionReturn::plain(RustType::Double));
    }

    #[test]
    fn test_parse_no_params() {
        let rust = r#"
            #[ffi]
            pub fn get_random() -> i32 {
                42
            }
        "#;

        let path = Path::new("test.rs");
        let functions = parse_rust_ffi_from_string(rust, path).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "get_random");
        assert_eq!(functions[0].params.len(), 0);
    }

    #[test]
    fn test_parse_all_types() {
        let rust = r#"
            #[ffi]
            pub fn is_positive(n: i32) -> bool {
                n > 0
            }

            #[ffi]
            pub fn square(x: f32) -> f32 {
                x * x
            }

            #[ffi]
            pub fn factorial(n: i64) -> i64 {
                n
            }
        "#;

        let path = Path::new("test.rs");
        let functions = parse_rust_ffi_from_string(rust, path).unwrap();

        assert_eq!(functions.len(), 3);
        assert_eq!(functions[0].return_type, RustFunctionReturn::plain(RustType::Bool));
        assert_eq!(functions[1].return_type, RustFunctionReturn::plain(RustType::Float));
        assert_eq!(functions[2].return_type, RustFunctionReturn::plain(RustType::Long));
    }

    #[test]
    fn test_string_support() {
        let rust = r#"
            #[ffi]
            pub fn greet(name: String) -> String {
                format!("Hello, {}!", name)
            }

            #[ffi]
            pub fn to_upper(text: String) -> String {
                text.to_uppercase()
            }
        "#;

        let path = Path::new("test.rs");
        let functions = parse_rust_ffi_from_string(rust, path).unwrap();

        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "greet");
        assert_eq!(functions[0].return_type, RustFunctionReturn::plain(RustType::String));
        assert_eq!(functions[0].params.len(), 1);
        assert_eq!(functions[0].params[0].1, RustType::String);

        assert_eq!(functions[1].name, "to_upper");
        assert_eq!(functions[1].return_type, RustFunctionReturn::plain(RustType::String));
    }

    #[test]
    fn test_void_return() {
        let rust = r#"
            #[ffi]
            pub fn log_message(msg: String) {
                println!("{}", msg);
            }
        "#;

        let path = Path::new("test.rs");
        let functions = parse_rust_ffi_from_string(rust, path).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].return_type, RustFunctionReturn::plain(RustType::Void));
    }

    #[test]
    fn test_type_conversions() {
        assert_eq!(RustType::Int.to_jni_type(), "jint");
        assert_eq!(RustType::Long.to_jni_type(), "jlong");
        assert_eq!(RustType::Float.to_jni_type(), "jfloat");
        assert_eq!(RustType::Double.to_jni_type(), "jdouble");
        assert_eq!(RustType::Bool.to_jni_type(), "jboolean");
        assert_eq!(RustType::String.to_jni_type(), "jstring");
        assert_eq!(RustType::IntArray.to_jni_type(), "jintArray");

        assert_eq!(RustType::Int.to_kotlin_type(), "Int");
        assert_eq!(RustType::Long.to_kotlin_type(), "Long");
        assert_eq!(RustType::Float.to_kotlin_type(), "Float");
        assert_eq!(RustType::Double.to_kotlin_type(), "Double");
        assert_eq!(RustType::Bool.to_kotlin_type(), "Boolean");
        assert_eq!(RustType::String.to_kotlin_type(), "String");
        assert_eq!(RustType::IntArray.to_kotlin_type(), "IntArray");

        assert_eq!(RustType::Int.to_rust_type(), "i32");
        assert_eq!(RustType::String.to_rust_type(), "String");
        assert_eq!(RustType::IntArray.to_rust_type(), "Vec<i32>");
        assert_eq!(RustType::StringArray.to_rust_type(), "Vec<String>");
    }

    #[test]
    fn test_array_support() {
        let rust = r#"
            #[ffi]
            pub fn double_values(values: Vec<i32>) -> Vec<i32> {
                values.iter().map(|v| v * 2).collect()
            }

            #[ffi]
            pub fn to_upper_all(strings: Vec<String>) -> Vec<String> {
                strings.iter().map(|s| s.to_uppercase()).collect()
            }
        "#;

        let path = Path::new("test.rs");
        let functions = parse_rust_ffi_from_string(rust, path).unwrap();

        assert_eq!(functions.len(), 2);

        // First function: Vec<i32>
        assert_eq!(functions[0].name, "double_values");
        assert_eq!(functions[0].return_type, RustFunctionReturn::plain(RustType::IntArray));
        assert_eq!(functions[0].params.len(), 1);
        assert_eq!(functions[0].params[0].1, RustType::IntArray);

        // Second function: Vec<String>
        assert_eq!(functions[1].name, "to_upper_all");
        assert_eq!(functions[1].return_type, RustFunctionReturn::plain(RustType::StringArray));
        assert_eq!(functions[1].params[0].1, RustType::StringArray);
    }

    #[test]
    fn test_all_array_types() {
        let rust = r#"
            #[ffi]
            pub fn process_ints(data: Vec<i32>) -> Vec<i32> { data }

            #[ffi]
            pub fn process_longs(data: Vec<i64>) -> Vec<i64> { data }

            #[ffi]
            pub fn process_floats(data: Vec<f32>) -> Vec<f32> { data }

            #[ffi]
            pub fn process_doubles(data: Vec<f64>) -> Vec<f64> { data }

            #[ffi]
            pub fn process_bools(data: Vec<bool>) -> Vec<bool> { data }

            #[ffi]
            pub fn process_strings(data: Vec<String>) -> Vec<String> { data }
        "#;

        let path = Path::new("test.rs");
        let functions = parse_rust_ffi_from_string(rust, path).unwrap();

        assert_eq!(functions.len(), 6);
        assert_eq!(functions[0].return_type, RustFunctionReturn::plain(RustType::IntArray));
        assert_eq!(functions[1].return_type, RustFunctionReturn::plain(RustType::LongArray));
        assert_eq!(functions[2].return_type, RustFunctionReturn::plain(RustType::FloatArray));
        assert_eq!(functions[3].return_type, RustFunctionReturn::plain(RustType::DoubleArray));
        assert_eq!(functions[4].return_type, RustFunctionReturn::plain(RustType::BoolArray));
        assert_eq!(functions[5].return_type, RustFunctionReturn::plain(RustType::StringArray));
    }
}
