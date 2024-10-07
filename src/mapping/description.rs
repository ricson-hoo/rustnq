use std::str::FromStr;
use anyhow::bail;
use crate::mapping::types::{Int, Varchar};
use crate::utils::stringUtils;

#[derive(Clone, Copy)]
pub enum Holding{
    Name,Value,Full
}

pub trait Selectable {
    fn name(&self) -> &str;
}

/*pub trait Column<'a> {
    fn name(&self) -> &'a str;
    fn value(&self) -> &'a str;
}*/

//pub struct ColumnType<'a>(Box<dyn Column<'a>>);

pub enum ColumnType {
    Varchar(Varchar, String, String),//instance, type_name, items to be imported
    Int(Int, String, String),
}

pub struct ColumnConstructInfo{
    pub type_name:String,
    pub default_value_str:String,
    pub import_statement:String
}

impl ColumnType {
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
        }
        ColumnConstructInfo{
            type_name:type_name_str.clone(),
            default_value_str:format!("ColumnType::{}({}::name({}))",type_name_str.clone(),type_name_str,column_name_str),
            import_statement:import_statement_str
        }
    }
}

impl FromStr for ColumnType {
    type Err = ();

    fn from_str(mysql_col_type_and_name: &str) -> Result<Self, Self::Err> {
        let mysql_col_type_and_name_final = mysql_col_type_and_name.replace(" ", "_");
        let parts = mysql_col_type_and_name_final.split(",").map(|s| s.to_string()).collect::<Vec<String>>();
        let type_name = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&parts[0]));

        match type_name.as_str() {
            //"CHAR" => ColumnType(Box::new(Char::name(""))),
            "Varchar" => Ok(ColumnType::Varchar(Varchar::name(parts[1].clone()),type_name.clone(),format!("types::{}",type_name))),
            /*"TINYTEXT" => Ok(crate::codegen::entity::MysqlDataType::Tinytext),
            "TEXT" => Ok(crate::codegen::entity::MysqlDataType::Text),
            "MEDIUMTEXT" => Ok(crate::codegen::entity::MysqlDataType::Mediumtext),
            "LONGTEXT" => Ok(crate::codegen::entity::MysqlDataType::Longtext),
            "ENUM" => Ok(crate::codegen::entity::MysqlDataType::Enum),
            "SET" => Ok(crate::codegen::entity::MysqlDataType::Set),
            "TINYINT" => Ok(crate::codegen::entity::MysqlDataType::Tinyint),
            "SMALLINT" => Ok(crate::codegen::entity::MysqlDataType::Smallint),
            "INT" => Ok(crate::codegen::entity::MysqlDataType::Int),
            "BIGINT" => Ok(crate::codegen::entity::MysqlDataType::Bigint),
            "BIGINT_UNSIGNED" => Ok(crate::codegen::entity::MysqlDataType::BigintUnsigned),
            "NUMERIC" => Ok(crate::codegen::entity::MysqlDataType::Numeric),
            "FLOAT" => Ok(crate::codegen::entity::MysqlDataType::Float),
            "DOUBLE" => Ok(crate::codegen::entity::MysqlDataType::Double),
            "DECIMAL" => Ok(crate::codegen::entity::MysqlDataType::Decimal),
            "DATE" => Ok(crate::codegen::entity::MysqlDataType::Date),
            "TIME" => Ok(crate::codegen::entity::MysqlDataType::Time),
            "DATETIME" => Ok(crate::codegen::entity::MysqlDataType::DateTime),
            "TIMESTAMP" => Ok(crate::codegen::entity::MysqlDataType::Timestamp),
            "YEAR" => Ok(crate::codegen::entity::MysqlDataType::Year),
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