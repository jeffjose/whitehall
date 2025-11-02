/// Abstract Syntax Tree for Whitehall

#[derive(Debug, Clone, PartialEq)]
pub struct WhitehallFile {
    pub imports: Vec<Import>,
    pub props: Vec<PropDeclaration>,
    pub state: Vec<StateDeclaration>,
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
    pub mutable: bool, // var vs val
    pub initial_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Markup {
    Component(Component),
    Text(String),
    Interpolation(String), // {variable} expression
    Sequence(Vec<Markup>), // Multiple markup items
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub props: Vec<ComponentProp>,
    pub children: Vec<Markup>,
    pub self_closing: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentProp {
    pub name: String,
    pub value: String, // Expression value
}

impl WhitehallFile {
    pub fn new() -> Self {
        WhitehallFile {
            imports: Vec::new(),
            props: Vec::new(),
            state: Vec::new(),
            markup: Markup::Text(String::new()),
        }
    }
}

impl Default for WhitehallFile {
    fn default() -> Self {
        Self::new()
    }
}
