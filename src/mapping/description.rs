use std::str::FromStr;
use std::error::Error;
use anyhow::bail;
use crate::mapping::types::{Int, Varchar,Enum,Set,DateTime};
use crate::utils::stringUtils;

#[derive(Clone, Copy)]
pub enum Holding{
    Name,Value,Full
}

pub trait Column {
    fn name(&self) -> &str;
    //fn value(value: String) -> Self;
    fn new(name: String) -> Self;
}

pub trait MappedEnum {
    fn name(&self) -> &str;
}

// Implement MappedEnum for any type T that implements MappedEnum
/*impl<'a, T: MappedEnum> MappedEnum for &'a T {
    fn name(&self) -> &str {
        (*self).name()
    }
}*/


/*pub trait Column<'a> {
    fn name(&self) -> &'a str;
    fn value(&self) -> &'a str;
}*/

//pub struct ColumnType<'a>(Box<dyn Column<'a>>);

pub enum ColumnType {
    Varchar,
    Int,
    Enum,
    Set,
    Datetime,
}

pub struct MysqlColumnDefinition{
    pub name: String,
    pub column_definition: String,
    pub default_value: String,
}

pub struct TableFieldConstructInfo {
    pub field_name:String,
    pub file_type:String,
    pub default_value_on_new:String,
    pub import_statement:String
}

/*
impl <'a> ColumnType<'a> {
    pub(crate) fn constructInfo(&self)-> ColumnConstructInfo {
        let mut type_name_str = "".to_string();
        let mut import_statement_str = "".to_string();
        let mut column_name_str = "".to_string();
        match self {
            ColumnType::Varchar(instance, type_name, import_statement) => {
                type_name_str = type_name.to_string();
                import_statement_str = import_statement.to_string();
                column_name_str = instance.name().to_string();
            }
            ColumnType::Int(instance, type_name, import_statement) => {
                type_name_str = type_name.to_string();
                import_statement_str = import_statement.to_string();
                column_name_str = instance.name().to_string();
            }
            ColumnType::Enum(instance, type_name, import_statement) => {
                type_name_str = format!("{}<T>",type_name);
                import_statement_str = import_statement.to_string();
                column_name_str = instance.name().to_string();
            }
            ColumnType::Set(instance, type_name, import_statement) => {
                type_name_str = format!("{}<T>",type_name);
                import_statement_str = import_statement.to_string();
                column_name_str = instance.name().to_string();
            }
            ColumnType::Datetime(instance, type_name, import_statement) => {
                type_name_str = type_name.to_string();
                import_statement_str = import_statement.to_string();
                column_name_str = instance.name().to_string();
            }
            _ => {
                type_name_str = "unsupported".to_string();
                import_statement_str = "".to_string();
                column_name_str = "unsupported".to_string();
            }
        }
        ColumnConstructInfo{
            type_name:type_name_str.clone(),
            default_value_str:format!("{}({}::name(\"{}\"))",type_name_str.clone(),type_name_str,column_name_str),
            import_statement:import_statement_str
        }
    }
}*/

impl FromStr for ColumnType {
    type Err = ();

    fn from_str(mysql_col_type: &str) -> Result<Self, Self::Err> {
        //let mysql_col_type_and_name_final = mysql_col_type_and_name.replace(" ", "_");
        //let parts = mysql_col_type_and_name_final.split(",").map(|s| s.to_string()).collect::<Vec<String>>();
        //let type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&parts[0]));

        match mysql_col_type {
            //"CHAR" => ColumnType(Box::new(Char::name(""))),
            "Varchar" => Ok(ColumnType::Varchar/*(Varchar::new(parts[1].clone()),type_name.clone(),format!("types::{}",type_name))*/),
            /*"TINYTEXT" => Ok(crate::codegen::entity::MysqlDataType::Tinytext),
            "TEXT" => Ok(crate::codegen::entity::MysqlDataType::Text),
            "MEDIUMTEXT" => Ok(crate::codegen::entity::MysqlDataType::Mediumtext),
            "LONGTEXT" => Ok(crate::codegen::entity::MysqlDataType::Longtext),*/
            "Enum" => Ok(ColumnType::Enum/*(Enum::new(parts[1].clone()),type_name.clone(),format!("types::{}",type_name))*/),
            "Set" => Ok(ColumnType::Set/*(Set::new(parts[1].clone()),type_name.clone(),format!("types::{}",type_name))*/),
            /*"TINYINT" => Ok(crate::codegen::entity::MysqlDataType::Tinyint),
            "SMALLINT" => Ok(crate::codegen::entity::MysqlDataType::Smallint),*/
            "Int" => Ok(ColumnType::Int/*(Int::new(parts[1].clone()),type_name.clone(),format!("types::{}",type_name))*/),
            /*"BIGINT" => Ok(crate::codegen::entity::MysqlDataType::Bigint),
            "BIGINT_UNSIGNED" => Ok(crate::codegen::entity::MysqlDataType::BigintUnsigned),
            "NUMERIC" => Ok(crate::codegen::entity::MysqlDataType::Numeric),
            "FLOAT" => Ok(crate::codegen::entity::MysqlDataType::Float),
            "DOUBLE" => Ok(crate::codegen::entity::MysqlDataType::Double),
            "DECIMAL" => Ok(crate::codegen::entity::MysqlDataType::Decimal),
            "DATE" => Ok(crate::codegen::entity::MysqlDataType::Date),
            "TIME" => Ok(crate::codegen::entity::MysqlDataType::Time),*/
            "Datetime" => Ok(ColumnType::Datetime/*(DateTime::new(parts[1].clone()),type_name.clone(),format!("types::{}",type_name))*/),
            "Timestamp" => Ok(ColumnType::Datetime/*(DateTime::new(parts[1].clone()),type_name.clone(),format!("types::{}",type_name))*/),
            /*"YEAR" => Ok(crate::codegen::entity::MysqlDataType::Year),
            "BLOB" => Ok(crate::codegen::entity::MysqlDataType::Blob),
            "JSON" => Ok(crate::codegen::entity::MysqlDataType::Json),*/
            _ => {
                Err(())
            }
        }
    }
}

/*impl <'a> From<&'a str> for ColumnType {
    fn from(mysql_col_type_and_name: &'a str) -> ColumnType {

    }
}*/

pub fn get_construct_info_from_column_definition(table_name:&str,mysql_col_definitin:MysqlColumnDefinition, name_of_crate_holds_enums: String)-> Result<TableFieldConstructInfo,Box<dyn Error>>{

    /*pub type_name:String,
    pub default_value_str:String,
    pub import_statement:String*/

    let col_definition = mysql_col_definitin.column_definition;
    let mut column_type_name = "".to_string();
    if !col_definition.contains("("){
        column_type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&col_definition.replace(" ", "_")));
    }else{
        let parts = col_definition.split("(").map(|s| s.to_string()).collect::<Vec<String>>();
        column_type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&parts[0]));
    }
    let column_type_parse_result = column_type_name.parse::<ColumnType>();
    let column_name = mysql_col_definitin.name;
    let mut file_type = "";
    let mut default_value = "".to_string();
    let import_statement = "";

    match column_type_parse_result{
        Ok(column_type) => {
            match column_type {
                ColumnType::Varchar => {
                    file_type = "Varchar";
                    default_value = format!("Varchar::name(\"{}\")",column_name);
                },
                ColumnType::Int => {
                    file_type = "Int";
                    default_value = format!("Int::name(\"{}\")",column_name);
                },
                ColumnType::Enum => {
                    let mut enumType = format!("entity::enums::{}{}",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&column_name)));
                    if !name_of_crate_holds_enums.is_empty(){
                        enumType = format!("{}::{}",name_of_crate_holds_enums, enumType);
                    }
                    file_type = "Enum";
                    default_value = format!("Enum<{}>::name(\"{}\")",enumType, column_name);
                },
                ColumnType::Set => {
                    let mut enumType = format!("entity::enums::{}{}",stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(table_name)),stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&column_name)));
                    if !name_of_crate_holds_enums.is_empty(){
                        enumType = format!("{}::{}",name_of_crate_holds_enums, enumType);
                    }
                    file_type = "Set";
                    default_value = format!("Set<{}>::name(\"{}\")",enumType, column_name);
                },
                ColumnType::Datetime => {
                    file_type = "Datetime";
                    default_value = format!("Datetime::name(\"{}\")",column_name);
                },
            }
        },
        Err(_) => {

        }
    };

    Ok(TableFieldConstructInfo{
        field_name : column_name,
        file_type:file_type.to_string(),
        default_value_on_new:default_value,
        import_statement: format!("rustnq::types::{}",file_type)
    })

}