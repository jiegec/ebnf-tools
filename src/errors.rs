use crate::loc::{Loc, NO_LOC};
use std::fmt;

pub struct Error(pub Loc, pub ErrorKind);

// Errors implements Debug, it prints errors line by line
pub struct Errors(pub Vec<Error>);

impl Default for Errors {
    fn default() -> Self {
        Self(vec![])
    }
}

impl Errors {
    // can save some typing in checking the program
    // because when issuing an error, it often follows return a false / error type, which is the default
    // if the compiler complains that it needs type hint, in many cases you can omit the ;, and it will be deduced to ()
    pub fn issue<T: Default>(&mut self, loc: Loc, e: ErrorKind) -> T {
        self.0.push(Error(loc, e));
        Default::default()
    }

    pub fn sorted(mut self) -> Self {
        self.0.sort_unstable_by_key(|e| e.0);
        self
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            NO_LOC => write!(f, "*** Error: {:?}", self.1),
            loc => write!(f, "*** Error at {:?}: {:?}", loc, self.1),
        }
    }
}

pub enum ErrorKind {
    UnrecognizedChar(char),
    SyntaxError,
}

impl fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ErrorKind::*;
        match self {
            UnrecognizedChar(ch) => write!(f, "unrecognized character '{}'", ch),
            SyntaxError => write!(f, "syntax error"),
        }
    }
}

impl fmt::Debug for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for e in &self.0 {
            writeln!(f, "{:?}", e)?
        }
        Ok(())
    }
}
