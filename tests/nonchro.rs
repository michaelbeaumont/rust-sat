extern crate sat;

use sat::nonchro::Solver;

use satsolver;

#[test]
fn test_solve_simple() {
    satsolver::test_solve_simple::<Solver>()
}

#[test]
fn test_solve_sat() {
    satsolver::test_solve_sat::<Solver>()
}

#[test]
fn test_solve_unsat() {
    satsolver::test_solve_unsat::<Solver>()
}
