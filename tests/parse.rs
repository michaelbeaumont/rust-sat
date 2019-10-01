use sat::parse;
use sat::Id;
use sat::Lit::{N, P};
use std::fs;

#[test]
fn parse_file() {
    match fs::read_to_string("./tests/uf20-91/uf20-0101.cnf") {
        Ok(s) => {
            let parsed = parse::parse_file(s);
            assert_eq!(parsed.is_ok(), true);
        }
        Err(e) => panic!("read error: {}", e),
    }
}

#[test]
fn parse_clause() {
    let control = vec![P(Id(1)), N(Id(2))];
    let parsed = parse::parse_clause("1 -2".to_string());
    assert_eq!(control, parsed.unwrap());
}

#[test]
fn parse_lit() {
    let control = P(Id(5));
    let parsed = parse::parse_lit("5".to_string());
    assert_eq!(control, parsed.unwrap());
    let control = N(Id(22));
    let parsed = parse::parse_lit("-22".to_string());
    assert_eq!(control, parsed.unwrap());
}
