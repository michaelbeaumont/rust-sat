extern crate test;

extern crate sat;

use test::Bencher;

use sat::watch::Solver;

#[bench]
fn bench_sat(b: &mut Bencher) {
    b.iter(|| super::bench_sat::<Solver>())
}

#[bench]
fn bench_unsat(b: &mut Bencher) {
    b.iter(|| super::bench_unsat::<Solver>())
}