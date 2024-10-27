use crate::mapping::description::Column;
use crate::mapping::types::Table;
use crate::query::builder::QueryBuilder;

pub fn select(fields: Vec<& impl Column>) -> QueryBuilder{
    QueryBuilder::init_with_select_fields(fields)
}

pub fn insert_or_update<A>(table: &A) -> QueryBuilder where A : Table{
    QueryBuilder::upsert_table_with_value(table)
}

/*pub fn insert_into<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}

pub fn update<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}*/