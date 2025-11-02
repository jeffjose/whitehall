/// Abstract Syntax Tree for Whitehall

#[derive(Debug, Clone, PartialEq)]
pub struct WhitehallFile {
    pub state: Vec<StateDeclaration>,
    pub markup: Markup,
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
    pub children: Vec<Markup>,
}

impl WhitehallFile {
    pub fn new() -> Self {
        WhitehallFile {
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
