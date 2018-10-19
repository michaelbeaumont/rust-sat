#![feature(test)]
extern crate test;

extern crate sat;

use test::Bencher;

use sat::watch::Solver;
mod benches;

#[bench]
fn bench_sat(b: &mut Bencher) {
    b.iter(|| benches::bench_sat::<Solver>())
}

#[bench]
fn bench_unsat(b: &mut Bencher) {
    b.iter(|| benches::bench_unsat::<Solver>())
}
