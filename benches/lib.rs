use criterion::{criterion_group, criterion_main, Criterion};
use glob::glob;
use sat::{check, naive, nonchro, parse, watch, SATSolver, Satness};
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn path_to_string(path: &Path) -> std::io::Result<String> {
    let file = File::open(path);
    let mut s = String::new();
    file.and_then(|mut f| f.read_to_string(&mut s))?;
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
                        Satness::SAT(interp) => assert_eq!(check(&cnf, &interp), true),
                    }
                }
            }
            _ => panic!("Error parsing file"),
        }
    }
}

pub fn bench_sat<Solver: SATSolver>() {
    let path20 = "tests/uf20-91/*.cnf";
    let _path50 = "tests/uf50-218/*.cnf";
    let _path100 = "tests/uf100-430/uf100-010.cnf";
    let _path125 = "tests/uf125-538/uf125-010.cnf";
    let _path150 = "tests/uf150-645/uf150-010.cnf";
    test_solve_file::<Solver>(path20, true)
}

pub fn bench_unsat<Solver: SATSolver>() {
    let path50 = "tests/uuf50-218/uf50-020.cnf";
    test_solve_file::<Solver>(path50, false)
}

fn bench_naive(c: &mut Criterion) {
    c.bench_function("naive - sat", |b| b.iter(|| bench_sat::<naive::Solver>()))
        .bench_function("naive - unsat", |b| {
            b.iter(|| bench_unsat::<naive::Solver>())
        });
}

fn bench_nonchro(c: &mut Criterion) {
    c.bench_function("nonchro - sat", |b| {
        b.iter(|| bench_sat::<nonchro::Solver>())
    })
    .bench_function("nonchro - unsat", |b| {
        b.iter(|| bench_unsat::<nonchro::Solver>())
    });
}

fn bench_watch(c: &mut Criterion) {
    c.bench_function("watch - sat", |b| b.iter(|| bench_sat::<watch::Solver>()))
        .bench_function("watch - unsat", |b| {
            b.iter(|| bench_unsat::<watch::Solver>())
        });
}

criterion_group!(benches_naive, bench_naive);
criterion_group!(benches_nonchro, bench_nonchro);
criterion_group!(benches_watch, bench_watch);
criterion_main!(benches_naive, benches_nonchro, benches_watch);
