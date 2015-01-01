#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

use std::collections::HashMap;

use Lit::{P, N};
use Satness::{SAT};

pub mod parse;

pub mod naive;
pub mod watch;

pub trait Negable {
    fn not(&self) -> Self;
}

#[deriving(Show, Clone, PartialEq, Eq, Hash)]
pub enum Lit {
    P(String),
    N(String)
}

impl<'a> Lit {
    fn get_truth_as(&self, v: bool) -> bool {
        match *self {
            P(_) => v,
            N(_) => !v
        }
    }

    pub fn id(&'a self) -> &'a str {
        match *self {
            P(ref s) => s.as_slice(),
            N(ref s) => s.as_slice()
        }
    }
}

impl Negable for Lit {
    fn not(&self) -> Lit {
        match *self {
            P(ref s) => N(s.clone()),
            N(ref s) => P(s.clone())
        }
    }
}


pub type Clause = Vec<Lit>;

pub type CNF = Vec<Clause>;

#[deriving(Show, Clone, PartialEq, Eq)]
pub struct Interp(HashMap<String, bool>);

impl Interp {
    pub fn get_val(&self, lit: &Lit) -> Option<&bool> {
        match *self {
            Interp(ref l) => l.get(&lit.id().to_string())
        }
    }
    pub fn assign_true(&mut self, lit: &Lit) {
        let _ = match *self {
            Interp(ref mut l) => l.insert(lit.id().to_string(), lit.get_truth_as(true))
        };
    }
}
#[deriving(Show)]
pub enum Satness {
    SAT(Interp),
    UNSAT(String)
}

impl Satness {
    pub fn is_sat(&self) -> bool {
        match *self {
            SAT(_) => true,
            _   => false
        }
    }
}

pub trait SATSolver {
    fn create(formula: CNF) -> Self;
    fn solve(&mut self) -> Satness;
}
