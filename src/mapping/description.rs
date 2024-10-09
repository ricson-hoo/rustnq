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
    //fn new(name: String) -> Self;
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

pub enum MysqlColumnType {
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
    Datetime,
    Timestamp,
    Year,
    Blob,
    Json
}

impl FromStr for MysqlColumnType {
    type Err = anyhow::Error;
    fn from_str(mysql_col_type: &str) -> Result<Self, anyhow::Error> {
        let typeName = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&mysql_col_type.replace(" ", "_")));
        match typeName.as_str() {
            "Char" => Ok(MysqlColumnType::Char),
            "Varchar" => Ok(MysqlColumnType::Varchar),
            "Tinytext" => Ok(MysqlColumnType::Tinytext),
            "Text" => Ok(MysqlColumnType::Text),
            "Mediumtext" => Ok(MysqlColumnType::Mediumtext),
            "Longtext" => Ok(MysqlColumnType::Longtext),
            "Enum" => Ok(MysqlColumnType::Enum),
            "Set" => Ok(MysqlColumnType::Set),
            "Tinyint" => Ok(MysqlColumnType::Tinyint),
            "Smallint" => Ok(MysqlColumnType::Smallint),
            "Int" => Ok(MysqlColumnType::Int),
            "Bigint" => Ok(MysqlColumnType::Bigint),
            "BigintUnsigned" => Ok(MysqlColumnType::BigintUnsigned),
            "Numeric" => Ok(MysqlColumnType::Numeric),
            "Float" => Ok(MysqlColumnType::Float),
            "Double" => Ok(MysqlColumnType::Double),
            "Decimal" => Ok(MysqlColumnType::Decimal),
            "Date" => Ok(MysqlColumnType::Date),
            "Time" => Ok(MysqlColumnType::Time),
            "Datetime" => Ok(MysqlColumnType::Datetime),
            "Timestamp" => Ok(MysqlColumnType::Timestamp),
            "Year" => Ok(MysqlColumnType::Year),
            "Blob" => Ok(MysqlColumnType::Blob),
            "Json" => Ok(MysqlColumnType::Json),
            _ => bail!("Unknown MysqlDataType"),
        }
    }
}

pub struct MysqlColumnDefinition{
    pub name: String,
    pub name_unmodified: String,
    pub column_definition: String,
    pub default_value: String,
}

pub struct TableFieldConstructInfo {
    pub field_name:String,
    pub file_type:String,
    pub default_value_on_new:String,
    pub import_statements:Vec<String>
}