use atom::{Atom, Closure};
use scope::{ScopeRef};
use util::prepend;
use scope::{new_child_scope, new_scope};

const BUILT_INS: [&'static str; 12] = ["define", "+", "-", "*", "/", "cons",
                                       "car", "cdr", "list", "let", "let*",
                                       "lambda"];


pub fn eval(atom: Atom) -> Result<Atom, &'static str> {
    eval_atoms(new_scope(), atom)
}

fn eval_atoms(scope: ScopeRef<Atom>, atom: Atom) -> Result<Atom, &'static str> {
    let result = match atom {
        Atom::List(atoms) => {
            let mut evaluated = Vec::with_capacity(atoms.len());
            for atom in atoms {
                evaluated.push(try!(eval_atom(scope.clone(), atom)));
            }
            match evaluated.last() {
                Some(value) => Ok(value.clone()),
                None => Err("eval atoms on empty list")
            }
        },
        _ => Err("eval_atoms must be called with atom list")
    };
    result
}

fn eval_atom(scope: ScopeRef<Atom>, atom: Atom) -> Result<Atom, &'static str> {
    let original = atom.clone();
    let result = match atom {
        Atom::Quoted(value) => Ok(*value),
        Atom::Integer(_) | Atom::Lambda(_) => Ok(atom),
        Atom::Identifier(ref name) => try_get(scope, name),
        Atom::List(atoms) => {
            match atoms.split_first() {
                // Macros / syntax rules, which have special evaluation
                Some((&Atom::Identifier(ref x), cdr)) if x == "let" => eval_let(scope, cdr),
                Some((&Atom::Identifier(ref x), cdr)) if x == "let*" => eval_let_star(scope, cdr),
                Some((&Atom::Identifier(ref x), cdr)) if x == "define" => eval_define(scope, cdr),
                Some((&Atom::Identifier(ref x), cdr)) if x == "set!" => eval_set(scope, cdr),
                Some((&Atom::Identifier(ref x), cdr)) if x == "lambda" => eval_lambda(scope, cdr),
                _ => {
                    let mut evaluated = Vec::with_capacity(atoms.len());
                    for atom in atoms.clone() {
                        evaluated.push(try!(eval_atom(scope.clone(), atom)));
                    }
                    match evaluated.split_first() {
                        Some((car, cdr)) => apply(&car, &cdr),
                        None => Err("invalid empty expression")
                    }
                }
            }
        },
    };
    if let &Ok(ref evaluated) = &result {
        println!("eval( {} ) -> {}", original, evaluated)
    }
    result
}

fn eval_let(scope: ScopeRef<Atom>, cdr: &[Atom]) -> Result<Atom, &'static str> {
    println!("eval( let ) -> let");
    let (binding_list, expressions) = try!(split_let_body(cdr));
    let new_scope = new_child_scope(&scope);
    let bindings = try!(extract_bindings(binding_list.clone()));
    for (name, expression) in bindings {
        let value = try!(eval_atom(scope.clone(), expression));
        new_scope.borrow_mut().set_local(&name, value);
    }
    
    let result = eval_atoms(new_scope, Atom::List(expressions.to_vec()));
    result
}

fn eval_let_star(scope: ScopeRef<Atom>, cdr: &[Atom]) -> Result<Atom, &'static str> {
    println!("eval( let* ) -> let*");
    let (binding_list, expressions) = try!(split_let_body(cdr));
    let new_scope = new_child_scope(&scope);
    let bindings = try!(extract_bindings(binding_list.clone()));
    for (name, expression) in bindings {
        let value = try!(eval_atom(new_scope.clone(), expression));
        new_scope.borrow_mut().set_local(&name, value);
    }

    let result = eval_atoms(new_scope, Atom::List(expressions.to_vec()));
    result   
}

fn eval_define(scope: ScopeRef<Atom>, cdr: &[Atom]) -> Result<Atom, &'static str> {
    println!("eval( define ) -> define");
    if cdr.len() != 2 { return Err("wrong number of arguments for define") }
    match cdr[0] {
        Atom::Identifier(ref name) => {
            let evaluated = try!(eval_atom(scope.clone(), cdr[1].clone()));
            scope.borrow_mut().set_local(&name, evaluated);
            Ok(Atom::Identifier("".to_string()))
        },
        _ => Err("first param of define must be an identifier")
    }
}

fn eval_set(scope: ScopeRef<Atom>, cdr: &[Atom]) -> Result<Atom, &'static str> {
    println!("eval( set! ) -> set!");
    if cdr.len() != 2 { return Err("wrong number of arguments for set! ")}
    match cdr[0] {
        Atom::Identifier(ref name) => {
            let existing = { scope.borrow().get(name) };
            match existing {
                Some(old_value) => {
                    let evaluated = try!(eval_atom(scope.clone(), cdr[1].clone()));
                    scope.borrow_mut().set_inherited(&name, evaluated);
                    Ok(old_value)
                },
                None => Err("attempt to set! undefined value")
            }
        },
        _ => Err("first parameter of set! must be an identifier")
    }
}

macro_rules! extract {
    ( $t:path, $cdr:expr ) => {{
        let cdr = $cdr;
        let mut result = Vec::with_capacity(cdr.len());
        for atom in cdr {
            match atom {
                &$t(ref val) => result.push(val.clone()),
                _ => return Err("invalid format")
            }
        }
        result
    }}
}

fn eval_lambda(scope: ScopeRef<Atom>, cdr: &[Atom]) -> Result<Atom, &'static str> {
    println!("eval( lambda ) -> lambda");
    match cdr.split_first() {
        Some((&Atom::List(ref params), ref body)) if !body.is_empty() => {
            let param_names = extract!(Atom::Identifier, params);
            Ok(Atom::Lambda(Closure{
                scope: new_child_scope(&scope),
                parameters: param_names,
                body: Box::new(Atom::List(body.to_vec()))
            }))
        },
        _ => Err("invalid lambda")
    }
}

fn try_get(scope: ScopeRef<Atom>, name: &str) -> Result<Atom, &'static str> {
    match scope.borrow().get(name) {
        Some(atom) => Ok(atom),
        None => {
            if BUILT_INS.contains(&name) {
                Ok(Atom::Identifier(name.to_string()))
            } else {
                Err("unknown identifier")
            }
        }
    }
}

fn apply(func: &Atom, args: &[Atom]) -> Result<Atom, &'static str> {
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
                _ => Err("unknown function")
            }
        },
        &Atom::Lambda(ref closure) => apply_closure(closure, args),
        _ => Err("cannot apply value of given type")
    }
}

fn apply_closure(closure: &Closure, args: &[Atom]) -> Result<Atom, &'static str> {
    if closure.parameters.len() != args.len() { 
        return Err("invalid closure arity")
    }
    
    let call_scope = new_child_scope(&closure.scope);
    for (i, arg) in args.iter().enumerate() {
        call_scope.borrow_mut().set_local(&closure.parameters[i], arg.clone());
    }
    return eval_atoms(call_scope, *closure.body.clone());
}

fn car(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 1 { return Err("wrong number of args to car") }
    match &cdr[0] {
        &Atom::List(ref atoms) => Ok(atoms[0].clone()),
        _ => Err("invalid argument to car")
    }
}

fn cdr(cdr: &[Atom]) -> Result<Atom, &'static str> {
    match cdr.first() {
        Some(&Atom::List(ref items)) if !items.is_empty() => {
            Ok(Atom::List(items[1..].to_vec()))
        },
        _ => Err("invalid cdr param")
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

fn math(start: i64, reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
    -> Result<Atom, &'static str> {
    if cdr.len() == 0 { return Err("attempted math on empty list") }
    let ints = extract!(Atom::Integer, cdr);
    Ok(Atom::Integer(ints.iter().fold(start, reduce)))
}

fn math_first(reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom]) 
    -> Result<Atom, &'static str> {
    let ints = extract!(Atom::Integer, cdr);
    if !ints.is_empty() {
        Ok(Atom::Integer(ints[1..].iter().fold(ints[0], reduce)))
    } else {
        Err("attempted math on empty list")
    }
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
    match atom {
        Atom::List(ref binding) if binding.len() == 2 => {
            match (&binding[0], &binding[1]) {
                (&Atom::Identifier(ref name), value) => Ok((name.clone(), value.clone())),
                _ => Err("binding must start with an identifier")
            }
        },
        _ => Err("invalid binding")
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