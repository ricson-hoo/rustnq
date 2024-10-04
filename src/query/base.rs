use std::{fmt,fmt::write, format};
use std::io::Write;

#[derive(Debug)]
pub struct Condition {
    query: String,
}

impl Condition {
    pub fn new(query: String) -> Condition {
        Condition { query }
    }

    pub fn and(self, other: Condition) -> Condition {
        Condition {
            query: format!("({}) AND ({})", self.query, other.query),
        }
    }

    pub fn or(self, other: Condition) -> Condition {
        Condition {
            query: format!("({}) OR ({})", self.query, other.query),
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.query)
    }
}