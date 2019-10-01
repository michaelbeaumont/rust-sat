use docopt::Docopt;
use sat::parse;
use sat::{naive, nonchro, watch};
use sat::{SATSolver, Satness};
use serde::Deserialize;
use std::fs;

// Write the Docopt usage string.
const USAGE: &'static str = "
Usage: rust-sat [--solver TYPE] <inputfile>
       rust-sat --help

Options:
    --solver TYPE  Valid values: naive, watch, nonchro.
    --help         Show this message.
";

#[derive(Deserialize)]
enum SolverType {
    Naive,
    Watch,
    Nonchro,
}

#[derive(Deserialize)]
struct Args {
    arg_inputfile: String,
    flag_solver: Option<SolverType>,
}

pub fn solve_file<Solver: SATSolver>(mut solver: Solver) {
    let solvable = solver.solve();
    print!("Formula is ");
    match solvable {
        Satness::UNSAT(_) => println!("UNSAT"),
        Satness::SAT(interp) => {
            println!("SAT with model:");
            println!("  {:?}", interp);
        }
    }
}

pub fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    match fs::read_to_string(&args.arg_inputfile) {
        Ok(contents) => match parse::parse_file(contents) {
            Ok(cnf) => match args.flag_solver {
                Some(SolverType::Naive) => solve_file(naive::Solver::create(cnf, None)),
                Some(SolverType::Watch) => solve_file(watch::Solver::create(cnf, None)),
                _ => solve_file(nonchro::Solver::create(cnf, None)),
            },
            Err(e) => panic!("parse error: {:?}", e),
        },
        Err(e) => panic!("read error: {}", e),
    }
}
