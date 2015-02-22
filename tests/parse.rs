extern crate sat;

use std::path::Path;
use std::fs::File;
use std::io::Read;

use sat::parse;

use sat::Id;
use sat::Lit::{P, N};

#[test]
fn parse_file(){
    let file = File::open(&Path::new("./tests/uf20-91/uf20-01.cnf"));
    let mut s = String::new();
    let file_read = file.and_then(|mut f| f.read_to_string(&mut s));
    match file_read {
        Ok(_) => {
            let parsed = parse::parse_file(s);
            assert_eq!(parsed.is_ok(), true);
        }
        Err(_) => panic!("file error")
    }
}

#[test]
fn parse_clause(){
    let control = vec![P(Id(1)), N(Id(2))];
    let parsed = parse::parse_clause("1 -2".to_string());
    assert_eq!(control, parsed.unwrap());
}

#[test]
fn parse_lit(){
    let control = P(Id(5));
    let parsed = parse::parse_lit("5".to_string());
    assert_eq!(control, parsed.unwrap());
    let control = N(Id(22));
    let parsed = parse::parse_lit("-22".to_string());
    assert_eq!(control, parsed.unwrap());
}
