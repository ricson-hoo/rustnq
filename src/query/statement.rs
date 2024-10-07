use crate::mapping::description::Selectable;
use crate::query::builder::QueryBuilder;

fn select(fields: Vec<& dyn Selectable>) -> QueryBuilder{
    QueryBuilder::new(fields)
}