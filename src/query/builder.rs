use crate::mapping::description::{Table,Column};
use std::{fmt, fmt::write, format, result};
use std::io::Write;
use serde::{Deserialize, Serialize};
use sqlx::{Column as MysqlColumn, Error, Row, TypeInfo, Value};
use crate::mapping::types::{Varchar};
use sqlx_mysql::{MySqlQueryResult, MySqlRow, MySqlTypeInfo};
use sqlx_mysql::{MySqlPool, MySqlPoolOptions};
use url::Url;
use lazy_static::lazy_static;
use std::sync::Mutex;
use serde_json::json;
use serde_json::Value as JsonValue;
use std::future::Future;
use sqlx::Executor;
use sqlx::Database;
use sqlx::IntoArguments;
use crate::query::pool::{POOL};
use crate::utils::stringUtils::to_camel_case;
use crate::mapping::description::SqlColumn;

#[derive(Debug, Serialize, Deserialize)]
pub enum BuildErrorType {
    MissingOperation,
    MissingCondition,
    MissingTargetTable,
    MissingFields,
    MissingValues,
    OtherError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryBuildError {
    pub(crate) error: BuildErrorType,
    pub(crate) message: String,
}

impl QueryBuildError{
    pub fn new(error: BuildErrorType, message:String) -> Self{
        QueryBuildError{
            error, message
        }
    }
}

pub trait RowMappable{
    fn from_row(row: &MySqlRow) -> Self;
}

#[derive(Debug,Clone)]
pub struct Condition {
    pub query: String,
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

#[derive(Debug,Clone)]
pub enum Operation{
    Select,Insert,Update,Insert_Or_Update,Delete
}

#[derive(Debug,Clone)]
pub(crate) struct TargetTable{
    pub name:String,
    pub columns:Vec<SqlColumn>,
    pub primary_key:Vec<SqlColumn>
}

impl TargetTable {
    fn from(table: & dyn Table) -> Self {
        TargetTable{
            name: table.name(),
            columns: table.columns(),
            primary_key: table.primary_key(),
        }
    }
}

#[derive(Debug,Clone)]
pub struct QueryBuilder {
    operation:Operation,
    is_select_all:Option<bool>,
    pub target_table: Option<TargetTable>,//Option<String>,
    fields: Vec<String>,
    upsert_values: Vec<String>,//insert values or update values
    conditions: Vec<Condition>,
    limit:Option<i32>,
}

impl QueryBuilder {

    pub fn select_all() -> QueryBuilder {
        QueryBuilder { operation:Operation::Select, is_select_all: Some(true), target_table:None, fields:vec![], conditions: vec![], upsert_values: vec![], limit: None }
    }

    pub fn init_with_select_fields(fields: Vec<String>) -> QueryBuilder {
        //let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder { operation:Operation::Select, is_select_all:None, target_table:None, fields:fields, conditions: vec![], upsert_values: vec![], limit: None }
    }

    pub fn insert_into_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        //table.insert_query_builder()
        QueryBuilder { operation:Operation::Insert, is_select_all:None, target_table:Some(TargetTable::from(table)), fields:vec![], conditions: vec![], upsert_values: vec![], limit: None }
    }

    pub fn update_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        //table.update_query_builder()
        QueryBuilder { operation:Operation::Update,is_select_all:None, target_table:Some(TargetTable::from(table)), fields:vec![], conditions: vec![], upsert_values: vec![], limit: None }
    }

    pub fn upsert_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        //table.upsert_query_builder()
        QueryBuilder { operation:Operation::Insert_Or_Update,is_select_all:None, target_table:Some(TargetTable::from(table)), fields:vec![], conditions: vec![], upsert_values: vec![], limit: None }
    }

    pub fn delete_one_from<A>(table:& A) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, target_table:Some(TargetTable::from(table)), fields:vec![], conditions: vec![], upsert_values: vec![], limit: Some(1) }
    }

    pub fn delete_one_where<A>(table:& A,condition: Condition) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, target_table:Some(TargetTable::from(table)), fields:vec![], conditions: vec![condition], upsert_values: vec![], limit: Some(1) }
    }

    pub fn delete_rows_with_conditions<A>(table:& A,condition: Condition) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, target_table:Some(TargetTable::from(table)), fields:vec![], conditions: vec![condition], upsert_values: vec![], limit: None }
    }

    pub fn from<A>(mut self, table:& A) -> QueryBuilder where A : Table{
        self.target_table = Some(TargetTable::from(table));
        /*if let Some(is_select_all) = self.is_select_all {
            if is_select_all {
                let fields_strs = table.columns().iter().map(|field| field.name()).collect();
                self.fields = fields_strs;
            }
        }*/
        self
    }

    pub fn add_select_fields(){

    }

    pub fn add_upsert_fields(){ //need key_values

    }


/*    pub fn with_entity(mut self, table:&'a A) -> QueryBuilder<'a> {
        self.target_table = Some(table);
        self
    }*/

    ///every call to where_, put a new condition or condition group to conditions
    pub fn where_(mut self, condition: Condition) -> QueryBuilder {
        self.conditions.push(condition);
        self
    }

    pub fn limit(mut self, limit:i32) -> QueryBuilder {
        self.limit = Some(limit);
        self
    }

    pub fn asVachar(mut self, name: &str) -> Varchar {
        Varchar::with_name_query(name.to_string(),Some(self))
    }

    /*pub fn fetch_one_into<T: RowMappable>(&self) -> T {
        // 假设这里是查询并获取到的一行数据row
        let row: MySqlRow = MySqlRow::fmt();// = get_row_from_query_result();

        T::from_row(&row)
    }

    pub fn fetch_into<T: RowMappable>(&self) -> Vec<T> {
        // 假设这里是查询并获取到的一行数据row
        let row: MySqlRow;// = get_row_from_query_result();

        vec![T::from_row(&row)]
    }*/

    ///execute insert/update/delete and return the affected rows number
    pub async fn execute(&self) -> Result<MySqlQueryResult,Error> {
        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);
            let result = sqlx::query(&query_string).execute(pool).await? as MySqlQueryResult; // Pass the reference to sqlx::query()
            Ok(result)
        }else if let Err(e) = build_result {
            Err(Error::Configuration(e.message.into()))
        }else {
            Err(Error::Configuration("未知错误".into()))
        }
    }

    /*pub async fn execute_return<T>(&self) -> Result<T,Error> {
        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);
            let result = sqlx::query(&query_string).execute(pool).await? as MySqlQueryResult; // Pass the reference to sqlx::query()
            Ok(result)
        }else if let Err(e) = build_result {
            Err(Error::Configuration(e.message.into()))
        }else {
            Err(Error::Configuration("未知错误".into()))
        }
    }*/

    pub async fn fetch<T: Serialize + for<'de> serde::Deserialize<'de>>(&self) -> Result<Vec<T>, Error> {

        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);

            let jsons = sqlx::query(&query_string)
                .try_map(|row:MySqlRow| {
                    let mut json_obj = json!({});
                    let columns = row.columns();
                    for column in columns {
                        let column_name = column.name();
                        let value_result: Result<JsonValue, _> = row.try_get(&column_name);
                        if let Ok(value) = value_result {
                            json_obj[column_name] = value;
                        }
                    }
                    Ok(json_obj)
                })
                .fetch_all(pool)
                .await?;

            Ok(jsons.iter()
                .map(|json| serde_json::from_value::<T>(json.clone()).unwrap())
                .collect::<Vec<_>>())
        }else if let Err(e) = build_result {
            Err(Error::Configuration(e.message.into()))
        }else {
            Err(Error::Configuration("未知错误".into()))
        }
    }

    pub async fn fetch_one<T: Serialize + for<'de> serde::Deserialize<'de>>(&mut self) -> Result<Option<T>,Error> {

        self.limit = Some(1);

        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);

            let json_result = sqlx::query(&query_string)
                .try_map(|row:MySqlRow| {
                    self.convert_to_json_value(row)
                })
                .fetch_one(pool)
                .await;

            match json_result {
                Ok(json)=>{
                    println!("json of product {:#?}", json);
                    let json_parse_result = serde_json::from_value(json);
                    match json_parse_result {
                        Ok(json)=>{
                            Ok(Some(json))
                        },
                        Err(e) =>{
                            Err(Error::Decode(e.to_string().into()))
                        }
                    }
                },
                Err(e) => {
                    Err(e)
                }
            }
        }else if let Err(e) = build_result {
            Err(Error::Encode(e.message.into()))
        }else {
            Err(Error::Encode("未知错误".into()))
        }
    }

    ///将mysql数据行转为JsonValue
    fn convert_to_json_value(&self, row:MySqlRow)-> Result<JsonValue, Error>{
        println!("row of product {:#?}", row);
        let mut json_obj = json!({});
        let columns = row.columns();
        let mut i=0;
        for column in columns {
            let column_name = column.name();
            let camel_case_column_name = to_camel_case(&column_name);
            let type_name = column.type_info().name();
            println!("type_name of {} {:#?}",column_name, type_name);
            match type_name {
                "VARCHAR" => {
                    let value_result: Result<String, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        json_obj[column_name] = serde_json::Value::String(value.clone());
                        json_obj[camel_case_column_name] = serde_json::Value::String(value);
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "INT" => {
                    let value_result: Result<i32, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        json_obj[column_name] = value.clone().into();
                        json_obj[camel_case_column_name] = value.into();
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "ENUM" => {
                    let value_result: Result<String, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        json_obj[column_name] = serde_json::Value::String(value.clone());
                        json_obj[camel_case_column_name] = serde_json::Value::String(value);
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "DATETIME" => {
                    // Handle DATETIME type
                    let value_result: Result<chrono::NaiveDateTime, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        json_obj[column_name] = serde_json::Value::String(value.clone().to_string());
                        json_obj[camel_case_column_name] = serde_json::Value::String(value.to_string());
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "CHAR" => {
                    // Handle CHAR type
                    let value_result: Result<String, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        json_obj[column_name] = serde_json::Value::String(value.clone());
                        json_obj[camel_case_column_name] = serde_json::Value::String(value);
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                &_ => {}
            }

            i += 1;
        }
        Ok(json_obj)
    }

    pub fn build(&self) -> Result<String,QueryBuildError> {
        let mut queryString = "".to_string();
        match self.operation {
            Operation::Select => {
                if(!self.fields.is_empty()){
                    queryString = format!("select {}",self.fields.join(", "));
                }else {
                    return Err(QueryBuildError::new(BuildErrorType::MissingFields,"please provide at lease on field for select operation".to_string()));
                }
                if self.target_table.is_some() {
                    queryString = format!("{} from {}",queryString, self.target_table.clone().unwrap().name);
                }else {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable,"please provide table name to select from".to_string()));
                }
                if self.conditions.len() > 0 {
                    queryString = format!("{} where {}",queryString, self.conditions.iter()
                            .map(|condition| condition.query.clone())
                            .collect::<Vec<String>>()
                            .join(" AND "));
                }
                if self.limit.is_some() {
                    queryString = format!("{} limit {}",queryString,self.limit.unwrap());
                }
            },
            Operation::Insert => {
                if self.target_table.is_none() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable, "please provide table name for insert operation".to_string()));
                }

                if self.upsert_values.is_empty() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingValues, "please provide values for insert operation".to_string()));
                }

                let columns: Vec<&str> = vec![];//self.upsert_values.keys().map(|s| s.as_str()).collect();
                let values: Vec<String> = vec![];//self.upsert_values.values().map(|v| v.to_string()).collect();

                queryString = format!("insert into {} ({}) values ({})", self.target_table.clone().unwrap().name, columns.join(", "), values.join(", "));
            },
            Operation::Update => {
                if self.target_table.is_none() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable, "please provide table name for update operation".to_string()));
                }

                if self.upsert_values.is_empty() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingValues, "please provide values for update operation".to_string()));
                }

                if self.conditions.len() <= 0 {
                    return Err(QueryBuildError::new(BuildErrorType::MissingCondition, "please provide filters for Update operation".to_string()));
                }

                let mut set_values: Vec<String> = Vec::new();
                //for (column, value) in &self.update_values {
                //    set_values.push(format!("{} = {}", column, value));
                //}

                queryString = format!("update {} set {} where {}", self.target_table.clone().unwrap().name, set_values.join(", "), self.conditions.iter()
                    .map(|condition| condition.query.clone())
                    .collect::<Vec<String>>()
                    .join(" AND "));
            },
            Operation::Insert_Or_Update => {
                //write code for me
            },
            Operation::Delete => {
                if self.target_table.is_none() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable, "please provide table name for delete operation".to_string()));
                }
                if self.conditions.len() <= 0 {
                    return Err(QueryBuildError::new(BuildErrorType::MissingCondition, "please provide filters for  delete operation".to_string()));
                }
                queryString = format!("delete from {} where {}", self.target_table.clone().unwrap().name, self.conditions.iter()
                    .map(|condition| condition.query.clone())
                    .collect::<Vec<String>>()
                    .join(" AND "));
            },
            _ => {
                return Err(QueryBuildError::new(BuildErrorType::MissingOperation,"please provide one of these operation Select, Insert, Update, Delete, Insert_Or_Update".to_string()));
            }
        }
        println!("buider: {:#?}",self);
        println!("queryString: {:#?}",queryString);
        Ok(queryString.to_string())
    }

}