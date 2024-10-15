use crate::mapping::description::{Column};
use std::{fmt,fmt::write, format};
use std::io::Write;
use serde::Serialize;
use sqlx::{Column as MysqlColumn, Row, Value};
use crate::mapping::types::Table;
use sqlx_mysql::MySqlRow;
use sqlx_mysql::{MySqlPool, MySqlPoolOptions};
use url::Url;
use lazy_static::lazy_static;
use std::sync::Mutex;
use serde_json::json;
use serde_json::Value as JsonValue;

lazy_static! {
    static ref POOL: Mutex<Option<MySqlPool>> = Mutex::new(None);
}

async fn get_pool() -> MySqlPool {
    let host = "xxx";
    let database = "iotdb";
    let user = "dbuser";
    let pass = "qooccdbuser#@!";
    let db_uri = format!("mysql://{}:33306/{}", host, database);
    let mut uri = Url::parse(&db_uri).unwrap();
    uri.set_username(user);
    uri.set_password(Some(pass));
    let uri = uri.as_str();

    let pool = {
        let mut guard = POOL.lock().unwrap();
        if guard.is_none() {
            let new_pool = MySqlPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_secs(20))
                .max_connections(5)
                .connect(uri)
                .await.expect("Failed to connect to database");
            *guard = Some(new_pool);
        }
        guard.as_ref().unwrap().clone()
    };

    pool
}

pub trait RowMappable{
    fn from_row(row: &MySqlRow) -> Self;
}

#[derive(Debug)]
pub struct Condition {
    query: String,
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


pub struct QueryBuilder<'a> {
    from: Option<&'a dyn Table>,
    fields: Vec<&'a str>,
    conditions: Vec<Condition>,
}

impl <'a> QueryBuilder<'a> {

    pub fn new(fields: Vec<&'a impl Column>) -> QueryBuilder<'a> {
        let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder { from:None, fields:fields_strs, conditions: vec![] }
    }

    pub fn from<A>(mut self, table:&'a A) -> QueryBuilder<'a> where A : Table{
        self.from = Some(table);
        self
    }

    ///every call to where_, put a new condition or condition group to conditions
    pub fn where_(mut self, condition: Condition) -> QueryBuilder<'a> {
        self.conditions.push(condition);
        self
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

    pub async fn fetch<T: Serialize + for<'de> serde::Deserialize<'de>>(&self) -> Vec<T> {

        let pool = get_pool().await;

        /*let result = sqlx::query(&self.build())
            .fetch_all(&pool)
            .await;

        match result {
            Err(e) => {
                println!("Error select data");
                println!("Error message: [{}].\n", e.to_string());
            }

            Ok(query_result) => {
                for (rindex, row) in query_result.iter().enumerate() {
                    println!("\n* Row number: {}", rindex+1);

                    println!("* Total columns: {}\n", row.columns().len());

                    for (cindex, col) in row.columns().iter().enumerate() {
                        println!(">> {}.", cindex+1);

                        println!(">> {:#?}", col.type_info());
                        println!(">> Name: {}", col.name());
                    }
                }
            }
        }

*/
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
            .fetch_all(&pool)
            .await.unwrap();

        jsons.iter()
            .map(|json| serde_json::from_value(json.clone()).unwrap())
            .collect()
    }

    pub async fn fetch_one<T: Serialize + for<'de> serde::Deserialize<'de>>(&self) -> T {
        let pool = get_pool().await;
        let json = sqlx::query(&self.build())
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
            .fetch_one(&pool)
            .await.unwrap();

        serde_json::from_value(json).expect("Failed to deserialize row")
    }

    pub fn build(&self) -> String {
        "".to_string()
    }

}