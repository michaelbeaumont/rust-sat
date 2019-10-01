#[macro_use]
extern crate log;
extern crate bit_set;
extern crate env_logger;
extern crate vec_map;

use vec_map::VecMap;

use Lit::{N, P};
use Satness::SAT;

pub mod parse;

pub mod naive;
pub mod nonchro;
pub mod watch;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lit {
    P(Id),
    N(Id),
}

impl Lit {
    fn eval(&self, v: bool) -> bool {
        match *self {
            P(_) => v,
            N(_) => !v,
        }
    }

    pub fn id(&self) -> &Id {
        match *self {
            P(ref s) => s,
            N(ref s) => s,
        }
    }

    pub fn not(&self) -> Lit {
        match *self {
            P(ref s) => N(s.clone()),
            N(ref s) => P(s.clone()),
        }
    }

    pub fn as_usize(&self) -> usize {
        match *self {
            P(Id(id)) => id * 2,
            N(Id(id)) => id * 2 + 1,
        }
    }
}

pub type Clause = Vec<Lit>;

pub type CNF = Vec<Clause>;

type Map<T> = VecMap<T>;

#[derive(Debug, Clone)]
pub struct Interp(Map<bool>);

impl Interp {
    pub fn new() -> Interp {
        Interp(Map::new())
    }

    pub fn with_capacity(capacity: usize) -> Interp {
        Interp(Map::with_capacity(capacity))
    }

    pub fn get_val(&self, lit: &Lit) -> Option<bool> {
        let &Id(id) = lit.id();
        match *self {
            Interp(ref l) => l.get(id).map(|&b| lit.eval(b)),
        }
    }

    pub fn set_true(&mut self, lit: &Lit) {
        let &Id(id) = lit.id();
        match *self {
            Interp(ref mut l) => l.insert(id, lit.eval(true)),
        };
    }
}

pub fn check_clause(cls: &Clause, interp: &Interp) -> bool {
    cls.iter().fold(false, |acc, next| {
        let truth = interp.get_val(next).unwrap();
        acc || truth
    })
}

pub fn check(form: &CNF, interp: &Interp) -> bool {
    form.iter().fold(true, |acc, next| {
        let truth = check_clause(next, interp);
        acc && truth
    })
}

#[derive(Debug)]
pub enum Satness {
    SAT(Interp),
    UNSAT(String),
}

impl Satness {
    pub fn is_sat(&self) -> bool {
        match *self {
            SAT(_) => true,
            _ => false,
        }
    }
}

pub trait SATSolver {
    fn create(formula: CNF, interp: Option<Interp>) -> Self;
    fn solve(&mut self) -> Satness;
}

#[cfg(test)]
mod tests {
    use super::Lit::{N, P};
    use super::{check, Id, Interp};

    #[test]
    fn test_check() {
        let mut interp = Interp::new();
        let cnf = vec![vec![P(Id(1)), N(Id(1))], vec![P(Id(2))]];
        interp.set_true(&cnf[0][0]);
        interp.set_true(&cnf[1][0]);
        assert_eq!(check(&cnf, &interp), true);
        interp.set_true(&cnf[1][0].not());
        assert_eq!(check(&cnf, &interp), false);
    }
}
