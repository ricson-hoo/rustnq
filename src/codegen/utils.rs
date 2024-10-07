use std::fs;

use sqlx::{AnyConnection, AnyPool, Column, Row};
use sqlx::any::AnyRow;
use sqlx_mysql::MySqlRow;

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

//这里用不了
/*impl<'r> sqlx::FromRow<'r, MySqlRow> for TableRow {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let mut str = "";
        for (column, value) in row.columns().iter().zip(row.clone().values) {
            if column.name().starts_with("Tables_in_") {
                str = row.try_get(column.name())?;
                break;
            }
        }
        Ok(TableRow {
            name: str.to_string()
        })
    }
}

//这里用不了
impl<'r> sqlx::FromRow<'r, AnyRow> for TableFieldRow {
    fn from_row(row: &'r AnyRow) -> Result<Self, sqlx::Error> {
        Ok(TableFieldRow {
            name: row.try_get("Field").unwrap_or("".to_string()),
            data_type: row.try_get("Type").unwrap_or("".to_string()),
            nullable: "Yes" == row.try_get("Null").unwrap_or("".to_string()),
            is_primary_key: "PRI" == row.try_get("Key").unwrap_or("".to_string()),
        })
    }
}
*/
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
                let blob_value: Vec<u8> = row.try_get("Type").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };
        let primary_value = match row.try_get::<String, _>("Key") {
            Ok(value) => value,
            Err(_) => {
                let blob_value: Vec<u8> = row.try_get("Type").expect("Failed to get BLOB value");
                String::from_utf8_lossy(&blob_value).to_string()
            }
        };

        TableFieldRow {
            name: name_value,
            data_type: type_value,
            nullable: "Yes" == nullable_value,
            is_primary_key: "PRI" == primary_value,
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