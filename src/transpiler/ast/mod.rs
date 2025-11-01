/// Abstract Syntax Tree for Whitehall

#[derive(Debug, Clone, PartialEq)]
pub struct WhitehallFile {
    pub imports: Vec<Import>,
    pub props: Vec<PropDeclaration>,
    pub state: Vec<StateDeclaration>,
    pub functions: Vec<Function>,
    pub derived_state: Vec<DerivedState>,
    pub lifecycle: Vec<LifecycleHook>,
    pub markup: Markup,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String, // e.g., "$lib.api.ApiClient" or "androidx.compose.runtime.*"
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
    pub mutable: bool, // var vs val
    pub state_type: Option<String>,
    pub initial_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DerivedState {
    pub name: String,
    pub state_type: Option<String>,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleHook {
    OnMount { body: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Markup {
    Component(Component),
    Text(String),
    Interpolation(String),
    Sequence(Vec<Markup>),
    ControlFlowIf(ControlFlowIf),
    ControlFlowFor(ControlFlowFor),
    ControlFlowWhen(ControlFlowWhen),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub props: Vec<Prop>,
    pub children: Vec<Markup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prop {
    pub name: String,
    pub value: PropValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    String(String),
    Expression(String),
    Binding(String), // for bind:value={expr}
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlowIf {
    pub condition: String,
    pub then_branch: Vec<Markup>,
    pub else_if_branches: Vec<(String, Vec<Markup>)>, // condition, body
    pub else_branch: Option<Vec<Markup>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlowFor {
    pub item: String,
    pub collection: String,
    pub key: Option<String>, // lambda expression
    pub body: Vec<Markup>,
    pub empty: Option<Vec<Markup>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlowWhen {
    pub branches: Vec<WhenBranch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenBranch {
    pub condition: Option<String>, // None for 'else' branch
    pub body: Vec<Markup>,
}

impl WhitehallFile {
    pub fn new() -> Self {
        WhitehallFile {
            imports: Vec::new(),
            props: Vec::new(),
            state: Vec::new(),
            functions: Vec::new(),
            derived_state: Vec::new(),
            lifecycle: Vec::new(),
            markup: Markup::Sequence(Vec::new()),
        }
    }
}

impl Default for WhitehallFile {
    fn default() -> Self {
        Self::new()
    }
}
