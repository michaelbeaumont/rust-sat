extern crate sat;

use std::io::{File, BufferedReader};

use sat::Lit::{P, N};
use sat::SATSolver;
use sat::watch::Solver;
use sat::parse;

#[test]
fn test_solve_watch() {
    let cnf1 = vec![vec![P("X".to_string()), N("Y".to_string())],
                    vec![N("X".to_string())]
                    ];
    println!("Test 1: {}", cnf1);
    let mut state1: Solver = SATSolver::create(cnf1);
    let ans1 = state1.solve();
    println!("{}", ans1);
    assert_eq!(ans1.is_sat(), true);

    let cnf2 = vec![vec![P("X".to_string())], vec![N("X".to_string())]];
    println!("Test 2: {}", cnf2);
    let mut state2: Solver = SATSolver::create(cnf2);
    let ans2 = state2.solve();
    println!("{}", ans2);
    assert_eq!(ans2.is_sat(), false);

    let cnf3 = vec![vec![N("X".to_string()), P("X".to_string())],
                    vec![P("X".to_string()), P("Y".to_string()), P("R".to_string())],
                    vec![P("X".to_string()), N("Y".to_string())]
                    ];
    println!("Test 3: {}", cnf3);
    let mut state3: Solver = SATSolver::create(cnf3);
    let ans3 = state3.solve();
    println!("{}", ans3);
    assert_eq!(ans3.is_sat(), true);

    let cnf4 = vec![vec![N("A".to_string()), P("A".to_string())],
                    vec![P("A".to_string()), P("B".to_string()), P("C".to_string())],
                    vec![P("A".to_string()), P("B".to_string()), N("C".to_string())],
                    vec![P("A".to_string()), N("B".to_string()), P("C".to_string())],
                    vec![P("A".to_string()), N("B".to_string()), N("C".to_string())]
                    ];
    println!("Test 4: {}", cnf4);
    let mut state4: Solver = SATSolver::create(cnf4);
    let ans4 = state4.solve();
    println!("{}", ans4);
    assert_eq!(ans4.is_sat(), true);
}


fn get_test_file_reader() -> BufferedReader<File> {
    match File::open(&Path::new("./tests/uf20-0264.cnf")) {
        Ok(f) => {
            BufferedReader::new(f)
        },
        Err(e) => panic!("Couldn't open test file: {}",e)
    }
}

#[test]
fn test_solve_parsed() {
    match parse::parse(get_test_file_reader()) {
        Ok(cnf) => {
            let mut solver: Solver = SATSolver::create(cnf);
            let solvable = solver.solve();
            assert_eq!(solvable.is_sat(), true);
            println!("Parsed SAT Formula is {}", solver.solve())
        },
        _     => panic!("FAIL")
    }
}

