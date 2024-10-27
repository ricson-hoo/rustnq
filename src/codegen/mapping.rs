use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::println;
use std::path::{Path};
use sqlx::{AnyConnection, AnyPool, Pool};
use sqlx_mysql::MySql;
use crate::codegen::entity::{FieldNamingConvention, GeneratedStructInfo};
use crate::codegen::utils;
use crate::codegen::utils::{format_name, prepare_directory, TableRow};
use crate::mapping::description::{SqlColumnType, Column, TableFieldConstructInfo, MysqlColumnDefinition, SqlColumn};
use crate::utils::stringUtils;
use std::any::Any;
use std::error::Error;
use crate::mapping::types::{Datetime, Enum, Int, Set, Varchar};
use crate::query::builder::{Condition, QueryBuilder};

pub struct MappingGenerateConfig{
    pub output_dir:String,
    pub crate_and_root_path_of_entity: String, //including 'entity' folder
    pub boolean_columns: HashMap<String, HashSet<String>>,
    pub entity_field_naming_convention: FieldNamingConvention,
    //pub trait_for_enum_types: HashMap<String, String>
}

impl MappingGenerateConfig{
    pub fn new(output_dir:String, crate_and_root_path_of_entity:String, boolean_columns:HashMap<String, HashSet<String>>, entity_field_naming_convention:FieldNamingConvention /*,trait_for_enum_types:HashMap<String, String>*/)->Self{
        MappingGenerateConfig{
            output_dir,
            crate_and_root_path_of_entity,
            boolean_columns,
            entity_field_naming_convention,
            //trait_for_enum_types
        }
    }

    pub fn default()->Self{
        MappingGenerateConfig{
            output_dir:"target/generated/mapping".to_string(),
            crate_and_root_path_of_entity : "shared".to_string(),
            boolean_columns:HashMap::new(),
            entity_field_naming_convention: FieldNamingConvention::SnakeCase
            //trait_for_enum_types:HashMap::new()
        }
    }

    pub fn default_with(crate_and_root_path_of_entity:String, entity_field_naming_convention:FieldNamingConvention)->Self{
        MappingGenerateConfig{
            output_dir:"target/generated/mapping".to_string(),
            crate_and_root_path_of_entity : crate_and_root_path_of_entity,
            boolean_columns:HashMap::new(),
            entity_field_naming_convention: entity_field_naming_convention
            //trait_for_enum_types:HashMap::new()
        }
    }
}

//generate table mappings to db & table definitions
pub async fn generate_mappings(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, db_name:&str, config: MappingGenerateConfig){ ///*mappings_out_dir:&str, name_of_crate_holds_enums: String, boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>*/
    let mappings_out_dir = config.output_dir.clone();
    let crate_and_root_path_of_entity = config.crate_and_root_path_of_entity.clone();
    let boolean_columns = config.boolean_columns.clone();
    let entity_field_naming_convention = config.entity_field_naming_convention.clone();

    //let trait_for_enum_types = config.trait_for_enum_types.clone();
    let mappings_out_path = std::path::Path::new(&mappings_out_dir);
    prepare_directory(mappings_out_path);

    let tables = utils::get_tables(conn).await;
    println!("{:#?}",tables);

    //collect what has been generated
    let mut generated_entities:Vec<GeneratedStructInfo> = Vec::new();

    match tables {
        Ok(tables) => {
            for table in tables {
                let generated_entity_info = generate_mapping(conn, table, mappings_out_path, crate_and_root_path_of_entity.clone(), &boolean_columns, entity_field_naming_convention/*, &trait_for_enum_types*/).await;
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

async fn generate_mapping(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, table: TableRow, output_path:&Path, crate_and_root_path_of_entity: String,
                          boolean_columns: &HashMap<String, HashSet<String>>, entity_field_naming_convention: FieldNamingConvention/*, trait_for_enum_types: &HashMap<String, String>*/) -> GeneratedStructInfo{
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

    let mut items_to_be_imported : Vec<String> = vec!["rustnq::mapping::description::SqlColumnNameAndValue".to_string(), "rustnq::query::builder::Condition".to_string()/*,"serde::Deserialize".to_string(), "serde::Serialize".to_string()*/];
    items_to_be_imported.push("rustnq::mapping::types::Table".to_string());
    let mut struct_fields = vec![];
    let mut instance_fields = vec![];
    let mut instance_with_value_fields = vec![];
    let mut columns_statements = vec![];
    let mut primary_keys_statements = vec![];
    //let mut primary_key = String::new();

    match fields_result {
        Ok(fields) => {
            for field in fields {
                let column_name = if utils::reserved_field_names().contains(&field.name) { format!("{}_", field.name) } else { field.name.clone() };
                let field_definition: String = field.data_type;
                let mysql_cloumn_definition = MysqlColumnDefinition{
                    name:column_name.clone(),
                    name_unmodified : field.name,
                    column_definition:field_definition,
                    default_value:"".to_string(), //all empty for now
                    is_primary_key: field.is_primary_key
                };
                let columnConstructInfo:TableFieldConstructInfo = get_construct_info_from_column_definition(&table.name,mysql_cloumn_definition, crate_and_root_path_of_entity.clone(),entity_field_naming_convention).expect(&format!("Failed to get construct info from table {}",table.name));

                if !columnConstructInfo.import_statements.is_empty() {
                    for import_statement in &columnConstructInfo.import_statements {
                       if !items_to_be_imported.contains(&import_statement){
                           items_to_be_imported.push(import_statement.to_string());
                       }
                    }
                }

                struct_fields.push(format!("pub {}:{},",&column_name,columnConstructInfo.field_type));
                instance_fields.push(format!("{}:{},",&column_name,columnConstructInfo.initial_assignment_with_name));
                instance_with_value_fields.push(format!("{}:{},",&column_name,columnConstructInfo.initial_assignment_with_name_and_value));
                columns_statements.push(format!("SqlColumn::{}(self.{}.clone())",&columnConstructInfo.sql_raw_type,&column_name));//SqlColumnNameAndValue::Varchar("","")
                if field.is_primary_key{
                    primary_keys_statements.push(format!("SqlColumn::{}(self.{}.clone())",&columnConstructInfo.sql_raw_type, &column_name));
                }
            }
        }
        Err(error) => {
            println!("unable to get fields of table {}, error: {:#?}", table.name, error);
        }
    }

    for import in items_to_be_imported{
        writeln!(buf_writer,"use {};",import).expect("Failed to table mapping code");
    }

    writeln!(buf_writer,"\n#[derive(Clone,Debug)]").expect("Failed to table mapping code");
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

    writeln!(buf_writer,"    pub fn new_with_value(entity:{}) ->Self {{", format_name(&table.name,FieldNamingConvention::PascalCase)).expect("Failed to table mapping code");
    writeln!(buf_writer,"        {} {{",struct_name).expect("Failed to table mapping code");
    for field in instance_with_value_fields{
        writeln!(buf_writer,"            {}",field).expect("Failed to table mapping code");
    }
    writeln!(buf_writer,"        }}").expect("Failed to table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to table mapping code");

    writeln!(buf_writer,"}}").expect("Failed to table mapping code");

    writeln!(buf_writer,"impl Table for {} {{", struct_name).expect("Failed to table mapping code");
    writeln!(buf_writer,"    fn name(&self) -> String {{").expect("Failed to table mapping code");
    writeln!(buf_writer,"        \"{}\".to_string()",&table.name).expect("Failed to table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to table mapping code");

    writeln!(buf_writer,"    fn columns(&self) -> Vec<SqlColumnNameAndValue> {{").expect("Failed to table mapping code");
    writeln!(buf_writer,"        vec![").expect("Failed to table mapping code");

    for statement in columns_statements{
        writeln!(buf_writer,"            {},",statement).expect("Failed to table mapping code");
    }
    writeln!(buf_writer,"        ]").expect("Failed to table mapping code");

    writeln!(buf_writer,"    }}").expect("Failed to table mapping code");


    writeln!(buf_writer,"    fn primary_key(&self) -> Vec<SqlColumnNameAndValue> {{").expect("Failed to table mapping code");
    //need primary_keys_as_condition statement list
    let mut i=0;
    for statement in primary_keys_statements{
        if i == 0 {
            write!(buf_writer,"            {}",statement).expect("Failed to table mapping code");
        }else{
            write!(buf_writer,".and({})",statement).expect("Failed to table mapping code");
        }
        i += 1;
    }
    writeln!(buf_writer,"\n    }}").expect("Failed to table mapping code");

    writeln!(buf_writer,"}}").expect("Failed to table mapping code");

    drop(buf_writer);

    GeneratedStructInfo{
        file_name_without_ext : out_file_name_without_ext,
        struct_name,
        enum_file_names_without_ext : vec![]
    }
}


pub fn get_construct_info_from_column_definition(table_name:&str, mysql_col_definition:MysqlColumnDefinition, crate_and_root_path_of_entity: String, entity_field_naming_convention: FieldNamingConvention) -> Result<TableFieldConstructInfo,Box<dyn Error>>{

    let col_definition = mysql_col_definition.column_definition;
    let mut column_type_name = "".to_string();
    if !col_definition.contains("("){
        column_type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&col_definition.replace(" ", "_")));
    }else{
        let parts = col_definition.split("(").map(|s| s.to_string()).collect::<Vec<String>>();
        column_type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&parts[0]));
    }
    let column_type_parse_result = column_type_name.parse::<SqlColumnType>();
    let column_name = mysql_col_definition.name;
    let mut field_type = "".to_string();
    let mut sql_raw_type = "".to_string();
    let mut import_type = "".to_string();
    let mut name_only_default_value = "".to_string();
    let mut name_and_value_from_entity_default_value = "".to_string();
    let mut entity_field_name = format_name(&mysql_col_definition.name_unmodified,entity_field_naming_convention);
    entity_field_name = if utils::reserved_field_names().contains(&mysql_col_definition.name_unmodified) { format!("{}_", &entity_field_name) } else { entity_field_name };
    let entity_struct_name = format_name(table_name,FieldNamingConvention::PascalCase);
    let mut import_statements : Vec<String> = vec![format!("{}::{}",&crate_and_root_path_of_entity, entity_struct_name)];

    match column_type_parse_result{
        Ok(column_type) => {
            match column_type {
                SqlColumnType::Varchar => {
                    field_type = "Varchar".to_string();
                    sql_raw_type = "Varchar".to_string();
                    name_only_default_value = format!("Varchar::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Varchar::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Char => {
                    field_type = "Char".to_string();
                    sql_raw_type = "Char".to_string();
                    name_only_default_value = format!("Char::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Char::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Tinytext => {
                    field_type = "Tinytext".to_string();
                    sql_raw_type = "Tinytext".to_string();
                    name_only_default_value = format!("Tinytext::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Tinytext::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Text => {
                    field_type = "Text".to_string();
                    sql_raw_type = "Text".to_string();
                    name_only_default_value = format!("Text::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Text::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Mediumtext => {
                    field_type = "Mediumtext".to_string();
                    sql_raw_type = "Mediumtext".to_string();
                    name_only_default_value = format!("Mediumtext::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Mediumtext::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Longtext => {
                    field_type = "Longtext".to_string();
                    sql_raw_type = "Longtext".to_string();
                    name_only_default_value = format!("Longtext::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Longtext::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Int => {
                    field_type = "Int".to_string();
                    sql_raw_type = "Int".to_string();
                    name_only_default_value = format!("Int::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Int::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Year => {
                    field_type = "Year".to_string();
                    sql_raw_type = "Year".to_string();
                    name_only_default_value = format!("Year::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Year::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Enum => {
                    let short_enum_name = format!("{}{}",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&column_name)));
                    let mut enumType = format!("{}",&short_enum_name);
                    if !crate_and_root_path_of_entity.is_empty(){
                        enumType = format!("{}::enums::{}",crate_and_root_path_of_entity, enumType);
                    }
                    field_type = format!("Enum<{}>", short_enum_name);
                    sql_raw_type = "Enum".to_string();
                    import_type = "Enum".to_string();
                    name_only_default_value = format!("Enum::<{}>::name(\"{}\".to_string())", short_enum_name, mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Enum::<{}>::name_value(\"{}\".to_string(), entity.{})", short_enum_name, mysql_col_definition.name_unmodified, &entity_field_name);
                    //import enum
                    import_statements.push(enumType);
                },
                SqlColumnType::Set => {
                    let short_enum_name = format!("{}{}",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&column_name)));
                    let mut enumType = format!("{}",&short_enum_name);
                    if !crate_and_root_path_of_entity.is_empty(){
                        enumType = format!("{}::enums::{}",crate_and_root_path_of_entity, enumType);
                    }
                    field_type = format!("Set<{}>", short_enum_name);
                    sql_raw_type = "Set".to_string();
                    import_type = "Set".to_string();
                    name_only_default_value = format!("Set::<{}>::name(\"{}\".to_string())", short_enum_name, mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Set::<{}>::name_value(\"{}\".to_string(), entity.{})", short_enum_name, mysql_col_definition.name_unmodified, &entity_field_name);
                    //import enum
                    import_statements.push(enumType);
                },
                SqlColumnType::Datetime => {
                    field_type = "Datetime".to_string();
                    sql_raw_type = "Datetime".to_string();
                    name_only_default_value = format!("Datetime::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Datetime::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Timestamp => {
                    field_type = "Timestamp".to_string();
                    sql_raw_type = "Timestamp".to_string();
                    name_only_default_value = format!("Timestamp::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Timestamp::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Tinyint => {
                    field_type = "Tinyint".to_string();
                    sql_raw_type = "Tinyint".to_string();
                    name_only_default_value = format!("Tinyint::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Tinyint::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Smallint => {
                    field_type = "Smallint".to_string();
                    sql_raw_type = "Smallint".to_string();
                    name_only_default_value = format!("Smallint::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Smallint::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Bigint => {
                    field_type = "Bigint".to_string();
                    sql_raw_type = "Bigint".to_string();
                    name_only_default_value = format!("Bigint::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Bigint::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::BigintUnsigned => {
                    field_type = "BigintUnsigned".to_string();
                    sql_raw_type = "BigintUnsigned".to_string();
                    name_only_default_value = format!("BigintUnsigned::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("BigintUnsigned::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Numeric => {
                    field_type = "Numeric".to_string();
                    sql_raw_type = "Numeric".to_string();
                    name_only_default_value = format!("Numeric::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Numeric::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Float => {
                    field_type = "Float".to_string();
                    sql_raw_type = "Float".to_string();
                    name_only_default_value = format!("Float::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Float::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Double => {
                    field_type = "Double".to_string();
                    sql_raw_type = "Double".to_string();
                    name_only_default_value = format!("Double::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Double::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Decimal => {
                    field_type = "Varchar".to_string();
                    sql_raw_type = "Varchar".to_string();
                    name_only_default_value = format!("Varchar::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Varchar::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Date => {
                    field_type = "Date".to_string();
                    sql_raw_type = "Date".to_string();
                    name_only_default_value = format!("Date::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Date::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Time => {
                    field_type = "Time".to_string();
                    sql_raw_type = "Time".to_string();
                    name_only_default_value = format!("Time::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Time::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Blob => {
                    field_type = "Blob".to_string();
                    sql_raw_type = "Blob".to_string();
                    name_only_default_value = format!("Blob::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Blob::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
                SqlColumnType::Json => {
                    field_type = "Json".to_string();
                    sql_raw_type = "Json".to_string();
                    name_only_default_value = format!("Json::name(\"{}\".to_string())", mysql_col_definition.name_unmodified);
                    name_and_value_from_entity_default_value = format!("Json::name_value(\"{}\".to_string(), entity.{})", mysql_col_definition.name_unmodified, &entity_field_name);
                },
            }
        },
        Err(_) => {

        }
    };
    //add current type to import statements
    import_statements.push(format!("rustnq::mapping::types::{}",if import_type.is_empty() {&field_type } else {&import_type}));

    Ok(TableFieldConstructInfo{
        field_name : column_name,
        field_type: field_type,
        initial_assignment_with_name: name_only_default_value,
        initial_assignment_with_name_and_value: name_and_value_from_entity_default_value,
        import_statements: import_statements,
        sql_raw_type:sql_raw_type, //如Char,Varchar,Tinytext,Datetime,Timestamp...
    })

}