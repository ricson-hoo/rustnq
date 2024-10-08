use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::println;
use std::path::{Path};
use sqlx::{AnyConnection, AnyPool, Pool};
use sqlx_mysql::MySql;
use crate::codegen::entity::GeneratedStructInfo;
use crate::codegen::utils;
use crate::codegen::utils::TableRow;
use crate::mapping::description::{ColumnType, Column, TableFieldConstructInfo, get_construct_info_from_column_definition, MysqlColumnDefinition};
use crate::utils::stringUtils;
use std::any::Any;

//generate table mappings to db & table definitions
pub async fn generate_mappings(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, db_name:&str, output_path:&Path, name_of_crate_holds_enums: String, boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>){
    let tables = utils::get_tables(conn).await;
    println!("{:#?}",tables);

    //collect what has been generated
    let mut generated_entities:Vec<GeneratedStructInfo> = Vec::new();

    match tables {
        Ok(tables) => {
            for table in tables {
                let generated_entity_info = generate_mapping(conn, table, output_path, name_of_crate_holds_enums.clone(), boolean_columns, trait_for_enum_types).await;
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

    let mut items_to_be_imported : Vec<String> = Vec::new();
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

                struct_fields.push(format!("{}:{},",&column_name,columnConstructInfo.file_type));
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

    writeln!(buf_writer,"\n#[derive(Serialize,Deserialize)]").expect("Failed to table mapping code");
    writeln!(buf_writer,"pub struct {} {{", struct_name).expect("Failed to table mapping code");

    for field in struct_fields{
        writeln!(buf_writer,"    {}",field).expect("Failed to table mapping code");
    }

    writeln!(buf_writer,"}}").expect("Failed to table mapping code");

    writeln!(buf_writer,"impl {} {{", struct_name).expect("Failed to table mapping code");
    writeln!(buf_writer,"    pub fn new(&self) ->Self {{").expect("Failed to table mapping code");
    writeln!(buf_writer,"        {} {{",struct_name).expect("Failed to table mapping code");
    for field in instance_fields{
        writeln!(buf_writer,"            {}",field).expect("Failed to table mapping code");
    }
    writeln!(buf_writer,"        }}").expect("Failed to table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to table mapping code");
    writeln!(buf_writer,"}}").expect("Failed to table mapping code");

    drop(buf_writer);

    GeneratedStructInfo{
        file_name_without_ext : out_file_name_without_ext,
        struct_name
    }
}

//convert mysql data field type to a type Description in rust
/*fn resolve_type_from_column_definition<'a>(table_name: &str, column_name: &str, column_definition: &str,boolean_columns: &HashMap<String, HashSet<String>>,
                                              trait_for_enum_types: &HashMap<&str, &str>, generated_code_dir: &Path) -> ColumnType<'a> {
    let definition_array: Vec<&str> = column_definition.split('(').collect();
    let column_type = definition_array[0].replace(" ", "_").to_uppercase();
    let column_type_and_name = format!("{},{}", column_type, column_name);
    //let mut field_type_qualified_name = "".to_string();
    //let mut container_struct = "";
    //let mut is_primitive_type: bool;

    match column_type_and_name.parse::<ColumnType>() {
        Ok(col_info) => {
            col_info
        }
        Err(_) => {
            panic!("{}.{} {} is not supported", table_name, column_name, column_definition);
        }
    }
}*/
