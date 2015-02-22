#![feature(box_syntax)]
#![feature(box_patterns)]
extern crate env_logger;
extern crate log;
extern crate glob;

extern crate sat;

mod parse;

mod satsolver;
mod naive;
mod watch;
mod nonchro;

#[test]
fn init() {
    env_logger::init().unwrap();
}
