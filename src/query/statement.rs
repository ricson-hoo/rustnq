use crate::mapping::description::Column;
use crate::mapping::types::Table;
use crate::query::builder::QueryBuilder;

pub fn select(fields: Vec<& impl Column>) -> QueryBuilder{
    QueryBuilder::select_fields(fields)
}

pub fn insert_or_update<A>(table: &A) -> QueryBuilder where A : Table{
    QueryBuilder::insert_or_update(table)
}

/*pub fn insert_into<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}

pub fn update<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}*/