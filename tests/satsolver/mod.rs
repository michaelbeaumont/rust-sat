use glob::glob;
use sat::parse;
use sat::Lit::{N, P};
use sat::{check, Id, SATSolver, Satness};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn test_solve_simple<Solver: SATSolver>() {
    let cnf1 = vec![vec![P(Id(1)), N(Id(2))], vec![N(Id(1))]];
    println!("Test 1: {:?}", cnf1);
    let mut state1: Solver = SATSolver::create(cnf1, None);
    let ans1 = state1.solve();
    println!("{:?}", ans1);
    assert_eq!(ans1.is_sat(), true);

    let cnf2 = vec![vec![P(Id(1))], vec![N(Id(1))]];
    println!("Test 2: {:?}", cnf2);
    let mut state2: Solver = SATSolver::create(cnf2, None);
    let ans2 = state2.solve();
    println!("{:?}", ans2);
    assert_eq!(ans2.is_sat(), false);

    let cnf3 = vec![
        vec![N(Id(1)), P(Id(1))],
        vec![P(Id(1)), P(Id(2))],
        vec![P(Id(1)), N(Id(2))],
    ];
    println!("Test 3: {:?}", cnf3);
    let mut state3: Solver = SATSolver::create(cnf3, None);
    let ans3 = state3.solve();
    println!("{:?}", ans3);
    assert_eq!(ans3.is_sat(), true);

    let cnf4 = vec![
        vec![N(Id(1)), P(Id(1))],
        vec![P(Id(1)), P(Id(2)), P(Id(3))],
        vec![P(Id(1)), P(Id(2)), N(Id(3))],
        vec![P(Id(1)), N(Id(2)), P(Id(3))],
        vec![P(Id(1)), N(Id(2)), N(Id(3))],
    ];
    println!("Test 4: {:?}", cnf4);
    let mut state4: Solver = SATSolver::create(cnf4, None);
    let ans4 = state4.solve();
    println!("{:?}", ans4);
    assert_eq!(ans4.is_sat(), true);
}

fn path_to_string(path: &Path) -> std::io::Result<String> {
    let file = File::open(path);
    let mut s = String::new();
    file.and_then(|mut f| f.read_to_string(&mut s))?;
    Ok(s)
}

pub fn test_solve_file<Solver: SATSolver>(path: &str, sat: bool) {
    //for path in glob("tests/uf50-218/*.cnf").unwrap() {
    //for path in glob("tests/uf100-430/uf100-010.cnf").unwrap() {
    //for path in glob("tests/uf125-538/uf125-010.cnf").unwrap() {
    for path in glob(path).unwrap() {
        //for path in glob("tests/uf175-753/uf175-010.cnf").unwrap() {
        //for path in glob("tests/sat/uf20-0584.cnf").unwrap() {
        println!("{:?}", &path);
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

pub fn test_solve_sat<Solver: SATSolver>() {
    let path20 = "tests/uf20-91/*.cnf";
    let _path125 = "tests/uf125-538/uf125-010.cnf";
    let _path150 = "tests/uf150-645/uf150-010.cnf";
    test_solve_file::<Solver>(path20, true)
}

pub fn test_solve_unsat<Solver: SATSolver>() {
    let _path50 = "tests/uuf50-218/*.cnf";
    //test_solve_file::<Solver>(path50, false)
}
