#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate log;
extern crate env_logger;
extern crate docopt;
extern crate sat;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use docopt::Docopt;

use sat::parse;
use sat::{Satness, SATSolver};
use sat::{naive, watch, nonchro};

// Write the Docopt usage string.
const USAGE: &'static str = "
Usage: rust-sat [--solver TYPE] <inputfile>
       rust-sat --help

Options:
    --solver TYPE  Valid values: naive, watch, nonchro.
    --help         Show this message.
";

#[derive(Deserialize)]
enum SolverType { Naive, Watch, Nonchro }

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
    let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());

    let filename: &str = args.arg_inputfile.as_ref();
    let file = File::open(&Path::new(filename));
    let mut contents = String::new();

    let file_read = file.and_then(|mut f| f.read_to_string(&mut contents));
    match file_read {
        Ok(_) => {
            match parse::parse_file(contents) {
                Ok(cnf) => {
                    match args.flag_solver {
                        Some(SolverType::Naive) =>
                            solve_file(naive::Solver::create(cnf, None)),
                        Some(SolverType::Watch) =>
                            solve_file(watch::Solver::create(cnf, None)),
                        _ =>
                            solve_file(nonchro::Solver::create(cnf, None))
                    }
                },
                Err(_)  => panic!("Error parsing file encountered.")
            }
        }
        Err(_) => panic!("Error reading file encountered.")
    }
}
