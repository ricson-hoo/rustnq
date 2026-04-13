use std::fs;

use sqlx::{AnyConnection, AnyPool, Column, Row};
use sqlx::any::AnyRow;
use sqlx_mysql::MySqlRow;
use crate::codegen::entity::NamingConvention;
use crate::utils::stringUtils;

#[derive(Debug)]
pub struct TableRow {
    pub name: String,
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