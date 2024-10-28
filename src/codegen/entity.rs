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
use crate::codegen::utils::{format_name, TableRow};
use crate::utils::stringUtils;
use serde::{Serialize, Deserialize};
use crate::mapping::description::{RustDataType, SqlColumn};
use crate::query::builder::Condition;

struct StructFieldType {
    qualified_name: String,
    is_primitive_type: bool,
    import:Option<String>,
    enum_file_name_without_ext: String
}

pub(crate) struct GeneratedStructInfo {
    pub file_name_without_ext : String,
    pub struct_name: String,
    pub enum_file_names_without_ext: Vec<String>,//camelCase
}

#[derive(Clone, Copy)]
pub enum FieldNamingConvention {
    CamelCase,
    SnakeCase,
    PascalCase
}

pub struct EntityGenerateConfig{
    pub output_dir:String,
    pub naming_convention: FieldNamingConvention,
    pub boolean_columns: HashMap<String, HashSet<String>>,
    pub trait_for_enum_types: HashMap<String, String>
}

impl EntityGenerateConfig{
    pub fn new(output_dir:String, naming_convention: FieldNamingConvention, boolean_columns: HashMap<String, HashSet<String>>, trait_for_enum_types: HashMap<String, String>)->Self{
        EntityGenerateConfig{
            output_dir,
            naming_convention,
            boolean_columns,
            trait_for_enum_types
        }
    }
    pub fn default()->Self{
        EntityGenerateConfig{
            output_dir : "target/generated/entity".to_string(),
            naming_convention: FieldNamingConvention::SnakeCase,
            boolean_columns:HashMap::new(),
            trait_for_enum_types:HashMap::new()
        }
    }

    pub fn default_with_naming_convention(naming_convention:FieldNamingConvention)->Self{
        EntityGenerateConfig{
            output_dir : "target/generated/entity".to_string(),
            naming_convention,
            boolean_columns:HashMap::new(),
            trait_for_enum_types:HashMap::new()
        }
    }
}


//generate entities according to db & table definitions
pub async fn generate_entities(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, db_name:&str, config:EntityGenerateConfig){/*entity_out_dir:&str, boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>*/
    let entity_out_dir = config.output_dir.clone();
    let boolean_columns = config.boolean_columns.clone();
    let mut trait_for_enum_types = config.trait_for_enum_types.clone();
    //trait_for_enum_types.insert("*".to_string(), "EntityEnum".to_string());
    let naming_convention = config.naming_convention;

    let entity_out_path = std::path::Path::new(&entity_out_dir);
    utils::prepare_directory(entity_out_path);

    let tables = utils::get_tables(conn).await;
    //println!("{:#?}",tables);

    //collect what has been generated
    let mut generated_entities:Vec<GeneratedStructInfo> = Vec::new();

    match tables {
        Ok(tables) => {
            for table in tables {
                let generated_entity_info = generate_entity(conn, table, entity_out_path, &boolean_columns, &trait_for_enum_types, naming_convention).await;
                generated_entities.push(generated_entity_info);
            }
            println!("entities generated successfully");
        }
        Err(error) => {
            println!("unable to generate entities, error: {:#?}",error);
        }
    }

    //generate mod.rs && /enums.rs
    let entity_mod_out_file = entity_out_path.join("mod.rs");
    let entity_enum_mod_out_file = entity_out_path.join("enums/mod.rs");

    utils::prepare_directory(&entity_mod_out_file);
    // Open the file for writing
    let entity_mod_out_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&entity_mod_out_file)
        .expect("Failed to open mod.rs for writing");

    let entity_enum_mod_out_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&entity_enum_mod_out_file)
        .expect("Failed to open mod.rs for writing");

    let mut entity_mod_out_file_buf_writer = BufWriter::new(entity_mod_out_file);
    let mut entity_enum_mod_out_file_buf_writer = BufWriter::new(entity_enum_mod_out_file);

    writeln!(entity_enum_mod_out_file_buf_writer,"#[allow(non_snake_case)]").expect("Failed to write entity/enum/mod.rs");
    writeln!(entity_mod_out_file_buf_writer,"#[allow(non_snake_case)]").expect("Failed to write entity/mod.rs");
    writeln!(entity_mod_out_file_buf_writer,"pub mod enums;").expect("Failed to write entity/mod.rs");

    for generated_entity_info in generated_entities {
        writeln!(entity_mod_out_file_buf_writer,"pub mod {};",format_name(&generated_entity_info.file_name_without_ext, naming_convention)).expect("Failed to write entity/mod.rs");
        writeln!(entity_mod_out_file_buf_writer,"pub use {}::{};",format_name(&generated_entity_info.file_name_without_ext, naming_convention),stringUtils::begin_with_upper_case(&generated_entity_info.struct_name)).expect("Failed to write entity/mod.rs");
        if !generated_entity_info.enum_file_names_without_ext.is_empty(){
            for enum_file_name in generated_entity_info.enum_file_names_without_ext{
                writeln!(entity_enum_mod_out_file_buf_writer,"pub mod {};",enum_file_name).expect("Failed to write entity/enum/mod.rs");
                writeln!(entity_enum_mod_out_file_buf_writer,"pub use {}::{};",enum_file_name,stringUtils::begin_with_upper_case(&enum_file_name)).expect("Failed to write entity/mod.rs");
            }
        }
    }

    //define traits for enums
    let mut added_enum_traits:Vec<String> = vec![];
    for (pattern, enum_trait) in trait_for_enum_types{
        if !added_enum_traits.contains(&enum_trait.to_string()){
            writeln!(entity_enum_mod_out_file_buf_writer,"pub trait {} {{}}",enum_trait).expect("Failed to write entity/enum/mod.rs");
            added_enum_traits.push(enum_trait.to_string());
        }
    }

    // Remember to flush the buffer to ensure all data is written to the file
    entity_mod_out_file_buf_writer.flush().expect("Failed to flush buffer");
    // Remember to flush the buffer to ensure all data is written to the file
    entity_enum_mod_out_file_buf_writer.flush().expect("Failed to flush buffer");
}

async fn generate_entity(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, table: TableRow, output_path:&Path,
                        boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<String, String>, naming_convention:FieldNamingConvention) -> GeneratedStructInfo{
    let struct_name = stringUtils::begin_with_upper_case(&format_name(&table.name, naming_convention));
    let fields_result = utils::get_table_fields(conn, &table.name).await;
    let out_file = output_path.join(format!("{}.rs", format_name(&table.name, naming_convention)));

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
    let mut struct_fields_init = vec![];
    let mut enum_file_names_without_ext = vec![];
    //let mut primary_key = String::new();

    match fields_result {
        Ok(fields) => {
            for it in fields {
                //println!("field_name : {}",it.name);
                //println!("utils::reserved_field_names() : {:#?}",utils::reserved_field_names());
                //println!("utils::reserved_field_names().contains : {}",utils::reserved_field_names().contains(&it.name));
                let field_name = it.name.clone();
                let field_definition: String = it.data_type;
                let nullable = it.nullable;
                let is_primary_key = it.is_primary_key;
                let field_type = resolve_type_from_column_definition(&table.name, &field_name, &field_definition, boolean_columns, trait_for_enum_types, output_path);
                if !field_type.enum_file_name_without_ext.is_empty() {
                    enum_file_names_without_ext.push(field_type.enum_file_name_without_ext);
                }
                let field_type_qualified_name = field_type.qualified_name;

                if let Some(import) = field_type.import {
                    if !import.is_empty() && !items_to_be_imported.contains(&import) {
                        items_to_be_imported.push(import);
                    }
                }

                //here we need to modify the field name if it matchs one of the rust keyword
                if utils::reserved_field_names().contains(&it.name){
                    let mut struct_field_definition = format!("#[serde(rename = \"{}\")] pub {}_:{},",&it.name, &it.name,field_type_qualified_name);
                    struct_fields.push(struct_field_definition);
                    struct_fields_init.push(format!("{}_:{},",&it.name, if field_type_qualified_name.starts_with("Vec") {"vec![]"} else {"None"}));
                }else {
                    //todo: snake_case or camelCase configurable
                    if field_type_qualified_name.clone().contains("NaiveDateTime"){
                        struct_fields.push("#[serde(deserialize_with = \"crate::serde::deserialize_datetime\")]".to_string()); //note:this is a temp solution
                        struct_fields.push("#[serde(serialize_with = \"crate::serde::serialize_datetime\")]".to_string()); //note:this is a temp solution
                    }else if field_type_qualified_name.clone().contains("NaiveDate"){
                        struct_fields.push("#[serde(deserialize_with = \"crate::serde::deserialize_date\")]".to_string()); //note:this is a temp solution
                        struct_fields.push("#[serde(serialize_with = \"crate::serde::serialize_date\")]".to_string()); //note:this is a temp solution
                    }
                    let mut struct_field_definition = format!("pub {}:{},",format_name(&it.name, naming_convention),field_type_qualified_name);
                    struct_fields.push(struct_field_definition);
                    struct_fields_init.push(format!("{}:{},",format_name(&it.name, naming_convention),"None"));
                }
            }
        }
        Err(error) => {
            println!("unable to get fields of table {}, error: {:#?}", table.name, error);
        }
    }

    for import in items_to_be_imported{
        writeln!(buf_writer,"use {};",import).expect("Failed to write entity code");
    }

    writeln!(buf_writer,"\n#[derive(Serialize,Deserialize,Clone,Debug)]").expect("Failed to write entity code");
    writeln!(buf_writer,"#[allow(non_snake_case)]").expect("Failed to write entity code");
    writeln!(buf_writer,"pub struct {}<D = ()> {{", struct_name).expect("Failed to write entity code");

    for field in struct_fields{
        writeln!(buf_writer,"    {}",field).expect("Failed to write field definition code");
    }
    //add a _ext for easy extension
    writeln!(buf_writer,"    pub _associated: Option<D>,").expect("Failed to write field definition code");

    writeln!(buf_writer,"}}").expect("Failed to write entity code");

    //associated new fn
    writeln!(buf_writer,"impl {} {{", struct_name).expect("Failed to write entity impl code");
    writeln!(buf_writer,"    pub fn new() -> {} {{", struct_name).expect("Failed to write entity impl code");
    writeln!(buf_writer,"        {} {{", struct_name).expect("Failed to write entity impl code");
    for field in struct_fields_init{
        writeln!(buf_writer,"            {}",field).expect("Failed to write field definition code");
    }
    writeln!(buf_writer,"            _associated: None,").expect("Failed to write field definition code");
    writeln!(buf_writer,"        }}").expect("Failed to write entity impl code");
    writeln!(buf_writer,"    }}").expect("Failed to write entity impl code");
    writeln!(buf_writer,"}}").expect("Failed to write entity impl code");

    drop(buf_writer);

    GeneratedStructInfo{
        file_name_without_ext : table.name,
        struct_name: struct_name,
        enum_file_names_without_ext: enum_file_names_without_ext
    }
}

impl RustDataType {
    fn resolve_qualified_type_name(&self, containerType:Option<RustDataType>, enumName:Option<&str>) -> String {
        let type_str = match self {
            RustDataType::String => "Option<String>",
            RustDataType::Enum => {
                match containerType {
                    Some(RustDataType::Vec) => &format!("Option<Vec<{}>>",enumName.unwrap_or("")),
                    _ => &format!("Option<{}>",enumName.unwrap_or("")),
                }
            },
            RustDataType::Vec => &format!("Option<Vec<{}>>",enumName.unwrap_or("")),
            RustDataType::i8 => "Option<i8>",
            RustDataType::i16 => "Option<i16>",
            RustDataType::i32 => "Option<i32>",
            RustDataType::i64 => "Option<i64>",
            RustDataType::u64 => "Option<u64>",
            RustDataType::f64 => "Option<f64>",
            RustDataType::f32 => "Option<f32>",
            RustDataType::u8 => {
                match containerType {
                    Some(RustDataType::Vec) => "Option<Vec<u8>>",
                    _ => "Option<u8>",
                }
            }
            RustDataType::chronoNaiveDate => "Option<chrono::NaiveDate>",
            RustDataType::chronoNaiveTime => "Option<chrono::NaiveTime>",
            RustDataType::chronoNaiveDateTime => "Option<chrono::NaiveDateTime>",
        };
        /*match containerType {
            Some(RustDataType::Vec)  => format!("Vec<{}>",type_str),
            _ => type_str.to_string()
        }*/
        type_str.to_string()
    }
}

#[derive(Debug,Clone)]
struct MysqlDataTypeProp {
    rust_type:  RustDataType,
    is_conditional_type: bool,
    container_type: Option<RustDataType>,
    import: Option<String>
}

impl SqlColumn {
    fn properties(&self) -> MysqlDataTypeProp {
        match self {
            SqlColumn::Char(_) => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Varchar(_) => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Tinytext(_) => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Text(_) => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Mediumtext(_) => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Longtext(_) => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Enum(_) => MysqlDataTypeProp {
                rust_type: RustDataType::Enum,
                is_conditional_type: true,
                container_type: None,
                import:None
            },
            SqlColumn::Set(_) => MysqlDataTypeProp {
                rust_type: RustDataType::Vec,
                is_conditional_type: true,
                container_type: Some(RustDataType::Vec),
                import:None
            },
            SqlColumn::Tinyint(_) => MysqlDataTypeProp {
                rust_type: RustDataType::i8,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Smallint(_) => MysqlDataTypeProp {
                rust_type: RustDataType::i16,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Int(_) => MysqlDataTypeProp {
                rust_type: RustDataType::i32,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Bigint(_) => MysqlDataTypeProp {
                rust_type: RustDataType::i64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::BigintUnsigned(_) => MysqlDataTypeProp {
                rust_type: RustDataType::u64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Numeric(_) => MysqlDataTypeProp {
                rust_type: RustDataType::f64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Float(_) => MysqlDataTypeProp {
                rust_type: RustDataType::f32,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Double(_) => MysqlDataTypeProp {
                rust_type: RustDataType::f64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Decimal(_) => MysqlDataTypeProp {
                rust_type: RustDataType::f64,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Date(_) => MysqlDataTypeProp {
                rust_type: RustDataType::chronoNaiveDate,
                is_conditional_type: false,
                container_type: None,
                import:Some("chrono".to_string())
            },
            SqlColumn::Time(_) => MysqlDataTypeProp {
                rust_type: RustDataType::chronoNaiveTime,
                is_conditional_type: false,
                container_type: None,
                import:Some("chrono".to_string())
            },
            SqlColumn::Datetime(_) => MysqlDataTypeProp {
                rust_type: RustDataType::chronoNaiveDateTime,
                is_conditional_type: false,
                container_type: None,
                import:Some("chrono".to_string())
            },
            SqlColumn::Timestamp(_) => MysqlDataTypeProp {
                rust_type: RustDataType::chronoNaiveDateTime,
                is_conditional_type: false,
                container_type: None,
                import:Some("chrono".to_string())
            },
            SqlColumn::Year(_) => MysqlDataTypeProp {
                rust_type: RustDataType::i32,
                is_conditional_type: false,
                container_type: None,
                import:None
            },
            SqlColumn::Blob(_) => MysqlDataTypeProp {
                rust_type: RustDataType::u8,
                is_conditional_type: false,
                container_type: Some(RustDataType::Vec),
                import:None
            },
            SqlColumn::Json(_) => MysqlDataTypeProp {
                rust_type: RustDataType::String,
                is_conditional_type: false,
                container_type: None,
                import:None
            }
        }
    }
}


//convert mysql data field type to rust type
fn resolve_type_from_column_definition(table_name: &str, column_name: &str, column_definition: &str,boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<String, String>, generated_code_dir: &Path) -> StructFieldType {
    let definition_array: Vec<&str> = column_definition.split('(').collect();
    let data_type = definition_array[0];//.replace(" ", "_");
    let mut field_type_qualified_name = "".to_string();
    //let mut container_struct = "";
    let mut is_primitive_type: bool;

    match data_type.parse::<SqlColumn>() {
        Ok(mysql_data_type) => {
            let mut mysql_data_type_prop = mysql_data_type.properties();
            //container_struct = mysql_data_type_prop.container_type;
            is_primitive_type = match mysql_data_type_prop.import {
                None => false,
                Some(_) => true
            };
            let mut enum_file_name_without_ext: String = "".to_string();

            if mysql_data_type_prop.is_conditional_type {
                match mysql_data_type {
                    SqlColumn::Tinyint(_) => {
                        field_type_qualified_name = if boolean_columns.contains_key(table_name){
                            "bool".to_string()
                        }else{
                            mysql_data_type_prop.rust_type.resolve_qualified_type_name(None, None)
                        };
                    }
                    SqlColumn::Enum(_) | SqlColumn::Set(_) => {
                        is_primitive_type = false;
                        let (enum_name, enum_file_name_no_ext) = &generate_and_get_enum_name(&table_name, &column_name, &column_definition, trait_for_enum_types, generated_code_dir);
                        mysql_data_type_prop.import = Some(format!("crate::entity::enums::{}",enum_name));
                        enum_file_name_without_ext = enum_file_name_no_ext.to_string();
                        let prop = mysql_data_type_prop.clone();
                        field_type_qualified_name = mysql_data_type_prop.rust_type.resolve_qualified_type_name(mysql_data_type_prop.container_type, Some(enum_name));
                        //println!("{:#?},{:#?},{:#?},{}",data_type.clone(),mysql_data_type.clone(),prop,field_type_qualified_name.clone());
                    }
                    _ => {}
                }
            }else{
                field_type_qualified_name = mysql_data_type_prop.rust_type.resolve_qualified_type_name(None, None)
            }
            StructFieldType{
                qualified_name: field_type_qualified_name,
                is_primitive_type,
                import : mysql_data_type_prop.import,
                enum_file_name_without_ext
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

fn generate_and_get_enum_name(table_name: &str, column_name: &str, column_definition: &str, trait_for_enum_types: &HashMap<String, String>, generated_code_dir:&Path) -> (String,String) {
    let enum_name = get_enum_name(table_name, column_name);
    let enum_dir = generated_code_dir.join("enums");
    let enum_file_name_without_ext = format!("{}{}",stringUtils::to_camel_case(table_name),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(column_name)));
    generate_enum(&enum_name, column_definition, table_name, column_name, &enum_dir, &enum_file_name_without_ext, trait_for_enum_types);
    (enum_name,enum_file_name_without_ext)
    //format!("{}.{}", get_package_from_directory(generated_enum_path), enum_class_name)
}

fn generate_enum(enum_name: &str, column_definition: &str, table_name: &str, column_name: &str, enum_dir: &std::path::PathBuf, enum_file_name_without_ext: &str, trait_for_enum_types: &HashMap<String, String>) {
    let file_path = enum_dir.join(format!("{}.rs",enum_file_name_without_ext));
    utils::prepare_directory(&file_path);
    // Open the file for writing
    let file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(&file_path)
    .expect("Failed to open file for writing");

    let mut traits_to_be_implemented = HashMap::new();
    //traits_to_be_implemented.insert("EntityEnum", "use rustnq::mapping::types::EntityEnum;".to_string());
    let pattern1 = format!("{}_{}",table_name,column_name);
    let pattern2 = format!("{}*",table_name);
    //let pattern3 = "*".to_string();

    if trait_for_enum_types.contains_key(pattern1.as_str()) {
        traits_to_be_implemented.insert(&*trait_for_enum_types[pattern1.as_str()],format!("use crate::entity::enums::{};", trait_for_enum_types[pattern1.as_str()]));
    }
    if trait_for_enum_types.contains_key(pattern2.as_str()){
        traits_to_be_implemented.insert(&*trait_for_enum_types[pattern2.as_str()],format!("use crate::entity::enums::{};", trait_for_enum_types[pattern2.as_str()]));
    }
    /*if trait_for_enum_types.contains_key(pattern3.as_str()){
        traits_to_be_implemented.push(&*trait_for_enum_types[pattern3.as_str()]);
    }*/

    let mut buf_writer = BufWriter::new(file);

    let mut enum_code_lines: Vec<String> = vec![];
    let mut enum_to_string_code_lines: Vec<String> = Vec::new();
    let mut enum_from_string_code_lines: Vec<String> = Vec::new();
    let mut enum_display_code_lines: Vec<String> = Vec::new();

    if !traits_to_be_implemented.is_empty() {
        for (trt,import_str) in traits_to_be_implemented.clone(){
            enum_code_lines.push(import_str);
        }
    }
    enum_code_lines.push("use serde::{{Serialize, Deserialize}};\n".to_string());
    enum_code_lines.push("use std::fmt;\n".to_string());

    enum_code_lines.push("#[derive(Serialize,Deserialize,Clone,Debug,Copy)]".to_string());
    enum_code_lines.push("#[allow(clippy::upper_case_acronyms)]".to_string());
    enum_code_lines.push("#[allow(non_camel_case_types)]".to_string());

    enum_code_lines.push(format!("pub enum {} {{", enum_name));

    enum_to_string_code_lines.push(format!("impl From<{}> for String {{",enum_name));
    enum_to_string_code_lines.push(format!("    fn from(item: {}) -> Self {{",enum_name));
    enum_to_string_code_lines.push("        match item {".to_string());

    enum_from_string_code_lines.push(format!("impl From<&str> for {} {{",enum_name));
    enum_from_string_code_lines.push("    fn from(s: &str) -> Self {".to_string());
    enum_from_string_code_lines.push("        match s {".to_string());

    enum_display_code_lines.push(format!("impl fmt::Display for {} {{",enum_name));
    enum_display_code_lines.push("    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {".to_string());
    enum_display_code_lines.push("        match self {".to_string());
    
    let enum_definition = column_definition.replace("'", "").replace("set(","").replace("enum(","").replace(")","");
    let enum_items: Vec<&str> = enum_definition.split(',').collect();

    for (index, item) in enum_items.iter().enumerate() {
        let enum_item = get_qualified_enum_key(item, &HashSet::new());
        enum_code_lines.push(format!("    {},", enum_item));
        enum_to_string_code_lines.push(format!("            {}::{} => \"{}\".to_string(),", enum_name, enum_item,enum_item));
        enum_from_string_code_lines.push(format!("            \"{}\" => {}::{},",enum_item, enum_name, enum_item));
        enum_display_code_lines.push(format!("            {}::{} => write!(f,\"{}\"),",enum_name, enum_item, enum_item));
    }

    enum_from_string_code_lines.push("            &_ => todo!(),".to_string());
    enum_code_lines.push("}\n".to_string());

    enum_to_string_code_lines.push("        }".to_string());
    enum_to_string_code_lines.push("    }".to_string());
    enum_to_string_code_lines.push("}\n".to_string());

    enum_from_string_code_lines.push("        }".to_string());
    enum_from_string_code_lines.push("    }".to_string());
    enum_from_string_code_lines.push("}".to_string());

    enum_display_code_lines.push("        }".to_string());
    enum_display_code_lines.push("    }".to_string());
    enum_display_code_lines.push("}".to_string());

    for line in enum_code_lines {
        writeln!(buf_writer,"{}",line).expect("Failed to write enum code");
    }

    for line in enum_to_string_code_lines {
        writeln!(buf_writer,"{}",line).expect("Failed to write enum to String code");
    }

    for line in enum_from_string_code_lines {
        writeln!(buf_writer,"{}",line).expect("Failed to write enum from String code");
    }

    for line in enum_display_code_lines {
        writeln!(buf_writer,"{}",line).expect("Failed to write enum from String code");
    }


    if !traits_to_be_implemented.is_empty() {
        for (trt,import_str) in traits_to_be_implemented {
            writeln!(buf_writer,"impl {} for {} {{}}",trt, enum_name).expect("Failed to write trait code");
        }
    }

    writeln!(buf_writer,"impl {} {{",enum_name).expect("Failed to write enum::values code");
    writeln!(buf_writer,"    pub fn values() -> Vec<{}> {{",enum_name);
    write!(buf_writer,"        vec![");
    for (index, item) in enum_items.iter().enumerate() {
        write!(buf_writer,"{}::{},",enum_name,item);
    }
    write!(buf_writer,"]\n");
    writeln!(buf_writer,"    }}");
    writeln!(buf_writer,"}}");

    buf_writer.flush().expect("Failed to flush buffer");

    drop(buf_writer);

}

