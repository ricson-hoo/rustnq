use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::println;
use std::path::{Path};
use sqlx::{AnyConnection, AnyPool, Pool};
use sqlx_mysql::MySql;
use crate::codegen::entity::GeneratedStructInfo;
use crate::codegen::utils;
use crate::codegen::utils::{prepare_directory, TableRow};
use crate::mapping::description::{MysqlColumnType, Column, TableFieldConstructInfo, MysqlColumnDefinition};
use crate::utils::stringUtils;
use std::any::Any;
use std::error::Error;

//generate table mappings to db & table definitions
pub async fn generate_mappings(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, db_name:&str, mappings_out_dir:&str, name_of_crate_holds_enums: String, boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>){

    let mappings_out_path = std::path::Path::new(&mappings_out_dir);
    prepare_directory(mappings_out_path);

    let tables = utils::get_tables(conn).await;
    println!("{:#?}",tables);

    //collect what has been generated
    let mut generated_entities:Vec<GeneratedStructInfo> = Vec::new();

    match tables {
        Ok(tables) => {
            for table in tables {
                let generated_entity_info = generate_mapping(conn, table, mappings_out_path, name_of_crate_holds_enums.clone(), boolean_columns, trait_for_enum_types).await;
                generated_entities.push(generated_entity_info);
            }
            println!("entities generated successfully");
        }
        Err(error) => {
            println!("unable to generate entities, error: {:#?}",error);
        }
    }

    //generate a mod.rs
    let out_file = mappings_out_path.join("mod.rs");

    utils::prepare_directory(&out_file);
    // Open the file for writing
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&out_file)
        .expect("Failed to open mod.rs for writing");

    let mut buf_writer = BufWriter::new(file);

    for generated_entity_info in generated_entities {
        writeln!(buf_writer,"pub mod {};",generated_entity_info.file_name_without_ext).expect("Failed to write entity/mod.rs");
        writeln!(buf_writer,"pub use {}::{};",generated_entity_info.file_name_without_ext,generated_entity_info.struct_name).expect("Failed to write entity/mod.rs");
    }
    // Remember to flush the buffer to ensure all data is written to the file
    buf_writer.flush().expect("Failed to flush buffer");

}

async fn generate_mapping(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, table: TableRow, output_path:&Path, name_of_crate_holds_enums: String,
                          boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>) -> GeneratedStructInfo{
    let struct_name = format!("{}Table",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&table.name)));
    let fields_result = utils::get_table_fields(conn, &table.name).await;
    let out_file_name_without_ext = format!("{}Table",stringUtils::to_camel_case(&table.name));
    let out_file = output_path.join(format!("{}.rs", out_file_name_without_ext));

    utils::prepare_directory(&out_file);
    // Open the file for writing
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&out_file)
        .expect("Failed to open file for writing");

    let mut buf_writer = BufWriter::new(file);

    let mut items_to_be_imported : Vec<String> = vec!["serde::Deserialize".to_string(), "serde::Serialize".to_string()];
    items_to_be_imported.push("rustnq::mapping::types::Table".to_string());
    let mut struct_fields = vec![];
    let mut instance_fields = vec![];
    //let mut primary_key = String::new();

    match fields_result {
        Ok(fields) => {
            for it in fields {
                let column_name = if utils::reserved_field_names().contains(&it.name) { format!("{}_", it.name) } else { it.name.clone() };
                let field_definition: String = it.data_type;
                let mysql_cloumn_definition = MysqlColumnDefinition{
                    name:column_name.clone(),
                    name_unmodified : it.name,
                    column_definition:field_definition,
                    default_value:"".to_string() //all empty for now
                };
                let columnConstructInfo:TableFieldConstructInfo = get_construct_info_from_column_definition(&table.name,mysql_cloumn_definition, name_of_crate_holds_enums.clone()).expect(&format!("Failed to get construct info from table {}",table.name));

                if !columnConstructInfo.import_statements.is_empty() {
                    for import_statement in &columnConstructInfo.import_statements {
                       if !items_to_be_imported.contains(&import_statement){
                           items_to_be_imported.push(import_statement.to_string());
                       }
                    }
                }

                struct_fields.push(format!("pub {}:{},",&column_name,columnConstructInfo.file_type));
                instance_fields.push(format!("{}:{},",&column_name,columnConstructInfo.default_value_on_new));
            }
        }
        Err(error) => {
            println!("unable to get fields of table {}, error: {:#?}", table.name, error);
        }
    }

    for import in items_to_be_imported{
        writeln!(buf_writer,"use {};",import).expect("Failed to table mapping code");
    }

    writeln!(buf_writer,"\n#[derive(Serialize,Deserialize,Clone,Debug)]").expect("Failed to table mapping code");
    writeln!(buf_writer,"pub struct {} {{", struct_name).expect("Failed to table mapping code");

    for field in struct_fields{
        writeln!(buf_writer,"    {}",field).expect("Failed to table mapping code");
    }

    writeln!(buf_writer,"}}").expect("Failed to table mapping code");

    writeln!(buf_writer,"impl {} {{", struct_name).expect("Failed to table mapping code");
    writeln!(buf_writer,"    pub fn new() ->Self {{").expect("Failed to table mapping code");
    writeln!(buf_writer,"        {} {{",struct_name).expect("Failed to table mapping code");
    for field in instance_fields{
        writeln!(buf_writer,"            {}",field).expect("Failed to table mapping code");
    }
    writeln!(buf_writer,"        }}").expect("Failed to table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to table mapping code");
    writeln!(buf_writer,"}}").expect("Failed to table mapping code");


    writeln!(buf_writer,"impl Table for {} {{", struct_name).expect("Failed to table mapping code");
    writeln!(buf_writer,"    fn name(&self) -> String {{").expect("Failed to table mapping code");
    writeln!(buf_writer,"        \"{}\".to_string()",&table.name).expect("Failed to table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to table mapping code");
    writeln!(buf_writer,"}}").expect("Failed to table mapping code");

    drop(buf_writer);

    GeneratedStructInfo{
        file_name_without_ext : out_file_name_without_ext,
        struct_name,
        enum_file_names_without_ext : vec![]
    }
}


pub fn get_construct_info_from_column_definition(table_name:&str, mysql_col_definition:MysqlColumnDefinition, name_of_crate_holds_enums: String) -> Result<TableFieldConstructInfo,Box<dyn Error>>{

    let col_definition = mysql_col_definition.column_definition;
    let mut column_type_name = "".to_string();
    if !col_definition.contains("("){
        column_type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&col_definition.replace(" ", "_")));
    }else{
        let parts = col_definition.split("(").map(|s| s.to_string()).collect::<Vec<String>>();
        column_type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&parts[0]));
    }
    let column_type_parse_result = column_type_name.parse::<MysqlColumnType>();
    let column_name = mysql_col_definition.name;
    let mut file_type = "".to_string();
    let mut import_type = "".to_string();
    let mut default_value = "".to_string();
    let mut import_statements : Vec<String> = vec![];

    match column_type_parse_result{
        Ok(column_type) => {
            match column_type {
                MysqlColumnType::Varchar => {
                    file_type = "Varchar".to_string();
                    default_value = format!("Varchar::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Char => {
                    file_type = "Char".to_string();
                    default_value = format!("Char::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Tinytext => {
                    file_type = "Tinytext".to_string();
                    default_value = format!("Tinytext::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Text => {
                    file_type = "Text".to_string();
                    default_value = format!("Text::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Mediumtext => {
                    file_type = "Mediumtext".to_string();
                    default_value = format!("Mediumtext::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Longtext => {
                    file_type = "Longtext".to_string();
                    default_value = format!("Longtext::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Int => {
                    file_type = "Int".to_string();
                    default_value = format!("Int::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Year => {
                    file_type = "Year".to_string();
                    default_value = format!("Year::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Enum => {
                    let short_enum_name = format!("{}{}",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&column_name)));
                    let mut enumType = format!("entity::enums::{}",&short_enum_name);
                    if !name_of_crate_holds_enums.is_empty(){
                        enumType = format!("{}::{}",name_of_crate_holds_enums, enumType);
                    }
                    file_type = format!("Enum<{}>",short_enum_name);
                    import_type = "Enum".to_string();
                    default_value = format!("Enum::<{}>::new(\"{}\".to_string())", short_enum_name, mysql_col_definition.name_unmodified);
                    //import enum
                    import_statements.push(enumType);
                },
                MysqlColumnType::Set => {
                    let short_enum_name = format!("{}{}",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&column_name)));
                    let mut enumType = format!("entity::enums::{}",&short_enum_name);
                    if !name_of_crate_holds_enums.is_empty(){
                        enumType = format!("{}::{}",name_of_crate_holds_enums, enumType);
                    }
                    file_type = format!("Set<{}>",short_enum_name);
                    import_type = "Set".to_string();
                    default_value = format!("Set::<{}>::new(\"{}\".to_string())", short_enum_name, mysql_col_definition.name_unmodified);
                    //import enum
                    import_statements.push(enumType);
                },
                MysqlColumnType::Datetime => {
                    file_type = "Datetime".to_string();
                    default_value = format!("Datetime::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Timestamp => {
                    file_type = "Timestamp".to_string();
                    default_value = format!("Timestamp::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Tinyint => {
                    file_type = "Tinyint".to_string();
                    default_value = format!("Tinyint::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Smallint => {
                    file_type = "Smallint".to_string();
                    default_value = format!("Smallint::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Bigint => {
                    file_type = "Bigint".to_string();
                    default_value = format!("Bigint::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::BigintUnsigned => {
                    file_type = "BigintUnsigned".to_string();
                    default_value = format!("BigintUnsigned::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Numeric => {
                    file_type = "Numeric".to_string();
                    default_value = format!("Numeric::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Float => {
                    file_type = "Float".to_string();
                    default_value = format!("Float::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Double => {
                    file_type = "Double".to_string();
                    default_value = format!("Double::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Decimal => {
                    file_type = "Varchar".to_string();
                    default_value = format!("Varchar::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Date => {
                    file_type = "Date".to_string();
                    default_value = format!("Date::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Time => {
                    file_type = "Time".to_string();
                    default_value = format!("Time::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Blob => {
                    file_type = "Blob".to_string();
                    default_value = format!("Blob::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
                MysqlColumnType::Json => {
                    file_type = "Json".to_string();
                    default_value = format!("Json::new(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                },
            }
        },
        Err(_) => {

        }
    };
    //add current type to import statements
    import_statements.push(format!("rustnq::mapping::types::{}",if import_type.is_empty() {&file_type} else {&import_type}));

    Ok(TableFieldConstructInfo{
        field_name : column_name,
        file_type:file_type,
        default_value_on_new:default_value,
        import_statements: import_statements
    })

}