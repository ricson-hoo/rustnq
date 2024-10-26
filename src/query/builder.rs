use crate::mapping::description::{Column};
use std::{fmt, fmt::write, format, result};
use std::io::Write;
use serde::{Deserialize, Serialize};
use sqlx::{Column as MysqlColumn, Error, Row, TypeInfo, Value};
use crate::mapping::types::{Table, Varchar};
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

#[derive(Debug, Serialize, Deserialize)]
pub enum BuildErrorType {
    MissingOperation,
    MissingCondition,
    MissingTargetTable,
    MissingFields,
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
#[derive(Debug)]
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

/*pub struct QueryBuilder2 {
    action:QueryAction,
    //from: Option<&'a dyn Table>,
    target_table: Option<String>, //Option<& dyn Table>,
    fields: Vec<String>,
    conditions: Vec<Condition>
}

impl <'a> QueryBuilder2 {
    pub fn select_fields(fields: Vec<&'a impl Column>) -> Self {
        //let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder2 { action:QueryAction::Select, target_table:None, fields:vec![], conditions: vec![] }
    }

    pub async fn execute(&self) {
        let executor = Pool_Provider::get_pool().await;
    }
}
*/

#[derive(Debug,Clone)]
pub struct QueryBuilder {
    operation:Operation,
    //from: Option<&'a dyn Table>,
    target_table: Option<String>,//Option<&'a dyn Table>,
    fields: Vec<String>,
    conditions: Vec<Condition>,
}

impl <'a> QueryBuilder<'a> {

    pub fn select_fields(fields: Vec<&'a impl Column>) -> QueryBuilder<'a> {
        let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder { operation:Operation::Select, target_table:None, fields:fields_strs, conditions: vec![] }
    }

    pub fn insert_or_update<A>(table:&'a A) -> QueryBuilder<'a> where A : Table{
        QueryBuilder { action:Operation::Insert_Or_Update,target_table:Some(table.name()), fields:vec![], conditions: vec![] }
    }

    pub fn from<A>(mut self, table:&'a A) -> QueryBuilder<'a> where A : Table{
        self.target_table = Some(table.name());
        self
    }

/*    pub fn with_entity(mut self, table:&'a A) -> QueryBuilder<'a> {
        self.target_table = Some(table);
        self
    }*/

    ///every call to where_, put a new condition or condition group to conditions
    pub fn where_(mut self, condition: Condition) -> QueryBuilder<'a> {
        self.conditions.push(condition);
        self
    }

    pub fn asVachar(mut self, name: &'a str) -> Varchar {
        Varchar::name_query(name,self)
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
    pub async fn execute(&self) -> Result<u64,Error> {
        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(result) = build_result {
            let result = sqlx::query(&result).execute(pool).await? as MySqlQueryResult; // Pass the reference to sqlx::query()
            Ok(result.rows_affected())
        }else if let Err(e) = build_result {
            Err(Error::Configuration(e.message.into()))
        }else {
            Err(Error::Configuration("未知错误".into()))
        }
    }

    pub async fn fetch<T: Serialize + for<'de> serde::Deserialize<'de>>(&self) -> Vec<T> {

        let pool = POOL.get().unwrap();
        let jsons = sqlx::query(&self.build())
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
            .await.unwrap();

        jsons.iter()
            .map(|json| serde_json::from_value(json.clone()).unwrap())
            .collect()
    }

    pub async fn fetch_one<T: Serialize + for<'de> serde::Deserialize<'de>>(&self) -> T {
        let pool = POOL.get().unwrap();
        let query_str = &self.build();
        println!("query str {:#?}", query_str);
        let json = sqlx::query(query_str)
            .try_map(|row:MySqlRow| {
                self.convert_to_json_value(row)
            })
            .fetch_one(pool)
            .await.unwrap();

        println!("json of product {:#?}", json);

        serde_json::from_value(json).expect("Failed to deserialize row")
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
                        json_obj[camel_case_column_name] = serde_json::Value::String(value);
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "INT" => {
                    let value_result: Result<i32, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        json_obj[camel_case_column_name] = value.into();
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "ENUM" => {
                    let value_result: Result<String, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        json_obj[camel_case_column_name] = serde_json::Value::String(value);
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "DATETIME" => {
                    // Handle DATETIME type
                    let value_result: Result<chrono::NaiveDateTime, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        json_obj[camel_case_column_name] = serde_json::Value::String(value.to_string());
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "CHAR" => {
                    // Handle CHAR type
                    let value_result: Result<String, _> = row.try_get(i);
                    if let Ok(value) = value_result {
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
        let queryString = "";
        match self.operation {
            Operation::Select => {},
            Operation::Insert => {},
            Operation::Update => {},
            Operation::Insert_Or_Update => {},
            Operation::Delete => {},
            _ => Err(QueryBuildError::new(BuildErrorType::MissingOperation,"please provide one of these operation Select, Insert, Update, Delete, Insert_Or_Update".to_string()))
        }
        println!("buider: {:#?}",self);
        println!("queryString: {:#?}",queryString);
        Ok(queryString.to_string())
    }

}