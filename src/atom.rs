use scope::{ScopeRef};

#[derive(Debug, Clone)]
pub enum Atom {
    List(Vec<Atom>),
    Integer(i64),
    Identifier(String),
    Quoted(Box<Atom>),
    Lambda(Closure)
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub scope: ScopeRef<Atom>,
    pub parameters: Vec<String>, 
    pub body: Box<Atom>
}