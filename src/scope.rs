use std::rc::Rc;

#[derive(Debug)]
struct Binding<V>{ name: String, value: V }

#[derive(Debug)]
pub struct Scope<'a, V: 'a> {
    pub parent: Option<Rc<&'a Scope<'a, V>>>,
    bindings: Vec<Binding<V>>
}

const SCOPE_SIZE_ESTIMATE: usize = 10;

impl<'a, V> Scope<'a, V> {
    pub fn new() -> Scope<'a, V> {
        Scope{parent: None, bindings: Vec::with_capacity(SCOPE_SIZE_ESTIMATE)}
    }

    pub fn new_child(&'a self) -> Scope<'a, V> {
        Scope{parent: Some(Rc::new(self)),
              bindings: Vec::with_capacity(SCOPE_SIZE_ESTIMATE)}
    }

    pub fn get(&self, name: &str) -> Option<&V> {
        self.get_local(name).or_else(|| self.get_from_parent(name))
    }

    pub fn set(&mut self, name: String, value: V) {
        self.bindings.push(Binding{name: name, value: value});
    }

    fn get_local(&self, name: &str) -> Option<&V> {
        self.bindings.iter().find(|b| b.name == name).map(|b| &b.value)
    }

    fn get_from_parent(&self, name: &str) -> Option<&V> {
        self.parent.as_ref().and_then(|p| p.get(name))
    }
}