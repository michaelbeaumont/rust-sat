#![feature(phase)]
#[phase(plugin, link)] extern crate log;

use std::collections::HashMap;

use Lit::{P, N};
use Satness::{SAT};

pub mod naive;
pub mod parse;

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

pub type Interp = HashMap<String, bool>;

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
