use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::println;
use std::path::{Path};
use sqlx::{AnyConnection, AnyPool, Pool};
use sqlx_mysql::MySql;
use crate::codegen::entity::{NamingConvention, GeneratedStructInfo};
use crate::codegen::utils;
use crate::codegen::utils::{format_name, prepare_directory, TableRow};
use crate::mapping::description::{Column, TableFieldConstructInfo, MysqlColumnDefinition, SqlColumn};
use crate::utils::stringUtils;
use std::any::Any;
use std::error::Error;
use crate::mapping::column_types::{Datetime, Enum, Int, Set, Varchar};
use crate::query::builder::{Condition, QueryBuilder};

pub struct MappingGenerateConfig{
    pub output_dir:String,
    pub crate_and_root_path_of_entity: String, //including 'entity' folder
    pub boolean_columns: HashMap<String, Vec<String>>,
    pub entity_field_naming_convention: NamingConvention,
    pub encrypted_columns: HashMap<String, Vec<&'static str>>,
    //pub trait_for_enum_types: HashMap<String, String>
}

impl MappingGenerateConfig{
    pub fn new(output_dir:String, crate_and_root_path_of_entity:String, boolean_columns:HashMap<String, Vec<String>>, entity_field_naming_convention: NamingConvention /*,trait_for_enum_types:HashMap<String, String>*/,encrypted_columns: HashMap<String, Vec<&'static str>>) ->Self{
        MappingGenerateConfig{
            output_dir,
            crate_and_root_path_of_entity,
            boolean_columns,
            entity_field_naming_convention,
            //trait_for_enum_types
            encrypted_columns
        }
    }

    pub fn default()->Self{
        MappingGenerateConfig{
            output_dir:"target/generated/mapping".to_string(),
            crate_and_root_path_of_entity : "shared".to_string(),
            boolean_columns:HashMap::new(),
            entity_field_naming_convention: NamingConvention::SnakeCase,
            //trait_for_enum_types:HashMap::new(),
            encrypted_columns:HashMap::new()
        }
    }

    pub fn default_with(crate_and_root_path_of_entity:String, entity_field_naming_convention: NamingConvention,encrypted_columns: HashMap<String, Vec<&'static str>>) ->Self{
        MappingGenerateConfig{
            output_dir:"target/generated/mapping".to_string(),
            crate_and_root_path_of_entity : crate_and_root_path_of_entity,
            boolean_columns:HashMap::new(),
            entity_field_naming_convention: entity_field_naming_convention,
            //trait_for_enum_types:HashMap::new()
            encrypted_columns
        }
    }
}

//generate table mappings to db & table definitions
pub async fn generate_mappings(conn: & sqlx::pool::Pool<sqlx_mysql::MySql>, db_name:&str, config: MappingGenerateConfig){ ///*mappings_out_dir:&str, name_of_crate_holds_enums: String, boolean_columns: &HashMap<String, HashSet<String>>, trait_for_enum_types: &HashMap<&str, &str>*/
    let mappings_out_dir = config.output_dir.clone();
    let crate_and_root_path_of_entity = config.crate_and_root_path_of_entity.clone();
    let boolean_columns = config.boolean_columns.clone();
    let entity_field_naming_convention = config.entity_field_naming_convention.clone();
    let encrypted_columns = config.encrypted_columns.clone();

    //let trait_for_enum_types = config.trait_for_enum_types.clone();
    let mappings_out_path = std::path::Path::new(&mappings_out_dir);
    prepare_directory(mappings_out_path);

    let tables = utils::get_tables(conn).await;
    //println!("{:#?}",tables);

    //collect what has been generated
    let mut generated_entities:Vec<GeneratedStructInfo> = Vec::new();

    match tables {
        Ok(tables) => {
            for table in tables {
                let encrypted_cols = encrypted_columns.get(&table.name.clone());
                let generated_entity_info = generate_mapping(conn, table, mappings_out_path, crate_and_root_path_of_entity.clone(), &boolean_columns, entity_field_naming_convention/*, &trait_for_enum_types*/,encrypted_cols.map_or(vec![],|cols|cols.clone())).await;
                generated_entities.push(generated_entity_info);
            }
            println!("mappings generated successfully");
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
                          boolean_columns: &HashMap<String, Vec<String>>, entity_field_naming_convention: NamingConvention/*, trait_for_enum_types: &HashMap<String, String>*/,
                          encrypted_columns: Vec<&'static str>) -> GeneratedStructInfo{
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

    let mut items_to_be_imported : Vec<String> = vec!["rustnq::mapping::description::{Table, Column, SqlColumn}".to_string(), "rustnq::query::builder::Condition".to_string()/*,"serde::Deserialize".to_string(), "serde::Serialize".to_string()*/];
    let mut struct_fields = vec![];
    let mut struct_field_names = vec![];
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
                let columnConstructInfo:TableFieldConstructInfo = get_construct_info_from_column_definition(&table.name,mysql_cloumn_definition, crate_and_root_path_of_entity.clone(),entity_field_naming_convention,boolean_columns,encrypted_columns.clone()).expect(&format!("Failed to get construct info from table {}",table.name));

                if !columnConstructInfo.import_statements.is_empty() {
                    for import_statement in &columnConstructInfo.import_statements {
                       if !items_to_be_imported.contains(&import_statement){
                           items_to_be_imported.push(import_statement.to_string());
                       }
                    }
                }
                struct_field_names.push(column_name.clone());
                struct_fields.push(format!("pub {}:{},",&column_name,columnConstructInfo.field_type));
                instance_fields.push(format!("{}:{},",&column_name,columnConstructInfo.initial_assignment_with_name));
                instance_with_value_fields.push(format!("{}:{},",&column_name,columnConstructInfo.initial_assignment_with_name_and_value));

                columns_statements.push(format!("{}(Some(self.{}.clone(){}))",&columnConstructInfo.sql_column_type.clone().unwrap().to_string(),&column_name, if columnConstructInfo.sql_column_type_modified {".into()"} else {""}));

                if field.is_primary_key{
                    primary_keys_statements.push(format!("{}(Some(self.{}.clone()))",&columnConstructInfo.sql_column_type.unwrap().to_string(), &column_name));
                }
            }
        }
        Err(error) => {
            println!("unable to get fields of table {}, error: {:#?}", table.name, error);
        }
    }

    for import in items_to_be_imported{
        writeln!(buf_writer,"use {};",import).expect("Failed to write table mapping code");
    }

    writeln!(buf_writer,"\n#[derive(Clone,Debug)]").expect("Failed to write table mapping code");
    writeln!(buf_writer,"pub struct {} {{", struct_name).expect("Failed to write table mapping code");

    for field in struct_fields{
        writeln!(buf_writer,"    {}",field).expect("Failed to write table mapping code");
    }
    writeln!(buf_writer,"    _primary_key:Vec<SqlColumn>,").expect("Failed to write table mapping code");

    writeln!(buf_writer,"    _alias:Option<String>").expect("Failed to write table mapping code");

    writeln!(buf_writer,"}}").expect("Failed to write table mapping code");

    writeln!(buf_writer,"impl {} {{", struct_name).expect("Failed to write table mapping code");

    writeln!(buf_writer,"    pub fn new(alias:Option<&str>) ->Self {{").expect("Failed to write table mapping code");
    //writeln!(buf_writer,"        self._alias = alias;").expect("Failed to write table mapping code");
    writeln!(buf_writer,"        let table_name = alias.clone().unwrap_or(\"{}\");",&table.name).expect("Failed to write table mapping code");
    writeln!(buf_writer,"        {} {{",struct_name).expect("Failed to write table mapping code");
    for field in instance_fields {
        writeln!(buf_writer,"            {}",field).expect("Failed to write table mapping code");
    }
    writeln!(buf_writer,"            _primary_key:vec![],").expect("Failed to write table mapping code");
    writeln!(buf_writer,"            _alias:alias.map(|s| s.to_string()),").expect("Failed to write table mapping code");
    writeln!(buf_writer,"        }}").expect("Failed to write table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to write table mapping code");

/*    writeln!(buf_writer,"    pub fn all_fields(&self) -> Vec<String> {{").expect("Failed to write table mapping code");
    write!(buf_writer,"        vec![").expect("Failed to write table mapping code");
    for field_name in struct_field_names {
        write!(buf_writer,"self.{}.name(), ",field_name).expect("Failed to write table mapping code");
    }
    write!(buf_writer,"]\n").expect("Failed to write table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to write table mapping code");*/

    writeln!(buf_writer,"    pub fn new_with_value(entity:{}, alias:Option<&str>) ->Self {{", format_name(&table.name, NamingConvention::PascalCase)).expect("Failed to write table mapping code");
    //writeln!(buf_writer,"        self._alias = alias;").expect("Failed to write table mapping code");
    writeln!(buf_writer,"        let table_name = alias.clone().unwrap_or(\"{}\");",&table.name).expect("Failed to write table mapping code");
    writeln!(buf_writer,"        {} {{",struct_name).expect("Failed to write table mapping code");
    for field in instance_with_value_fields{
        writeln!(buf_writer,"            {}",field).expect("Failed to write table mapping code");
    }
    writeln!(buf_writer,"            _primary_key:vec![],").expect("Failed to write table mapping code");
    writeln!(buf_writer,"            _alias:alias.map(|s| s.to_string()),").expect("Failed to write table mapping code");
    writeln!(buf_writer,"        }}").expect("Failed to write table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to write table mapping code");

    writeln!(buf_writer,"}}").expect("Failed to write table mapping code");

    writeln!(buf_writer,"impl Table for {} {{", struct_name).expect("Failed to write table mapping code");
    writeln!(buf_writer,"    fn name(&self) -> String {{").expect("Failed to table write mapping code");
    writeln!(buf_writer,"        if self._alias.is_some() {{ format!(\"{} as {{}}\",self._alias.clone().unwrap()) }} else {{\"{}\".to_string()}}",&table.name, &table.name).expect("Failed to write table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to write table mapping code");

    writeln!(buf_writer,"    fn all_columns(&self) -> Vec<SqlColumn> {{").expect("Failed to write table mapping code");
    writeln!(buf_writer,"        vec![").expect("Failed to write table mapping code");

    for statement in columns_statements{
        writeln!(buf_writer,"            {},",statement).expect("Failed to write table mapping code");
    }
    writeln!(buf_writer,"        ]").expect("Failed to write table mapping code");

    writeln!(buf_writer,"    }}").expect("Failed to write table mapping code");


    writeln!(buf_writer,"    fn primary_key(&self) -> Vec<SqlColumn> {{").expect("Failed to write table mapping code");
    //need primary_keys_as_condition statement list
    writeln!(buf_writer,"        if self._primary_key.is_empty() {{ vec![").expect("Failed to write table mapping code");
    for statement in primary_keys_statements{
        writeln!(buf_writer,"            {},",statement).expect("Failed to write table mapping code");
    }
    writeln!(buf_writer,"        ]}} else {{self._primary_key.clone()}}").expect("Failed to write table mapping code");

    writeln!(buf_writer,"    }}").expect("Failed to write table mapping code");

    writeln!(buf_writer,"    fn update_primary_key(&mut self, primary_key: Vec<SqlColumn>) -> () {{").expect("Failed to write table mapping code");
    writeln!(buf_writer,"            self._primary_key = primary_key;").expect("Failed to write table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to write table mapping code");

/*  writeln!(buf_writer,"    fn as_(&mut self,alias:&str) -> Self {{").expect("Failed to write table mapping code");
    writeln!(buf_writer,"            self._alias = Some(alias.to_string());").expect("Failed to write table mapping code");
    writeln!(buf_writer,"            self.clone()").expect("Failed to write table mapping code");
    writeln!(buf_writer,"    }}").expect("Failed to write table mapping code");*/

    writeln!(buf_writer,"}}").expect("Failed to write table mapping code");

    drop(buf_writer);

    GeneratedStructInfo{
        file_name_without_ext : out_file_name_without_ext,
        struct_name,
        enum_file_names_without_ext : vec![]
    }
}


pub fn get_construct_info_from_column_definition(table_name:&str, mysql_col_definition:MysqlColumnDefinition, crate_and_root_path_of_entity: String, entity_field_naming_convention: NamingConvention,boolean_columns: &HashMap<String, Vec<String>>,encrypted_columns: Vec<&'static str>) -> Result<TableFieldConstructInfo,Box<dyn Error>>{

    let col_definition = mysql_col_definition.column_definition;
    let mut col_len = "".to_string();
    let mut column_type_name = "".to_string();
    if !col_definition.contains("("){
        column_type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&col_definition.replace(" ", "_")));
    }else{
        let parts = col_definition.split("(").map(|s| s.to_string()).collect::<Vec<String>>();
        column_type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&parts[0]));
        col_len = if parts.len() > 1 {
            parts[1].replace(")", "")
        } else {
            "".to_string()
        };
    }
    let column_type_parse_result = column_type_name.parse::<SqlColumn>();
    let column_name = mysql_col_definition.name;
    let mut field_type = "".to_string();
    let mut sql_column_type:Option<SqlColumn> = None;
    let mut sql_column_type_modified = false;
    let mut import_type = "".to_string();
    let mut name_only_default_value = "".to_string();
    let mut name_and_value_from_entity_default_value = "".to_string();
    let mut entity_field_name = format_name(&mysql_col_definition.name_unmodified,entity_field_naming_convention);
    entity_field_name = if utils::reserved_field_names().contains(&mysql_col_definition.name_unmodified) { format!("{}_", &entity_field_name) } else { entity_field_name };
    let entity_struct_name = format_name(table_name, NamingConvention::PascalCase);
    let mut import_statements : Vec<String> = vec![format!("{}::{}",&crate_and_root_path_of_entity, entity_struct_name)];
    let is_encrypted = encrypted_columns.contains(&&*column_name);

    match column_type_parse_result{
        Ok(column_type) => {
            sql_column_type = Some(column_type.clone());
            match column_type {
                SqlColumn::Varchar(_) => {
                    field_type = "Varchar".to_string();
                    name_only_default_value = format!("Varchar::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Varchar::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Char(_) => {
                    field_type = "Char".to_string();
                    name_only_default_value = format!("Char::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Char::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});

                },
                SqlColumn::Tinytext(_) => {
                    field_type = "Tinytext".to_string();
                    name_only_default_value = format!("Tinytext::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Tinytext::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Text(_) => {
                    field_type = "Text".to_string();
                    name_only_default_value = format!("Text::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Text::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Mediumtext(_) => {
                    field_type = "Mediumtext".to_string();
                    name_only_default_value = format!("Mediumtext::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Mediumtext::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Longtext(_) => {
                    field_type = "Longtext".to_string();
                    name_only_default_value = format!("Longtext::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Longtext::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Int(_) => {
                    field_type = "Int".to_string();
                    name_only_default_value = format!("Int::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Int::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Year(_) => {
                    field_type = "Year".to_string();
                    name_only_default_value = format!("Year::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Year::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Enum(_) => {
                    let short_enum_name = format!("{}{}",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&column_name)));
                    let mut enumType = format!("{}",&short_enum_name);
                    if !crate_and_root_path_of_entity.is_empty(){
                        enumType = format!("{}::enums::{}",crate_and_root_path_of_entity, enumType);
                    }
                    field_type = format!("Enum<{}>", short_enum_name);
                    sql_column_type = Some(SqlColumn::Varchar(None));
                    sql_column_type_modified = true;
                    import_type = "Enum".to_string();
                    //name_only_default_value = format!("Enum::<{}>::with_qualified_name(\"{}.{}\".to_string())", short_enum_name, table_name, mysql_col_definition.name_unmodified);
                    name_only_default_value = format!("Enum::<{}>::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}",short_enum_name, mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Enum::<{}>::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", short_enum_name, mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});

                    //import enum
                    import_statements.push(enumType);
                },
                SqlColumn::Set(_) => {
                    let short_enum_name = format!("{}{}",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&column_name)));
                    let mut enumType = format!("{}",&short_enum_name);
                    if !crate_and_root_path_of_entity.is_empty(){
                        enumType = format!("{}::enums::{}",crate_and_root_path_of_entity, enumType);
                    }
                    field_type = format!("Set<{}>", short_enum_name);
                    sql_column_type = Some(SqlColumn::Varchar(None));
                    sql_column_type_modified = true;
                    import_type = "Set".to_string();
                    name_only_default_value = format!("Set::<{}>::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", short_enum_name, mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Set::<{}>::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", short_enum_name, mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                    //import enum
                    import_statements.push(enumType);
                },
                SqlColumn::Datetime(_) => {
                    field_type = "Datetime".to_string();
                    name_only_default_value = format!("Datetime::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Datetime::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Timestamp(_) => {
                    field_type = "Timestamp".to_string();
                    name_only_default_value = format!("Timestamp::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Timestamp::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Tinyint(_) => {
                    if col_len == "1".to_string() || (boolean_columns.contains_key(table_name) && boolean_columns[table_name].contains(&column_name.to_string())) {
                        sql_column_type = Some(SqlColumn::Boolean(None));
                        field_type = "Boolean".to_string();
                        name_only_default_value = format!("Boolean::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                        name_and_value_from_entity_default_value = format!("Boolean::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                        sql_column_type_modified = true;
                    }else{
                        field_type = "Tinyint".to_string();
                        name_only_default_value = format!("Tinyint::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                        name_and_value_from_entity_default_value = format!("Tinyint::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                    }
                },
                SqlColumn::Boolean(_) => {
                    field_type = "Boolean".to_string();
                    name_only_default_value = format!("Boolean::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Boolean::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                    sql_column_type_modified = true;
                },
                SqlColumn::Smallint(_) => {
                    field_type = "Smallint".to_string();
                    name_only_default_value = format!("Smallint::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Smallint::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Bigint(_) => {
                    field_type = "Bigint".to_string();
                    name_only_default_value = format!("Bigint::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Bigint::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::BigintUnsigned(_) => {
                    field_type = "BigintUnsigned".to_string();
                    name_only_default_value = format!("BigintUnsigned::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("BigintUnsigned::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Numeric(_) => {
                    field_type = "Numeric".to_string();
                    name_only_default_value = format!("Numeric::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Numeric::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});

                },
                SqlColumn::Float(_) => {
                    field_type = "Float".to_string();
                    name_only_default_value = format!("Float::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Float::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Double(_) => {
                    field_type = "Double".to_string();
                    name_only_default_value = format!("Double::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Double::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Decimal(_) => {
                    field_type = "Decimal".to_string();
                    name_only_default_value = format!("Decimal::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Decimal::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Date(_) => {
                    field_type = "Date".to_string();
                    name_only_default_value = format!("Date::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Date::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Time(_) => {
                    field_type = "Time".to_string();
                    name_only_default_value = format!("Time::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Time::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Blob(_) => {
                    field_type = "Blob".to_string();
                    name_only_default_value = format!("Blob::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Blob::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
                SqlColumn::Json(_) => {
                    field_type = "Json".to_string();
                    name_only_default_value = format!("Json::with_qualified_name(table_name.to_string(),\"{}\".to_string()){}", mysql_col_definition.name_unmodified, if is_encrypted { ".set_encrypted(true)" } else {""});
                    name_and_value_from_entity_default_value = format!("Json::with_qualified_name_value(table_name.to_string(),\"{}\".to_string(), entity.{}){}", mysql_col_definition.name_unmodified, &entity_field_name, if is_encrypted { ".set_encrypted(true)" } else {""});
                },
            }
        },
        Err(_) => {

        }
    };
    //add current type to import statements
    import_statements.push(format!("rustnq::mapping::column_types::{}",if import_type.is_empty() {&field_type } else {&import_type}));

    Ok(TableFieldConstructInfo{
        field_name : column_name,
        field_type: field_type,
        initial_assignment_with_name: name_only_default_value,
        initial_assignment_with_name_and_value: name_and_value_from_entity_default_value,
        import_statements: import_statements,
        sql_column_type:sql_column_type, //如Char,Varchar,Tinytext,Datetime,Timestamp...
        sql_column_type_modified: sql_column_type_modified
    })

}