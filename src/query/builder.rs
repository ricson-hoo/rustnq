use crate::mapping::description::{Column};
use std::{fmt,fmt::write, format};
use std::io::Write;
use crate::mapping::types::Table;

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


#[derive(Debug)]
pub struct QueryBuilder<'a> {
    from: Option<&'a Table<'a>>,
    fields: Vec<&'a str>,
    conditions: Vec<Condition>,
}

impl <'a> QueryBuilder<'a> {

    pub fn new(fields: Vec<&'a impl Column>) -> QueryBuilder<'a> {
        let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder { from:None, fields:fields_strs, conditions: vec![] }
    }

    fn from(mut self, table:&'a Table) -> QueryBuilder<'a> {
        self.from = Some(table);
        self
    }

    ///every call to where_, put a new condition or condition group to conditions
    pub fn where_(mut self, condition: Condition) -> QueryBuilder<'a> {
        self.conditions.push(condition);
        self
    }
}