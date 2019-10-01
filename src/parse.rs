use std::fmt::Debug;
use std::str::CharIndices;

use super::{Clause, Id, Lit, CNF};

#[derive(Debug)]
pub enum ParseError {
    Syntax(Box<dyn Debug>),
    EOF(usize),
}

pub type Parse<T> = Result<T, ParseError>;

pub fn parse_file(f: String) -> Parse<CNF> {
    CNFParser::new(&f).parse_file()
}

pub fn parse_clause(_f: String) -> Parse<Clause> {
    let mut f = _f.clone();
    f.push_str(" 0");
    CNFParser::new(&f).parse_clause()
}

pub fn parse_lit(l: String) -> Parse<Lit> {
    CNFParser::new(&l).parse_lit()
}

struct CNFParser<'a> {
    curr: char,
    pos: usize,
    //buff: String
    buff: CharIndices<'a>,
}

impl<'a> CNFParser<'a> {
    fn new(buff: &'a String) -> CNFParser<'a> {
        let buff_ = buff.char_indices();
        CNFParser {
            curr: ' ',
            pos: 0,
            buff: buff_,
        }
    }

    fn parse_file(&mut self) -> Parse<CNF> {
        let mut formula = Vec::new();
        loop {
            match self.parse_line() {
                Ok(Some(cls)) => formula.push(cls),
                Ok(None) => return Ok(formula),
                Err(e) => return Err(e),
            }
            self.take()?;
        }
    }

    //add more error checking for correct file format
    fn parse_line(&mut self) -> Parse<Option<Clause>> {
        //move through any whitespace
        self.consume_whitespace()?;
        //if we have a comment or %, ignore the line
        //TODO: take p info into account
        if self.curr == 'c' || self.curr == 'p' || self.curr == '%' {
            self.parse_comment().and(self.parse_line())
        }
        //we have a 0 at the beginning of the line
        //indicating nothing left to parse
        else if self.curr == '0' {
            Ok(None)
        } else {
            self.parse_clause().map(Some)
        }
    }

    fn parse_clause(&mut self) -> Parse<Clause> {
        let mut clause: Clause = Vec::new();
        //move through any whitespace
        self.consume_whitespace()?;
        //while we're not at the end of the clause, parse lits
        while self.curr != '0' {
            match self.parse_lit() {
                Ok(lit) => clause.push(lit),
                Err(e) => return Err(e),
            }
            //remove whitespace inbetween
            match self.consume_whitespace() {
                Err(ParseError::EOF(_)) => break,
                Err(e) => return Err(e),
                _ => {}
            }
        }
        Ok(clause)
    }

    fn parse_lit(&mut self) -> Parse<Lit> {
        //move through any whitespace
        self.consume_whitespace()?;
        if self.curr == '-' {
            self.take().and_then(|()| self.parse_ident().map(Lit::N))
        } else {
            self.parse_ident().map(Lit::P)
        }
    }

    fn parse_ident(&mut self) -> Parse<Id> {
        let mut lit = String::new();
        while !CNFParser::is_whitespace(self.curr) {
            lit.push(self.curr);
            //TODO: make this better with eof maybe self.curr is Option
            match self.take() {
                Err(ParseError::EOF(_)) => break,
                Err(e) => return Err(e),
                _ => {}
            }
        }
        lit.parse()
            .map(Id)
            .map_err(|e| ParseError::Syntax(Box::new(e)))
        /*match lit.parse() {
            Ok(lit)     => Ok(Id(lit)),
            Err(e)      => Err(ParseError::Syntax(box e))
        }*/
    }

    fn parse_comment(&mut self) -> Parse<()> {
        //ignore line
        self.consume_while(&|c| c != '\n')
    }

    fn is_whitespace(c: char) -> bool {
        c == ' ' || c == '\n'
    }

    fn consume_whitespace(&mut self) -> Parse<()> {
        self.consume_while(&|c| CNFParser::is_whitespace(c))
    }

    fn consume_while(&mut self, p: &dyn Fn(char) -> bool) -> Parse<()> {
        while p(self.curr) {
            self.take()?;
        }
        Ok(())
    }

    fn take(&mut self) -> Parse<()> {
        //get a char, wrap any errors and update self.token
        match self.buff.next() {
            None => Err(ParseError::EOF(self.pos)),
            Some((pos, ch)) => {
                self.pos = pos;
                self.curr = ch;
                Ok(())
            }
        }
    }
}
