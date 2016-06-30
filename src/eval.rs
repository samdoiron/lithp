use atom::Atom;
use util::prepend;

pub fn eval(atom: Atom) -> Result<Atom, &'static str> {
    match atom {
        Atom::Quoted(value) => Ok(*value),
        Atom::Integer(_) | Atom::Identifier(_) => Ok(atom),
        Atom::List(atoms) => {
            match atoms.split_first() {
                Some((car, cdr)) => apply(car, cdr),
                None => Err("eval() on empty list")
            }
        }
    }
}

fn apply(car: &Atom, cdr: &[Atom]) -> Result<Atom, &'static str> {
    match car {
        &Atom::Identifier(ref name) => {
            let name_ref: &str = name;
            match name_ref {
                "+" => apply_math(0, &|a, &b| a + b, cdr),
                "*" => apply_math(1, &|a, &b| a * b, cdr),
                "/" => apply_math_first(&|a, &b| a / b, cdr),
                "-" => apply_math_first(&|a, &b| a - b, cdr),
                "car" => apply_car(cdr),
                "cdr" => apply_cdr(cdr),
                "cons" => apply_cons(cdr),
                "list" => apply_list(cdr),
                _ => Err("unknown function")
            }
        },
        _ => Err("cannot apply non-identifier")
    }
}

fn apply_math(start: i64, reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
    -> Result<Atom, &'static str> {
    if cdr.len() == 0 { return Err("attempted math on empty list") }
    match extract_ints(cdr) {
        Some(ints) => Ok(Atom::Integer(ints.iter().fold(start, reduce))),
        None => Err("attempted math on non-integer")
    }
}

fn apply_math_first(reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
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
    let mut result = Vec::new();
    for atom in cdr {
        match atom {
            &Atom::Integer(val) => result.push(val),
            _ => return None
        }
    }
    Some(result)
}

fn apply_car(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 1 { return Err("wrong number of args to car") }
    match &cdr[0] {
        &Atom::List(ref atoms) => Ok(atoms[0].clone()),
        _ => Err("invalid argument to car")
    }
}

fn apply_cdr(cdr: &[Atom]) -> Result<Atom, &'static str> {
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

fn apply_cons(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 2 { return Err("wrong number of args for cons ") }
    match &cdr[1] {
        &Atom::List(ref vals) => Ok(Atom::List(prepend(cdr[0].clone(), &mut vals.clone()))),
        _ => Err("invalid type to cons() onto")
    }
}

fn apply_list(cdr: &[Atom]) -> Result<Atom, &'static str> {
    Ok(Atom::List(cdr.to_vec()))
}
