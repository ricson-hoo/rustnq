use crate::mapping::description::Selectable;
use crate::query::builder::QueryBuilder;

fn select<'a>(fields: Vec<&'a  dyn Selectable<'a>>) -> QueryBuilder<'a>{
    QueryBuilder::new(fields)
}