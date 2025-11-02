/// Abstract Syntax Tree for Whitehall

#[derive(Debug, Clone, PartialEq)]
pub struct WhitehallFile {
    pub markup: Markup,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Markup {
    Component(Component),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub children: Vec<Markup>,
}

impl WhitehallFile {
    pub fn new() -> Self {
        WhitehallFile {
            markup: Markup::Text(String::new()),
        }
    }
}

impl Default for WhitehallFile {
    fn default() -> Self {
        Self::new()
    }
}
