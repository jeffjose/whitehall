/// Abstract Syntax Tree for Whitehall

#[derive(Debug, Clone, PartialEq)]
pub struct WhitehallFile {
    pub imports: Vec<Import>,
    pub props: Vec<PropDeclaration>,
    pub state: Vec<StateDeclaration>,
    pub functions: Vec<FunctionDeclaration>,
    pub lifecycle_hooks: Vec<LifecycleHook>,
    pub classes: Vec<ClassDeclaration>,  // Store classes (@store annotation)
    pub markup: Markup,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String, // e.g., "$models.User" or "androidx.compose.ui.Modifier"
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropDeclaration {
    pub name: String,
    pub prop_type: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateDeclaration {
    pub name: String,
    pub mutable: bool,                   // var vs val
    pub type_annotation: Option<String>, // e.g., "List<Post>"
    pub initial_value: String,
    pub is_derived_state: bool,          // true if initial_value uses derivedStateOf
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: String,              // Parameters as string (e.g., "postId: String")
    pub return_type: Option<String>, // Optional return type (e.g., "String", "Unit")
    pub body: String,                // Just capture the whole function body as a string
    pub is_suspend: bool,            // Whether this is a suspend function
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifecycleHook {
    pub hook_type: String, // "onMount", "onUnmount", etc.
    pub body: String,      // Hook body content
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclaration {
    pub annotations: Vec<String>,          // e.g., ["store", "HiltViewModel"]
    pub is_object: bool,                   // true for "object", false for "class"
    pub name: String,                      // e.g., "UserProfile"
    pub constructor: Option<ConstructorDeclaration>,  // Constructor with @Inject
    pub properties: Vec<PropertyDeclaration>,  // var/val properties
    pub functions: Vec<FunctionDeclaration>,   // Methods
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstructorDeclaration {
    pub annotations: Vec<String>,  // e.g., ["Inject"]
    pub parameters: String,        // e.g., "private val repository: ProfileRepository"
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDeclaration {
    pub name: String,
    pub mutable: bool,                   // var vs val
    pub type_annotation: Option<String>, // e.g., "String"
    pub initial_value: Option<String>,   // e.g., "\"\"" or "false"
    pub getter: Option<String>,          // Custom getter for derived properties
    pub visibility: Option<String>,      // "private", "protected", "public", or None (default)
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Sequence variant reserved for future use
pub enum Markup {
    Component(Component),
    Text(String),
    Interpolation(String), // {variable} expression
    Sequence(Vec<Markup>), // Multiple markup items
    IfElse(IfElseBlock),   // @if/@else control flow
    ForLoop(ForLoopBlock), // @for loop with key and empty block
    When(WhenBlock),       // @when expression
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfElseBlock {
    pub condition: String,
    pub then_branch: Vec<Markup>,
    pub else_ifs: Vec<ElseIfBranch>,
    pub else_branch: Option<Vec<Markup>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseIfBranch {
    pub condition: String,
    pub body: Vec<Markup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoopBlock {
    pub item: String,         // e.g., "post"
    pub collection: String,   // e.g., "posts"
    pub key_expr: Option<String>, // e.g., "it.id" or "post.id"
    pub body: Vec<Markup>,    // Loop body content
    pub empty_block: Option<Vec<Markup>>, // Optional empty block
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenBlock {
    pub branches: Vec<WhenBranch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenBranch {
    pub condition: Option<String>, // None for "else" branch
    pub body: Markup,              // Single markup item per branch
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub props: Vec<ComponentProp>,
    pub children: Vec<Markup>,
    pub self_closing: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    /// Simple expression: {someVariable} or {expression}
    Expression(String),
    /// Component as prop value: {<Component />}
    Markup(Box<Markup>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentProp {
    pub name: String,
    pub value: PropValue,
}

impl WhitehallFile {
    pub fn new() -> Self {
        WhitehallFile {
            imports: Vec::new(),
            props: Vec::new(),
            state: Vec::new(),
            functions: Vec::new(),
            lifecycle_hooks: Vec::new(),
            classes: Vec::new(),
            markup: Markup::Text(String::new()),
        }
    }
}

impl Default for WhitehallFile {
    fn default() -> Self {
        Self::new()
    }
}
