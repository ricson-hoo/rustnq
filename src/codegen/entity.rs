use std::path::Path;
use std::fs;
use std::fs::{File, OpenOptions};
use std::println;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::io::{Write, BufWriter};
use anyhow::bail;
use sqlx::pool::PoolConnection;
use sqlx::{AnyConnection, AnyPool, Error};
use sqlx_mysql::MySql;
use crate::codegen::utils;
use crate::codegen::utils::TableRow;
use crate::utils::stringUtils;
use serde::{Serialize, Deserialize}; 

struct StructFieldType {
    qualified_name: String,
    is_primitive_type: bool,
    import:Option<String>
}

struct GeneratedStructInfo {
    file_name_without_ext : String,
    struct_name: String
}

//generate entities according to db & table definitions
pub async fn generate_entities(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, db_name:&str, output_path:&Path,boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>){
    let tables = utils::get_tables(conn).await;
    println!("{:#?}",tables);

    //collect what has been generated
    let mut generated_entities:Vec<GeneratedStructInfo> = Vec::new();

    match tables {
        Ok(tables) => {
            for table in tables {
                let generated_entity_info = generate_entity(conn, table, output_path, boolean_columns, trait_for_enum_types).await;
                generated_entities.push(generated_entity_info);
            }
            println!("entities generated successfully");
        }
        Err(error) => {
            println!("unable to generate entities, error: {:#?}",error);
        }
    }

    //generate a mod.rs
    let out_file = output_path.join("mod.rs");

    utils::prepare_directory(&out_file);
    // Open the file for writing
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&out_file)
        .expect("Failed to open mod.rs for writing");

    let mut buf_writer = BufWriter::new(file);

    writeln!(buf_writer,"pub mod enums;").expect("Failed to write entity/mod.rs");
    for generated_entity_info in generated_entities {
        writeln!(buf_writer,"pub mod {};",generated_entity_info.file_name_without_ext).expect("Failed to write entity/mod.rs");
        writeln!(buf_writer,"pub use {}::{};",generated_entity_info.file_name_without_ext,generated_entity_info.struct_name).expect("Failed to write entity/mod.rs");
    }
}

async fn generate_entity(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, table: TableRow, output_path:&Path,
                        boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>) -> GeneratedStructInfo{
    let struct_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&table.name));
    let fields_result = utils::get_table_fields(conn, &table.name).await;
    let out_file = output_path.join(format!("{}.rs", table.name));

    utils::prepare_directory(&out_file);
    // Open the file for writing
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&out_file)
        .expect("Failed to open file for writing");

    let mut buf_writer = BufWriter::new(file);

    let mut items_to_be_imported = vec!["serde::Deserialize".to_string(), "serde::Serialize".to_string()];
    let mut struct_fields = vec![];
    //let mut primary_key = String::new();

    match fields_result {
        Ok(fields) => {
            for it in fields {
                let field_name = if utils::reserved_field_names().contains(&it.name) { format!("{}_", it.name) } else { it.name };
                let field_definition: String = it.data_type;
                let nullable = it.nullable;
                let is_primary_key = it.is_primary_key;
                let field_type = resolve_type_from_column_definition(&table.name, &field_name, &field_definition,boolean_columns, trait_for_enum_types, output_path);
                let field_type_qualified_name = field_type.qualified_name;

                if let Some(import) = field_type.import {
                    if !import.is_empty() && !items_to_be_imported.contains(&import) {
                        items_to_be_imported.push(import);
                    }
                }

                let mut struct_field_definition = format!("{}:{},",field_name,field_type_qualified_name);
                struct_fields.push(struct_field_definition);
            }
        }
        Err(error) => {
            println!("unable to get fields of table {}, error: {:#?}", table.name, error);
        }
    }

    for import in items_to_be_imported{
        writeln!(buf_writer,"use {};",import).expect("Failed to write entity code");
    }

    writeln!(buf_writer,"\n#[derive(Serialize,Deserialize)]").expect("Failed to write entity code");
    writeln!(buf_writer,"pub struct {} {{", struct_name).expect("Failed to write entity code");

    for field in struct_fields{
        writeln!(buf_writer,"    {}",field).expect("Failed to write field definition code");
    }

    writeln!(buf_writer,"}}").expect("Failed to write entity code");

    drop(buf_writer);

    GeneratedStructInfo{
        file_name_without_ext : table.name,
        struct_name: struct_name
    }
}

enum RustDataType {
    String,
    Enum,
    Vec,
    i8,
    i16,
    i32,
    i64,
    u64,
    f64,
    f32,
    u8,//byte
    chronoNaiveDate,
    chronoNaiveTime,
    chronoNaiveDateTime,
}

impl RustDataType {
    fn resolve_qualified_type_name(&self, containerType:Option<RustDataType>, enumName:Option<&str>) -> String {
        let type_str = match self {
            RustDataType::String => "String",
            RustDataType::Enum => enumName.unwrap_or(""),
            RustDataType::Vec => enumName.unwrap_or(""),
            RustDataType::i8 => "i8",
            RustDataType::i16 => "i16",
            RustDataType::i32 => "i32",
            RustDataType::i64 => "i64",
            RustDataType::u64 => "u64",
            RustDataType::f64 => "f64",
            RustDataType::f32 => "f32",
            RustDataType::u8 => "u8",
            RustDataType::chronoNaiveDate => "chrono::NaiveDate",
            RustDataType::chronoNaiveTime => "chrono::NaiveTime",
            RustDataType::chronoNaiveDateTime => "chrono::NaiveDateTime",
        };
        match containerType {
            Some(RustDataType::Vec)  => format!("Vec<{}>",type_str),
            _ => type_str.to_string()
        }
    }
}

#[derive(Debug)]
enum MysqlDataType {
    Char,
    Varchar,
    Tinytext,
    Text,
    Mediumtext,
    Longtext,
    Enum,
    Set,
    Tinyint,
    Smallint,
    Int,
    Bigint,
    BigintUnsigned,
    Numeric,
    Float,
    Double,
    Decimal,
    Date,
    Time,
    DateTime,
    Timestamp,
    Year,
    Blob,
    Json
}
struct MysqlDataTypeProp {
    rust_type:  RustDataType,
    is_conditional_type: bool,
    container_type: Option<RustDataType>,
    import: Option<String>
}
impl FromStr for MysqlDataType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "CHAR" => Ok(MysqlDataType::Char),
            "VARCHAR" => Ok(MysqlDataType::Varchar),
            "TINYTEXT" => Ok(MysqlDataType::Tinytext),
            "TEXT" => Ok(MysqlDataType::Text),
            "MEDIUMTEXT" => Ok(MysqlDataType::Mediumtext),
            "LONGTEXT" => Ok(MysqlDataType::Longtext),
            "ENUM" => Ok(MysqlDataType::Enum),
            "SET" => Ok(MysqlDataType::Set),
            "TINYINT" => Ok(MysqlDataType::Tinyint),
            "SMALLINT" => Ok(MysqlDataType::Smallint),
            "INT" => Ok(MysqlDataType::Int),
            "BIGINT" => Ok(MysqlDataType::Bigint),
            "BIGINT_UNSIGNED" => Ok(MysqlDataType::BigintUnsigned),
            "NUMERIC" => Ok(MysqlDataType::Numeric),
            "FLOAT" => Ok(MysqlDataType::Float),
            "DOUBLE" => Ok(MysqlDataType::Double),
            "DECIMAL" => Ok(MysqlDataType::Decimal),
            "DATE" => Ok(MysqlDataType::Date),
            "TIME" => Ok(MysqlDataType::Time),
            "DATETIME" => Ok(MysqlDataType::DateTime),
            "TIMESTAMP" => Ok(MysqlDataType::Timestamp),
            "YEAR" => Ok(MysqlDataType::Year),
            "BLOB" => Ok(MysqlDataType::Blob),
            "JSON" => Ok(MysqlDataType::Json),
            _ => bail!("Unknown MysqlDataType"),
        }
    }
}
impl MysqlDataType {
    fn properties(&self) -> MysqlDataTypeProp {
        match self {
            MysqlDataType::Char => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Varchar => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Tinytext => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Text => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Mediumtext => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Longtext => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Enum => MysqlDataTypeProp {
                rust_type: RustDataType::Enum,
                is_conditional_type: true,
                container_type: None,
                import:None
            },
            MysqlDataType::Set => MysqlDataTypeProp {
                rust_type: RustDataType::Enum,
                is_conditional_type: true,
                container_type: Some(RustDataType::Vec),
                import:None
            },
            MysqlDataType::Tinyint => MysqlDataTypeProp {
                rust_type: RustDataType::i8,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Smallint => MysqlDataTypeProp {
                rust_type: RustDataType::i16,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Int => MysqlDataTypeProp {
                rust_type: RustDataType::i32,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Bigint => MysqlDataTypeProp {
                rust_type: RustDataType::i64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::BigintUnsigned => MysqlDataTypeProp {
                rust_type: RustDataType::u64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Numeric => MysqlDataTypeProp {
                rust_type: RustDataType::f64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Float => MysqlDataTypeProp {
                rust_type: RustDataType::f32,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Double => MysqlDataTypeProp {
                rust_type: RustDataType::f64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Decimal => MysqlDataTypeProp {
                rust_type: RustDataType::f64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Date => MysqlDataTypeProp {
                rust_type: RustDataType::chronoNaiveDate,
                is_conditional_type: false,
                container_type: None,
                import:Some("chrono".to_string())
            },
            MysqlDataType::Time => MysqlDataTypeProp {
                rust_type: RustDataType::chronoNaiveTime,
                is_conditional_type: false,
                container_type: None,
                import:Some("chrono".to_string())
            },
            MysqlDataType::DateTime => MysqlDataTypeProp {
                rust_type: RustDataType::chronoNaiveDateTime,
                is_conditional_type: false,
                container_type: None,
                import:Some("chrono".to_string())
            },
            MysqlDataType::Timestamp => MysqlDataTypeProp {
                rust_type: RustDataType::chronoNaiveDateTime,
                is_conditional_type: false,
                container_type: None,
                import:Some("chrono".to_string())
            },
            MysqlDataType::Year => MysqlDataTypeProp {
                rust_type: RustDataType::i32,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            MysqlDataType::Blob => MysqlDataTypeProp {
                rust_type: RustDataType::u8,
                is_conditional_type: false,
                container_type: Some(RustDataType::Vec),
                import:None
            },
            MysqlDataType::Json => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            }
        }
    }
}


//convert mysql data field type to rust type
fn resolve_type_from_column_definition(table_name: &str, column_name: &str, column_definition: &str,boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>, generated_code_dir: &Path) -> StructFieldType {
    let definition_array: Vec<&str> = column_definition.split('(').collect();
    let data_type = definition_array[0].replace(" ", "_").to_uppercase();
    let mut field_type_qualified_name = "".to_string();
    //let mut container_struct = "";
    let mut is_primitive_type: bool;

    match data_type.parse::<MysqlDataType>() {
        Ok(mysql_data_type) => {
            let mut mysql_data_type_prop = mysql_data_type.properties();
            //container_struct = mysql_data_type_prop.container_type;
            is_primitive_type = match mysql_data_type_prop.import {
                None => false,
                Some(_) => true
            };

            if mysql_data_type_prop.is_conditional_type {
                match mysql_data_type {
                    MysqlDataType::Tinyint => {
                        field_type_qualified_name = if boolean_columns.contains_key(table_name){
                            "bool".to_string()
                        }else{
                            mysql_data_type_prop.rust_type.resolve_qualified_type_name(None, None)
                        };
                    }
                    MysqlDataType::Enum | MysqlDataType::Set => {
                        is_primitive_type = false;
                        let enum_name = &generate_and_get_enum_name(&table_name, &column_name, &column_definition, trait_for_enum_types, generated_code_dir);
                        mysql_data_type_prop.import = Some(format!("enums::{}",enum_name));
                        field_type_qualified_name = mysql_data_type_prop.rust_type.resolve_qualified_type_name(mysql_data_type_prop.container_type, Some(enum_name))
                    }
                    _ => {}
                }
            }else{
                field_type_qualified_name = mysql_data_type_prop.rust_type.resolve_qualified_type_name(None, None)
            }
            StructFieldType{
                qualified_name: field_type_qualified_name,
                is_primitive_type,
                import : mysql_data_type_prop.import
                //generic_type_qualified_name: container_struct.to_string(),
            }
        }
        Err(_) => {
            panic!("{}.{} {} is not supported", table_name, column_name, column_definition);
        }
    }
}

fn get_enum_name(table_name: &str, column_name: &str) -> String {
    format!("{}{}", stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)), stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(column_name)))
}

fn get_qualified_enum_key(enum_value: &str, unsupported_char_in_enum_key: &HashSet<&str>) -> String {
    let mut qualified_key = enum_value.to_string();
    let is_numeric = enum_value.chars().next().unwrap().is_numeric();

    if is_numeric {
        qualified_key = format!("_{}", enum_value);
    }/* else {
        for un_supported in unsupported_char_in_enum_key.iter() {
            if qualified_key.contains(un_supported) {
                qualified_key = format!("_{}", enum_value);
                break;
            }
        }
    }*/

    qualified_key
}

fn generate_and_get_enum_name(table_name: &str, column_name: &str, column_definition: &str, trait_for_enum_types: &HashMap<&str, &str>, generated_code_dir:&Path) -> String {
    let enum_name = get_enum_name(table_name, column_name);
    let enum_dir = generated_code_dir.join("enums");
    generate_enum(&enum_name, column_definition, table_name, column_name, &enum_dir, trait_for_enum_types);
    enum_name
    //format!("{}.{}", get_package_from_directory(generated_enum_path), enum_class_name)
}

fn generate_enum(enum_name: &str, column_definition: &str, table_name: &str, column_name: &str, enum_dir: &std::path::PathBuf, trait_for_enum_types: &HashMap<&str, &str>) {
    let file_path = enum_dir.join(format!("{}.rs",enum_name));
    utils::prepare_directory(&file_path);
    // Open the file for writing
    let file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(&file_path)
    .expect("Failed to open file for writing");

    let mut buf_writer = BufWriter::new(file);

    let mut enum_code_lines: Vec<String> = Vec::new();
    let mut enum_to_string_code_lines: Vec<String> = Vec::new();
    let mut enum_from_string_code_lines: Vec<String> = Vec::new();

    enum_code_lines.push("use serde::{{Serialize, Deserialize}};\n".to_string());
    enum_code_lines.push("#[derive(Serialize,Deserialize)]".to_string());
    enum_code_lines.push(format!("pub enum {} {{", enum_name));

    enum_to_string_code_lines.push(format!("impl From<{}> for String {{",enum_name));
    enum_to_string_code_lines.push(format!("    fn from(item: {}) -> Self {{",enum_name));
    enum_to_string_code_lines.push("        match item {".to_string());

    enum_from_string_code_lines.push(format!("impl From<&str> for {} {{",enum_name));
    enum_from_string_code_lines.push("    fn from(s: &str) -> Self {{".to_string());
    enum_from_string_code_lines.push("        match s {".to_string());
    
    let enum_definition = column_definition.replace("'", "").replace("set(","").replace("enum(","").replace(")","");
    let enum_items: Vec<&str> = enum_definition.split(',').collect();

    for (index, item) in enum_items.iter().enumerate() {
        let enum_item = get_qualified_enum_key(item, &HashSet::new());
        enum_code_lines.push(format!("    {},", enum_item));
        enum_to_string_code_lines.push(format!("            {}::{} => \"{}\".to_string(),", enum_name, enum_item,enum_item));
        enum_from_string_code_lines.push(format!("            \"{}\" => {}::{},",enum_item, enum_name, enum_item));
    }
    enum_code_lines.push("}}\n".to_string());

    enum_to_string_code_lines.push("        }}".to_string());
    enum_to_string_code_lines.push("    }}".to_string());
    enum_to_string_code_lines.push("}}\n".to_string());

    enum_from_string_code_lines.push("        }}".to_string());
    enum_from_string_code_lines.push("    }}".to_string());
    enum_from_string_code_lines.push("}}".to_string());

    for line in enum_code_lines {
        writeln!(buf_writer,"{}",line).expect("Failed to write enum code");
    }

    for line in enum_to_string_code_lines {
        writeln!(buf_writer,"{}",line).expect("Failed to write enum to String code");
    }

    for line in enum_from_string_code_lines {
        writeln!(buf_writer,"{}",line).expect("Failed to write enum from String code");
    }

    buf_writer.flush().expect("Failed to flush buffer");

    drop(buf_writer);

}

