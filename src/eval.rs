use atom::Atom;
use scope::Scope;
use util::prepend;

const BUILT_INS: [&'static str; 8] = ["+", "*", "/", "-", "car", "cdr", 
                                      "cons", "list"];

pub struct Eval<'a> {
    scope: Scope<'a, Atom>
}

impl<'a> Eval<'a> {
    pub fn new() -> Eval<'a> {
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
            _ => unreachable!()
        }
    }

    fn eval_atom(&mut self, atom: Atom) -> Result<Atom, &'static str> {
        match atom {
            Atom::Quoted(value) => Ok(*value),
            Atom::Integer(_) => Ok(atom),
            Atom::Identifier(ref name) => self.try_get(name),
            Atom::List(atoms) => {
                match atoms.split_first() {
                    Some((car, cdr)) => {
                        let mut evaluated_cdr = Vec::with_capacity(cdr.len());
                        for atom in cdr {
                            evaluated_cdr.push(try!(self.eval_atom(atom.clone())));
                        }
                        self.apply(car, &evaluated_cdr)
                    }
                    None => Ok(Atom::List(vec![]))
                }
            }
        }
    }

    fn try_get(&self, name: &str) -> Result<Atom, &'static str> {
        if BUILT_INS.contains(&name) {
            return Ok(Atom::Identifier(name.to_string()));
        }
        match self.scope.get(name) {
            Some(atom) => Ok(atom.clone()),
            None => Ok(Atom::Identifier(name.to_string()))
        }
    }

    fn apply(&mut self, car: &Atom, cdr: &[Atom]) -> Result<Atom, &'static str> {
        match car {
            &Atom::Identifier(ref name) => {
                let name_ref: &str = name;
                match name_ref {
                    "+" => self.apply_math(0, &|a, &b| a + b, cdr),
                    "*" => self.apply_math(1, &|a, &b| a * b, cdr),
                    "/" => self.apply_math_first(&|a, &b| a / b, cdr),
                    "-" => self.apply_math_first(&|a, &b| a - b, cdr),
                    "car" => self.apply_car(cdr),
                    "cdr" => self.apply_cdr(cdr),
                    "cons" => self.apply_cons(cdr),
                    "list" => self.apply_list(cdr),
                    _ => Err("unknown function")
                }
            },
            other => {
                println!("car is {:?}", car);
                Err("cannot apply non-identifier")
            }
        }
    }

    fn apply_math(&self, start: i64, reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
        -> Result<Atom, &'static str> {
        if cdr.len() == 0 { return Err("attempted math on empty list") }
        match self.extract_ints(cdr) {
            Some(ints) => Ok(Atom::Integer(ints.iter().fold(start, reduce))),
            None => Err("attempted math on non-integer")
        }
    }

    fn apply_math_first(&self, reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
        -> Result<Atom, &'static str> {
        match self.extract_ints(cdr) {
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

    fn extract_ints(&self, cdr: &[Atom]) -> Option<Vec<i64>> {
        let mut result = Vec::new();
        for atom in cdr {
            match atom {
                &Atom::Integer(val) => result.push(val),
                _ => return None
            }
        }
        Some(result)
    }

    fn apply_car(&self, cdr: &[Atom]) -> Result<Atom, &'static str> {
        if cdr.len() != 1 { return Err("wrong number of args to car") }
        match &cdr[0] {
            &Atom::List(ref atoms) => Ok(atoms[0].clone()),
            _ => Err("invalid argument to car")
        }
    }

    fn apply_cdr(&self, cdr: &[Atom]) -> Result<Atom, &'static str> {
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

    fn apply_cons(&self, cdr: &[Atom]) -> Result<Atom, &'static str> {
        if cdr.len() != 2 { return Err("wrong number of args for cons ") }
        match &cdr[1] {
            &Atom::List(ref vals) => Ok(Atom::List(prepend(cdr[0].clone(), &mut vals.clone()))),
            _ => Err("invalid type to cons() onto")
        }
    }

    fn apply_list(&self, cdr: &[Atom]) -> Result<Atom, &'static str> {
        Ok(Atom::List(cdr.to_vec()))
    }

}
