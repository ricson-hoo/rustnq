use std::collections::HashMap;
use uuid::uuid;
use crate::mapping::description::{Column, SqlColumn};
use crate::mapping::description::Table;
use crate::mapping::column_types::{Bigint, Date, Int};
use crate::query::builder::{Condition, InnerTable, QueryBuilder, SelectField, TargetTable};
use serde::Serialize;
use sqlx::Error;
use tokio::sync::RwLock;
use crate::configuration::{get_processors, PROCESSORS};
use crate::mapping::column_types::Varchar;
use crate::query::builder::construct_upsert_primary_key_value;
use crate::utils::date_sub_unit::DateSubUnit;

pub fn select<T: Into<SelectField>>(fields: Vec<T>) -> QueryBuilder{
    let fields = fields.into_iter().map(|field| field.into()).collect();
    QueryBuilder::init_with_select_fields(fields)
}

pub fn select_distinct<T: Into<SelectField>>(fields: Vec<T>) -> QueryBuilder{
    let fields = fields.into_iter().map(|field| field.into()).collect();
    QueryBuilder::init_with_select_distinct_fields(fields)
}

pub fn count<T: Into<SelectField>>(field:T) -> Bigint{
   Bigint::with_name(format!("count({})", field.into().to_string()))
}

pub fn union_all(sql_list: Vec<QueryBuilder>) -> QueryBuilder{
    let mut list: Vec<String> = vec![];
    for sql in sql_list {
        let sql_result = sql.build();
        if let Ok(sql_string) = sql_result {
            println!("sql string {}", sql_string);
            list.push(sql_string);
        }
    }
    let table = InnerTable{
        table_name: format!("({}) as my_table", list.join(" union all ")),
        map_fields: Default::default(),
    };
    QueryBuilder::init_with_select_all_fields(&table)
}

pub fn exists(sql:QueryBuilder) -> Condition{
    Condition::new(format!("exists ({})", sql.build().unwrap_or_default()))
}

pub fn not_exists(sql:QueryBuilder) -> Condition{
    Condition::new(format!("not exists ({})", sql.build().unwrap_or_default()))
}

pub fn max<T: Into<SelectField>>(field:T) -> Varchar{
    Varchar::with_name(format!("max ({})", field.into().to_string()))
}

pub fn timestamp_diff<T: Into<SelectField>>(date: T, unit: DateSubUnit) -> Int{
    Int::with_name(format!("TIMESTAMPDIFF ({}, {}, CURDATE())", unit, date.into().to_string()))
}

pub fn curdate() -> Varchar{
    Varchar::with_name("CURDATE()".to_string())
}

pub fn year<T: Into<SelectField>>(field:T) -> Varchar{
    Varchar::with_name(format!("YEAR ({})", field.into().to_string()))
}

pub fn month<T: Into<SelectField>>(field:T) -> Varchar{
    Varchar::with_name(format!("MONTH ({})", field.into().to_string()))
}

pub fn count_all() -> Bigint{
    Bigint::with_name("count(*)".to_string())
}

pub fn count_distinct<T: Into<SelectField>>(field:T) -> Bigint{
    Bigint::with_name(format!("count(distinct {})", field.into().to_string()))
}

///DATE_SUB(date, INTERVAL value unit)
pub fn date_sub<T: Into<SelectField>>(value: i32, unit: DateSubUnit) -> Date{
    Date::with_name(format!("DATE_SUB (CURDATE(), INTERVAL {} {})", value, unit))
}

pub fn group_concat<T: Into<SelectField>>(fields: Vec<T>) -> Varchar{
    let fields_str = fields.into_iter().map(|field| field.into().to_string()).collect::<Vec<String>>().join(",");
    Varchar::with_name(format!("group_concat({})",fields_str))
}

pub fn concat<T: Into<SelectField>>(fields: Vec<T>) -> Varchar{
    let fields_str = fields.into_iter().map(|field| field.into().to_string()).collect::<Vec<String>>().join(",");
    Varchar::with_name(format!("concat({})",fields_str))
}

pub async fn insert_or_update<A,T: Serialize + for<'de> serde::Deserialize<'de>>(table_with_value: &mut A) -> Result<T,Error> where A : Table{
    let target_table:TargetTable = TargetTable::new(table_with_value);
    //let mut multiple_typed_primary_key = MultiTypedPrimaryKey{uuid_key:None,i32_key:None,i64_key:None,u64_key:None};
    let primary_key_vec = table_with_value.primary_key();
    if primary_key_vec.len()<1{
        Err(sqlx::Error::ColumnNotFound("no primary key column is found".to_string()))?
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

pub fn update_field_where<A>(table:& A, condition: Condition) -> QueryBuilder where A : Table{
    QueryBuilder::update_table_with_value(table).where_(condition)
}

/*pub fn insert_into<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}

pub fn update<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}*/