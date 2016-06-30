#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Atom {
    List(Vec<Atom>),
    Integer(i64),
    Identifier(String),
    Quoted(Box<Atom>)
}