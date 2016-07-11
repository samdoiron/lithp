use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug)]
pub struct Scope<V> {
    pub parent: Option<Box<Scope<V>>>,
    bindings: HashMap<String, V>
}

impl<V: Display + Clone> Scope<V> {
    pub fn new() -> Scope<V> {
        Scope{parent: None, bindings: HashMap::new()}
    }

    pub fn get(&self, name: &str) -> Option<V> {
        self.get_local(name).or_else(|| self.get_from_parent(name))
    }

    pub fn set(&mut self, name: String, value: V) {
        self.bindings.insert(name, value);
    }

    fn get_local(&self, name: &str) -> Option<V> {
        match self.bindings.get(name) {
            Some(value) => Some(value.to_owned()),
            None => None
        }
    }

    fn get_from_parent(&self, name: &str) -> Option<V> {
        self.parent.as_ref().and_then(|p| p.get(name))
    }
}