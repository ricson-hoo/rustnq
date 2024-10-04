use crate::db::base::{Selectable, Table};
use crate::query::base::Condition;

#[derive(Debug)]
pub struct QueryBuilder<'a> {
    from: Option<&'a Table<'a>>,
    fields: Vec<&'a str>,
    conditions: Vec<Condition>,
}

impl <'a> QueryBuilder<'a> {

    pub fn new(fields: Vec<&'a dyn Selectable>) -> QueryBuilder<'a> {
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