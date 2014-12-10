use std::io::{IoError, File, BufferedReader};
use super::{Lit, CNF, Clause};
use std::io::IoErrorKind::{EndOfFile};

#[deriving(Show)]
enum ParseError {
    Io(IoError),
    FileParsed
}

impl ParseError {
    fn allow_eof(e: ParseError) -> ParseError {
        match e {
            ParseError::Io(e) =>
                match e.kind {
                    EndOfFile => ParseError::FileParsed,
                    _         => ParseError::Io(e)
                },
                e => e
        }
    }
}

pub type Parse<T> = Result<T, ParseError>;

pub fn parse(f: BufferedReader<File>) -> Parse<CNF> {
    let mut parser = CNFParser { token: ' ', f: f};
    parser.parse_file()//.ok()
}

struct CNFParser {
    token: char,
    f: BufferedReader<File>
}

impl CNFParser {
    fn parse_file(&mut self) -> Parse<CNF> {
        let mut formula = Vec::new();
        loop {
            match self.parse_line() {
                Ok(cls)                     => formula.push(cls),
                Err(ParseError::FileParsed) => return Ok(formula),
                Err(e)                      => return Err(e)
            }
        }
    }

    //add more error checking for correct file format
    fn parse_line(&mut self) -> Parse<Clause> {
        //take the first char and move through any whitespace
        //eof is allowed here
        try!(self.take().
             and(self.consume_whitespace().map_err(ParseError::allow_eof)
            ));
        //if we have a comment or %, ignore the line
        //ignore p lines for now
        if self.token == 'c' || self.token == 'p' || self.token == '%' { 
            self.parse_comment().and(self.parse_line())
        }
        //we have a 0 indicating nothing left to parse
        else if self.token == '0' {
            return Err(ParseError::FileParsed)
        }
        else {
            //we have what should be a clause
            let mut clause: Clause = Vec::new();
            //while we're not at the end of the clause, parse lits
            while self.token != '0' {
                match self.parse_lit() {
                    Ok(lit) => clause.push(lit),
                    Err(e) => return Err(e)
                }
                //remove whitespace inbetween
                try!(self.consume_whitespace());
            }
            Ok(clause)
        }
    }

    fn parse_lit(&mut self) -> Parse<Lit> {
        if self.token == '-' { 
            self.take().
            and(self.parse_ident().map(Lit::N))
        }
        else { 
            self.parse_ident().map(Lit::P)
        }
    }

    fn parse_ident(&mut self) -> Parse<String> {
        let mut lit = String::new();
        while !CNFParser::is_whitespace(self.token) {
            lit.push(self.token);  
            let take = self.take();
            if take.is_err() { return Err(take.unwrap_err()) }
        }
        return Ok(lit)
    }

    fn parse_comment(&mut self) -> Parse<()> {
        //ignore line
        self.f.read_line().map(|_| ()).map_err(ParseError::Io)
    }

    fn is_whitespace(c: char) -> bool {
        c == ' ' || c == '\n'
    }

    fn consume_whitespace(&mut self) -> Parse<()> {
        self.consume_while(CNFParser::is_whitespace)
    }

    fn consume_while(&mut self, p: |char| -> bool) -> Parse<()> {
        while p(self.token) {
            let take = self.take();
            if take.is_err() { return take }
        }
        Ok(())
    }

    fn take(&mut self) -> Parse<()> {
        //get a char, wrap any errors and update self.token
        let char_read = self.f.read_char().map_err(|e| ParseError::Io(e));
        char_read.and_then(|x| { Ok(self.token = x)})
    }
}

#[cfg(test)]
mod tests {
    use std::io::{File, BufferedReader};
    #[test]
    fn parse_test(){
        match File::open(&Path::new("./tests/uf20-01.cnf")) {
            Ok(f) => {
                let reader = BufferedReader::new(f);
                let parsing = super::parse(reader);
                assert_eq!(parsing.is_ok(), true);
                println!("{}", parsing);
            },
            Err(e) => panic!("Couldn't open test file: {}",e)
        }
    }
}
