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

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Atom::Identifier(ref a), &Atom::Identifier(ref b)) => a == b,
            (&Atom::Integer(a), &Atom::Integer(b)) => a == b,
            (&Atom::List(ref a), &Atom::List(ref b)) => a == b,
            (&Atom::Quoted(ref a), &Atom::Quoted(ref b)) => a == b,
            _ => false
        }
    }
}