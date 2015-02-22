#![feature(test)]
extern crate test;
extern crate glob;

extern crate sat;

use std::io::Read;
use std::path::Path;
use std::fs::File;

use glob::glob;

use sat::{Satness, check, SATSolver};

use sat::parse;

mod watch;
mod naive;
mod nonchro;

fn path_to_string(path: &Path) -> std::io::Result<String> {
    let file = File::open(path);
    let mut s = String::new();
    try!(file.and_then(|mut f| f.read_to_string(&mut s)));
    Ok(s)
}

pub fn test_solve_file<Solver: SATSolver>(path: &str, sat: bool) {
    for path in glob(path).unwrap() {
        let s: String = path_to_string(&path.unwrap()).unwrap();
        match parse::parse_file(s) {
            Ok(cnf) => {
                let mut solver: Solver = SATSolver::create(cnf.clone(), None);
                let solvable = solver.solve();
                assert_eq!(solvable.is_sat(), sat);
                if sat {
                    match solvable {
                        Satness::UNSAT(_) => panic!("UNSAT"),
                        Satness::SAT(interp) => assert_eq!(check(&cnf, &interp), true)
                    }
                }
            },
            _     => panic!("Error parsing file")
        }
    }
}

pub fn bench_sat<Solver: SATSolver>() {
    let path20 = "tests/uf20-91/*.cnf";
    let path50 = "tests/uf50-218/*.cnf";
    let path100 = "tests/uf100-430/uf100-010.cnf";
    let path125 = "tests/uf125-538/uf125-010.cnf";
    let path150 = "tests/uf150-645/uf150-010.cnf";
    test_solve_file::<Solver>(path20, true)
}

pub fn bench_unsat<Solver: SATSolver>() {
    let path50 = "tests/uuf50-218/uf50-020.cnf";
    test_solve_file::<Solver>(path50, false)
}
