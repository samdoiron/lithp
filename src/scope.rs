use std::fmt::Display;
use std::cell::RefCell;
use std::rc::Rc;

pub type ScopeRef<V> = Rc<RefCell<Scope<V>>>;   

#[derive(Debug, Clone)]
pub struct Scope<V> {
    pub parent: Option<Rc<RefCell<Scope<V>>>>,
    bindings: Vec<(String, V)>
}

impl<V: Display + Clone> Scope<V> {
    pub fn new() -> Scope<V> {
        Scope{parent: None, bindings: Vec::new()}
    }

    pub fn get(&self, name: &str) -> Option<V> {
        match self.get_local(name) {
            Some(value) => Some(value),
            None => self.parent.as_ref().and_then(|p| (*p).borrow().get(name))
        }
    }

    pub fn get_local(&self, target: &str) -> Option<V> {
        for i in (0..self.bindings.len()).rev() {
            let (ref name, ref value) = self.bindings[i];
            if name == target {
                return Some(value.clone());
            }
        }
        return None;
    }

    pub fn set_local(&mut self, name: &str, value: V) {
        self.bindings.push((name.to_string(), value));
    }

    pub fn set_inherited(&mut self, name: &str, value: V) {
        match self.get_local(name) {
            Some(_) => { self.set_local(name, value); },
            None => { self.parent.as_ref().map(|p| (*p).borrow_mut().set_inherited(name, value)); }
        }
    }
}

pub fn new_scope<V: Display + Clone>() -> ScopeRef<V> {
    Rc::new(RefCell::new(Scope::new()))
}

pub fn new_child_scope<V: Display + Clone>(scope: &ScopeRef<V>) -> ScopeRef<V> {
    let scope = Scope{parent: Some(scope.clone()), bindings: Vec::new()};
    Rc::new(RefCell::new(scope))
}