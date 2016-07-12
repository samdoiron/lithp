use std::collections::HashMap;
use std::fmt::Display;
use std::cell::RefCell;
use std::rc::Rc;


pub type ScopeRef<V> = Rc<RefCell<Scope<V>>>;   

#[derive(Debug, Clone)]
pub struct Scope<V> {
    pub parent: Option<Rc<RefCell<Scope<V>>>>,
    bindings: HashMap<String, V>
}

impl<V: Display + Clone> Scope<V> {
    pub fn new() -> Scope<V> {
        Scope{parent: None, bindings: HashMap::new()}
    }

    pub fn get(&self, name: &str) -> Option<V> {
        match self.bindings.get(name) {
            Some(value) => Some(value.clone()),
            None => self.parent.as_ref().and_then(|p| (*p).borrow().get(name))
        }
    }

    pub fn set_local(&mut self, name: &str, value: V) {
        self.bindings.insert(name.to_string(), value);
    }

    pub fn set_inherited(&mut self, name: &str, value: V) {
        match self.bindings.get(name) {
            Some(_) => { self.bindings.insert(name.to_string(), value); },
            None => { self.parent.as_ref().map(|p| (*p).borrow_mut().set_inherited(name, value)); }
        }
    }
}

pub fn new_scope<V: Display + Clone>() -> ScopeRef<V> {
    Rc::new(RefCell::new(Scope::new()))
}

pub fn new_child_scope<V: Display + Clone>(scope: &ScopeRef<V>) -> ScopeRef<V> {
    let scope = Scope{parent: Some(scope.clone()), bindings: HashMap::new()};
    Rc::new(RefCell::new(scope))
}