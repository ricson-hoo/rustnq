use uuid::uuid;
use crate::mapping::description::{Column, SqlColumn};
use crate::mapping::description::Table;
use crate::mapping::types::{Bigint, Int};
use crate::query::builder::{Condition, QueryBuilder, TargetTable};
use serde::Serialize;
use sqlx::Error;
use crate::mapping::types::Varchar;
use crate::query::builder::construct_upsert_primary_key_value;

/*pub(crate) struct MultiTypedPrimaryKey {
    pub(crate)uuid_key:Option<String>,
    pub(crate)i32_key:Option<i32>,
    pub(crate)i64_key:Option<i64>,
    pub(crate)u64_key:Option<u64>,
}*/

pub fn select<T: Into<String>>(fields: Vec<T>) -> QueryBuilder{
    let fields = fields.into_iter().map(|field| field.into()).collect();
    QueryBuilder::init_with_select_fields(fields)
}

pub fn count<T: Into<String>>(field:T) -> Bigint{
   Bigint::with_name(format!("count {}", field.into()))
}

pub fn count_all() -> Bigint{
    Bigint::with_name("count (*)".to_string())
}

pub fn count_distinct<T: Into<String>>(field:T) -> Bigint{
    Bigint::with_name(format!("count (distinct {})", field.into()))
}

pub fn concat<T: Into<String>>(fields: Vec<T>) -> Varchar{
    let fields_str = fields.into_iter().map(|field| field.into()).collect::<Vec<String>>().join(",");
    Varchar::with_name(format!("concat({})",fields_str))
}

pub async fn insert_or_update<A,T: Serialize + for<'de> serde::Deserialize<'de>>(table_with_value: &mut A) -> Result<T,Error> where A : Table{
    let target_table:TargetTable = TargetTable::new(table_with_value);
    //let mut multiple_typed_primary_key = MultiTypedPrimaryKey{uuid_key:None,i32_key:None,i64_key:None,u64_key:None};
    let primary_key_vec = table_with_value.primary_key();
    if primary_key_vec.len()<1{
        Err(sqlx::Error::Encode("no primary key is found".into()))?
    }
    let mut text_primary_key_value:Option<String> = None;
    let mut text_primary_key_name:Option<String> = None;
    if primary_key_vec.len() == 1 {
        let primary_key = primary_key_vec.get(0).unwrap().clone();
        match primary_key {
            SqlColumn::Varchar(optional_column_info) => { //uuid
                if let Some(column_info) = optional_column_info {
                    text_primary_key_name = Some(column_info.name());
                    let primary_column_value = column_info.value();
                    if let None = primary_column_value {
                        let generated_uuid_key = Some(uuid::Uuid::new_v4().to_string().replace("-", "").to_string());
                        table_with_value.update_primary_key(vec![SqlColumn::Varchar(Some(Varchar::with_name_value(column_info.name(),generated_uuid_key.clone())))]);
                        text_primary_key_value = generated_uuid_key;
                    }else {
                        text_primary_key_value = primary_column_value;
                    }

                }
            }
            _ => {
                //Err(sqlx::Error::Encode("unsupported primary key".into()))?
            }
        }
    }

    //
    let upsert_result = QueryBuilder::upsert_table_with_value(table_with_value).execute().await;
    match upsert_result {
        Ok(query_result) => {
            let mut condition= Condition::new("1 = 1".to_string());
            if query_result.rows_affected() > 0 {
                if table_with_value.primary_key().len()>1{
                    let mut primary_key_as_conditions = vec![];
                    construct_upsert_primary_key_value(&table_with_value.primary_key(), &mut vec![], &mut vec![], &mut primary_key_as_conditions);
                    for cond in primary_key_as_conditions {
                        condition = condition.and(Condition::new(cond))
                    }
                }else{
                    if(text_primary_key_value.is_some()){
                        condition = condition.and(Condition::new(format!("{} = '{}'",&text_primary_key_name.unwrap(),&text_primary_key_value.unwrap_or_default())))
                    }else{ //primary key is not a string
                        let primary_key = primary_key_vec.get(0).unwrap().clone();
                        let primary_key_name = primary_key.get_col_name();
                        let last_insert_id = query_result.last_insert_id();
                        if last_insert_id>0 {
                            condition = condition.and(Condition::new(format!("{} = {}",primary_key_name, last_insert_id)));
                        }else{ //updated a row?
                            let mut primary_key_as_conditions = vec![];
                            construct_upsert_primary_key_value(&table_with_value.primary_key(), &mut vec![], &mut vec![], &mut primary_key_as_conditions);
                            for cond in primary_key_as_conditions {
                                condition = condition.and(Condition::new(cond))
                            }
                        }
                    }
                }
                let result = QueryBuilder::select_all_fields().from(table_with_value).where_(condition).fetch_one().await;
                match result {
                    Ok(row) => {
                        match row {
                            Some(row) => {
                                return Ok(row)
                            }
                            None => {
                                return Err(sqlx::Error::RowNotFound)
                            }
                        }
                    },
                    Err(error) => {
                        Err(error)
                    }
                }
            }else {
                Err(sqlx::Error::RowNotFound)
            }
        }
        Err(error) => {
            return Err(error)
        }
    }
}

pub fn delete_one_from<A>(table:& A) -> QueryBuilder where A : Table{
    QueryBuilder::delete_one_from(table)
}

pub fn delete_one_where<A>(table:& A, condition: Condition) -> QueryBuilder where A : Table{
    QueryBuilder::delete_one_where(table, condition)
}

/*pub fn insert_into<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}

pub fn update<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}*/