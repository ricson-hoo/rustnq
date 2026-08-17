use std::fs;
use std::collections::HashMap;

use sqlx::{AnyConnection, AnyPool, Column, Row};
use sqlx::any::AnyRow;
use sqlx_mysql::MySqlRow;
use crate::codegen::entity::NamingConvention;
use crate::utils::stringUtils;

#[derive(Debug)]
pub struct TableRow {
    pub name: String,
}

/// SHOW INDEX 返回的每一行，代表"一个索引的一列"
#[derive(Debug)]
pub struct TableIndexColumnRow {
    pub index_name: String,
    pub column_name: String,
    pub non_unique: bool,
}

/// 分组后的表索引信息
#[derive(Debug)]
pub struct TableIndexRow {
    pub index_name: String,
    pub columns: Vec<String>,
    pub non_unique: bool,
}

#[derive(Debug)]
pub struct TableFieldRow {
    pub(crate) name: String,
    pub(crate) data_type: String,//varchar(32),enum('a','b'),set('a','b'),tinyint(1)
    pub(crate) nullable:bool,
    pub(crate) is_primary_key:bool
}

#[derive(Debug)]
pub struct TableFullFieldRow {
    pub(crate) name: String,
    pub(crate) comment: String,
    pub(crate) data_type: String,//varchar(32),enum('a','b'),set('a','b'),tinyint(1)
    pub(crate) nullable:bool,
    pub(crate) is_primary_key:bool
}

impl From<&MySqlRow> for TableRow {
    fn from(row: &MySqlRow) -> Self {
        let mut str = "".to_string();
        for column in row.columns() {
            let column_name = column.name();
            if column_name.starts_with("Tables_in_") {
                if let Ok(value) = row.try_get::<Vec<u8>, _>(column_name) {
                    if let Ok(utf8_string) = String::from_utf8(value) {
                        str = utf8_string;
                    } else {
                        str = format!("Error decoding VARBINARY column '{}'", column_name);
                    }
                } else {
                    str = format!("Error retrieving VARBINARY value for column '{}'", column_name);
                }
            }
        }
        TableRow {
            name: str.to_string()
        }
    }
}

impl From<&MySqlRow> for TableIndexColumnRow {
    fn from(row: &MySqlRow) -> Self {
        let index_name = row.try_get::<String,_>("Key_name").unwrap_or_else(|error|{
            panic!("failed to get Key_name: {}",error);
        });
        let column_name = row.try_get::<String,_>("Column_name").unwrap_or_default();
        // Non_unique: 0 表示唯一索引，1 表示非唯一
        let non_unique = row.try_get::<i64,_>("Non_unique").unwrap_or(1) != 0;
        TableIndexColumnRow {
            index_name,
            column_name,
            non_unique,
        }
    }
}

impl From<&MySqlRow> for TableFieldRow {
    fn from(row: &MySqlRow) -> Self {
        let name_value = row.try_get::<String,_>("Field").unwrap_or_else(|error|{
            panic!("failed to get name: {}",error);
        });
        // Attempt to get the value of the "Type" column as a String
        let type_value = match row.try_get::<String, _>("Type") {
            Ok(value) => value,
            Err(_) => {
                let blob_value: Vec<u8> = row.try_get("Type").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };
        let nullable_value = match row.try_get::<String, _>("Null") {
            Ok(value) => value,
            Err(_) => {
                let blob_value: Vec<u8> = row.try_get("Null").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };
        let primary_value = match row.try_get::<String, _>("Key") {
            Ok(value) => value,
            Err(_) => {
                let blob_value: Vec<u8> = row.try_get("Key").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };

        TableFieldRow {
            name: name_value,
            data_type: type_value,
            nullable: "Yes" == nullable_value || "YES" == nullable_value,
            is_primary_key: "Pri" == primary_value || "PRI" == primary_value,
        }
    }
}

impl From<&MySqlRow> for TableFullFieldRow {
    fn from(row: &MySqlRow) -> Self {
        let name_value = row.try_get::<String,_>("Field").unwrap_or_else(|error|{
            panic!("failed to get name: {}",error);
        });
        // Attempt to get the value of the "Type" column as a String
        let type_value = match row.try_get::<String, _>("Type") {
            Ok(value) => value,
            Err(_) => {
                let blob_value: Vec<u8> = row.try_get("Type").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };
        let nullable_value = match row.try_get::<String, _>("Null") {
            Ok(value) => value,
            Err(_) => {
                let blob_value: Vec<u8> = row.try_get("Null").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };
        let primary_value = match row.try_get::<String, _>("Key") {
            Ok(value) => value,
            Err(_) => {
                let blob_value: Vec<u8> = row.try_get("Key").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };
        let comment_value = match row.try_get::<String, _>("Comment") {
            Ok(value) => value,
            Err(_) => {
                let blob_value: Vec<u8> = row.try_get("Comment").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };

        TableFullFieldRow {
            name: name_value,
            comment: comment_value,
            data_type: type_value,
            nullable: "Yes" == nullable_value || "YES" == nullable_value,
            is_primary_key: "Pri" == primary_value || "PRI" == primary_value,
        }
    }
}

//get tables' definitions via show tables statement
pub(crate) async fn get_tables(conn: &sqlx::pool::Pool<sqlx_mysql::MySql>) -> Result<Vec<TableRow>, sqlx::Error> {
    let select_query = sqlx::query("SHOW TABLES");
    let rows = select_query.fetch_all(conn).await?;
    let tables: Vec<TableRow> = rows.iter().map(|row:&MySqlRow| {
        row.into()
    }).collect();
    Ok(tables)
}

//get table fields/columns
pub(crate) async fn get_table_fields(conn: &sqlx::pool::Pool<sqlx_mysql::MySql>, table_name: &str) -> Result<Vec<TableFieldRow>, sqlx::Error> {
    let query = format!("DESCRIBE `{}`;",table_name);
    let select_query = sqlx::query(&query);
    let rows = select_query.fetch_all(conn).await?;
    let fields: Vec<TableFieldRow> = rows.iter().map(|row:&MySqlRow| {
        row.into()
    }).collect();
    Ok(fields)
}

pub(crate) async fn get_table_full_fields(conn: &sqlx::pool::Pool<sqlx_mysql::MySql>, table_name: &str) -> Result<Vec<TableFullFieldRow>, sqlx::Error> {
    let query = format!("SHOW FULL COLUMNS FROM `{}`;",table_name);
    let select_query = sqlx::query(&query);
    let rows = select_query.fetch_all(conn).await?;
    let fields: Vec<TableFullFieldRow> = rows.iter().map(|row:&MySqlRow| {
        row.into()
    }).collect();
    Ok(fields)
}

//get table indexes via show index statement, 按索引名分组
pub(crate) async fn get_table_indexes(conn: &sqlx::pool::Pool<sqlx_mysql::MySql>, table_name: &str) -> Result<Vec<TableIndexRow>, sqlx::Error> {
    let query = format!("SHOW INDEX FROM `{}`;",table_name);
    let select_query = sqlx::query(&query);
    let rows = select_query.fetch_all(conn).await?;
    // MySQL 中唯一索引的所有列 Non_unique 均为 0，非唯一索引的所有列均为 1，按行取即可
    let mut index_map: HashMap<String, (Vec<String>, bool)> = HashMap::new();
    for row in rows.iter() {
        let index_column: TableIndexColumnRow = row.into();
        let entry = index_map.entry(index_column.index_name.clone()).or_insert((vec![], !index_column.non_unique));
        entry.0.push(index_column.column_name);
        if index_column.non_unique {
            entry.1 = true;
        }
    }
    let mut indexes: Vec<TableIndexRow> = index_map.into_iter().map(|(index_name, (columns, non_unique))| {
        TableIndexRow {
            index_name,
            columns,
            non_unique,
        }
    }).collect();
    indexes.sort_by(|a, b| a.index_name.cmp(&b.index_name));
    Ok(indexes)
}

// 索引名 -> 常量名: snake_case -> SCREAMING_SNAKE_CASE
pub(crate) fn to_screaming_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut first = true;
    for c in name.chars() {
        if c.is_uppercase() && !first {
            result.push('_');
        }
        if c.is_alphanumeric() {
            result.push(c.to_uppercase().next().unwrap());
        }
        first = false;
    }
    result
}

pub(crate) fn reserved_field_names() -> Vec<String> {
    vec![
        "as", "async", "await", "break", "const", "continue", "crate", "dyn",
        "else", "enum", "extern", "false", "fn", "for", "if", "impl",
        "in", "let", "loop", "match", "mod", "move", "mut", "pub",
        "ref", "return", "self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while"
    ].iter().map(|s| s.to_string()).collect()
}

pub(crate) fn get_simple_name(qualified_name: &str) -> String {
    if qualified_name.is_empty(){
        return "".to_string();
    } 
    let last_sep_index = qualified_name.rfind("::").unwrap_or(0);
    qualified_name[(last_sep_index + 1)..].to_string()
}

// Create parent directories if they do not exist yet.
pub fn prepare_directory(path:& std::path::Path){
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect(&format!("Failed to create directory {}", parent.display()));
        }
    }
    if path.is_dir() && !path.exists() {
        fs::create_dir_all(path).expect(&format!("Failed to create directory {}", path.display()));
    }
}

pub fn format_name(name:&str, convention: NamingConvention) -> String{
    match convention {
       NamingConvention::CamelCase => {
           stringUtils::to_camel_case(name)
       }
       NamingConvention::SnakeCase => {
            let mut result = String::new();
            let mut first = true;
            for c in name.chars() {
                if c.is_uppercase() && !first {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
                first = false;
            }
            result
        }
        NamingConvention::PascalCase => {
            let mut result = String::new();
            let mut capitalize_next = true;
            for c in name.chars() {
                if c.is_alphanumeric() {
                    if capitalize_next {
                        result.push(c.to_uppercase().next().unwrap());
                        capitalize_next = false;
                    } else {
                        result.push(c.to_lowercase().next().unwrap());
                    }
                } else {
                    capitalize_next = true;
                }
            }
            result
        }
    }
}