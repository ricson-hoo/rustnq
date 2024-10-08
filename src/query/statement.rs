use crate::mapping::description::Column;
use crate::query::builder::QueryBuilder;

fn select(fields: Vec<& impl Column>) -> QueryBuilder{
    QueryBuilder::new(fields)
}