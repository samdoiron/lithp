use atom::Atom;
use scope::Scope;
use util::prepend;
use std::mem;

const BUILT_INS: [&'static str; 10] = ["define", "+", "-", "*", "/", "cons",
                                       "car", "cdr", "list", "let"];

pub struct Eval<> {
    scope: Scope<Atom>
}

impl Eval {
    pub fn new() -> Eval {
        Eval{scope: Scope::new()}
    }

    pub fn eval_atoms(&mut self, atom: Atom) -> Result<Atom, &'static str> {
        match atom {
            Atom::List(atoms) => {
                match atoms.into_iter().map(|a| self.eval_atom(a)).last() {
                    Some(Ok(value)) => Ok(value),
                    Some(Err(message)) => Err(message),
                    None => Err("eval atoms on empty list")
                }
            },
            _ => Err("eval_atoms must be called with atom list")
        }
    }

    fn eval_atom(&mut self, atom: Atom) -> Result<Atom, &'static str> {
        match atom {
            Atom::Quoted(value) => Ok(*value),
            Atom::Integer(_) => Ok(atom),
            Atom::Identifier(ref name) => self.try_get(name),
            Atom::List(atoms) => {
                match atoms.split_first() {
                    Some((&Atom::Identifier(ref x), cdr)) if x == "let" => self.eval_let(cdr),
                    Some((&Atom::Identifier(ref x), cdr)) if x == "let*" => self.eval_let_star(cdr),
                    Some((car, cdr)) => {
                        let mut evaluated_cdr = Vec::with_capacity(cdr.len());
                        for atom in cdr {
                            evaluated_cdr.push(try!(self.eval_atom(atom.clone())));
                        }
                        apply(&mut self.scope, car, &evaluated_cdr)
                    }
                    None => Ok(Atom::List(vec![]))
                }
            }
        }
    }

    fn eval_let(&mut self, cdr: &[Atom]) -> Result<Atom, &'static str> {
        let (binding_list, expressions) = try!(split_let_body(cdr));
        let mut new_scope = Scope::new();
        let bindings = try!(extract_bindings(binding_list.clone()));
        for (name, expression) in bindings {
            let value = try!(self.eval_atom(expression));
            new_scope.set(name, value);
        }
        
        self.push_scope(new_scope);
        let result = self.eval_atoms(Atom::List(expressions.to_vec()));
        self.pop_scope();
        result
    }

    fn eval_let_star(&mut self, cdr: &[Atom]) -> Result<Atom, &'static str> {
        let (binding_list, expressions) = try!(split_let_body(cdr));
        self.push_scope(Scope::new());
        let bindings = try!(extract_bindings(binding_list.clone()));
        for (name, expression) in bindings {
            let value = try!(self.eval_atom(expression));
            self.scope.set(name, value);
        }

        let result = self.eval_atoms(Atom::List(expressions.to_vec()));
        self.pop_scope();
        result   
    }

    fn push_scope(&mut self, new_scope: Scope<Atom>) {
        let old_scope = mem::replace(&mut self.scope, new_scope);
        self.scope.parent = Some(Box::new(old_scope));
    }

    fn pop_scope(&mut self) {
        let maybe_old_scope = mem::replace(&mut self.scope.parent, None);
        let old_scope = *maybe_old_scope.unwrap();
        mem::replace(&mut self.scope, old_scope);
    }

    fn try_get(&self, name: &str) -> Result<Atom, &'static str> {
        if BUILT_INS.contains(&name) {
            return Ok(Atom::Identifier(name.to_string()));
        }
        match self.scope.get(name) {
            Some(atom) => Ok(atom.clone()),
            None => Err("unknown identifier")
        }
    }

}

fn apply(scope: &mut Scope<Atom>, func: &Atom, args: &[Atom]) -> Result<Atom, &'static str> {
    match func {
        &Atom::Identifier(ref name) => {
            let name_ref: &str = name;
            match name_ref {
                "+" => math(0, &|a, &b| a + b, args),
                "*" => math(1, &|a, &b| a * b, args),
                "/" => math_first(&|a, &b| a / b, args),
                "-" => math_first(&|a, &b| a - b, args),
                "car" => car(args),
                "cdr" => cdr(args),
                "cons" => cons(args),
                "list" => list(args),
                "define" => define(scope, args),
                _ => Err("unknown function")
            }
        },
        _ => Err("cannot apply non-identifier")
    }
}

fn car(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 1 { return Err("wrong number of args to car") }
    match &cdr[0] {
        &Atom::List(ref atoms) => Ok(atoms[0].clone()),
        _ => Err("invalid argument to car")
    }
}

fn cdr(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if let Some(&Atom::List(ref items)) = cdr.first() {
        if items.len() != 0 {
            Ok(Atom::List(items[1..].to_vec()))
        } else {
            Err("cdr on empty list")
        }
    } else {
        Err("cdr invalid param")
    }
}

fn cons(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 2 { return Err("wrong number of args for cons ") }
    match &cdr[1] {
        &Atom::List(ref vals) => Ok(Atom::List(prepend(cdr[0].clone(), &mut vals.clone()))),
        _ => Err("invalid type to cons() onto")
    }
}

fn list(cdr: &[Atom]) -> Result<Atom, &'static str> {
    Ok(Atom::List(cdr.to_vec()))
}

fn define(scope: &mut Scope<Atom>, cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 2 { return Err("wrong number of arguments for define") }
    match cdr[0] {
        Atom::Identifier(ref name) => {
            scope.set(name.clone(), cdr[1].clone() );
            Ok(Atom::List(vec![]))
        },
        _ => Err("first param of define must be an identifier")
    }
}

fn math(start: i64, reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
    -> Result<Atom, &'static str> {
    if cdr.len() == 0 { return Err("attempted math on empty list") }
    match extract_ints(cdr) {
        Some(ints) => Ok(Atom::Integer(ints.iter().fold(start, reduce))),
        None => Err("attempted math on non-integer")
    }
}

fn math_first(reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
    -> Result<Atom, &'static str> {
    match extract_ints(cdr) {
        Some(ints) => {
            if ints.len() == 0 {
                Err("attempted math on empty list")
            } else {
                Ok(Atom::Integer(ints[1..].iter().fold(ints[0], reduce)))
            }
        }
        None => Err("attempted math on non-integer")
    }
}

fn extract_ints(cdr: &[Atom]) -> Option<Vec<i64>> {
    let mut result = Vec::with_capacity(cdr.len());
    for atom in cdr {
        match atom {
            &Atom::Integer(val) => result.push(val),
            _ => return None
        }
    }
    Some(result)
}

fn extract_bindings(atom_list: Atom) -> Result<Vec<(String, Atom)>, &'static str> {
    let atoms = match atom_list {
        Atom::List(a) => a,
        _ => return Err("bindings must be in a list")
    };

    let mut bindings = Vec::with_capacity(atoms.len());
    for atom in atoms {
        bindings.push(try!(extract_binding(atom)));
    }
    Ok(bindings)
}

fn extract_binding(atom: Atom) -> Result<(String, Atom), &'static str> {
    if let Atom::List(binding) = atom {
        if binding.len() != 2 {
            return Err("binding must have length 2")
        }
        match (&binding[0], &binding[1]) {
            (&Atom::Identifier(ref name), value) => Ok((name.clone(), value.clone())),
            _ => Err("binding must start with an identifier")
        }
    } else {
        Err("binding must be a list")
    }
}

fn split_let_body(cdr: &[Atom]) -> Result<(&Atom, &[Atom]), &'static str> {
match cdr.split_first() {
    Some((binding_list, expressions))
        if expressions.len() >= 1 => Ok((binding_list, expressions)),
    Some(_) => Err("invalid let(*) format"),
    None => Err("empty let(*)")
}
}