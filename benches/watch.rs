extern crate test;
extern crate sat;

use test::Bencher;

use std::io::{File, BufferedReader};

use sat::SATSolver;
use sat::watch::Solver;
use sat::parse;

fn get_test_file_reader() -> BufferedReader<File> {
    match File::open(&Path::new("./tests/uf20-0264.cnf")) {
        Ok(f) => {
            BufferedReader::new(f)
        },
        Err(e) => panic!("Couldn't open test file: {}",e)
    }
}

#[bench]
fn bench_parse(b: &mut Bencher) {
    match parse::parse(get_test_file_reader()) {
        Ok(cnf) => {
            b.iter(|| {
                   let mut solver: Solver = SATSolver::create(cnf.clone());
                   let solvable = solver.solve();
                   assert_eq!(solvable.is_sat(), true);
                   //println!("Parsed SAT Formula is {}", solve_watch(&mut solver))
            })}
                   _     => panic!("FAIL")
    }
}
