use uuid::uuid;
use crate::mapping::description::{Column, SqlColumn};
use crate::mapping::description::Table;
use crate::mapping::types::Int;
use crate::query::builder::{QueryBuilder, TargetTable};
use serde::Serialize;
use sqlx::Error;

pub(crate) struct MultiTypedPrimaryKey {
    pub(crate)uuid_key:Option<String>,
    pub(crate)i32_key:Option<i32>,
    pub(crate)i64_key:Option<i64>,
    pub(crate)u64_key:Option<u64>,
}

pub fn select(fields: Vec<String>) -> QueryBuilder{
    QueryBuilder::init_with_select_fields(fields)
}

pub fn insert_or_update<A,T: Serialize + for<'de> serde::Deserialize<'de>>(table_with_value: &A) -> Result<T,Error> where A : Table{
    /*let target_table = TargetTable::from(table_with_value);
    let mut multiple_typed_primary_key = MultiTypedPrimaryKey{uuid_key:None,i32_key:None,i64_key:None,u64_key:None};
    let primary_key_vec = table_with_value.primary_key();
    if primary_key_vec.len()<1{
        Err(sqlx::Error::Encode("no primary key is found".into()))?
    }
    let primary_key = primary_key_vec.get(0).unwrap().clone();
    match primary_key {
        SqlColumn::Varchar(optional_column_info) => { //uuid
            if let Some(column_info) = optional_column_info {
                let primary_column_value = column_info.value();
                if let None = primary_column_value {
                    let generated_uuid_key = Some(uuid::Uuid::new_v4().to_string().replace("-", "").to_string());
                    multiple_typed_primary_key.uuid_key = generated_uuid_key.clone();
                    //update query_builder with generated uuid_key
                    table_with_value.update_primary_key(generated_uuid_key);
                }else{
                    multiple_typed_primary_key.uuid_key = primary_column_value;
                }
            }else {
                Err(sqlx::Error::Encode("no primary key definition is found".into()))?
            }
        }
        SqlColumn::Int(optional_column_info) => { //non uuid, auto increment
            if let Some(column_info) = optional_column_info {
                multiple_typed_primary_key.i32_key = column_info.value();
            }
        }
        SqlColumn::Bigint(optional_column_info) => { //non uuid, auto increment
            if let Some(column_info) = optional_column_info {
                multiple_typed_primary_key.i64_key = column_info.value();
            }
        }
        SqlColumn::BigintUnsigned(optional_column_info) => { //non uuid, auto increment
            if let Some(column_info) = optional_column_info {
                multiple_typed_primary_key.u64_key = column_info.value();
            }
        }
        _ => {
            Err(sqlx::Error::Encode("unsupported primary key".into()))?
        }
    }
    //
    let query_builder = QueryBuilder::upsert_table_with_value(table_with_value);*/
    Err(Error::RowNotFound)
}

/*pub fn insert_into<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}

pub fn update<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}*/