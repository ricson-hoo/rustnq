use std::str::FromStr;
use std::error::Error;
use anyhow::bail;
use chrono::Local;
use crate::mapping::types::{Int, Varchar, Enum, Set, DateTime};
use crate::utils::stringUtils;
use serde::{Serialize,Deserialize};
use sqlx::encode::IsNull;

#[derive(Serialize,Deserialize,Clone,Copy,Debug)]
pub enum Holding{
    Name,Value,NameValue,SubQuery
}

pub enum SqlColumnNameAndValue {
    Char(String, Option<String>),
    Varchar(String, Option<String>),
    Tinytext(String, Option<String>),
    Text(String, Option<String>),
    Mediumtext(String, Option<String>),
    Longtext(String, Option<String>),
    Enum(String, Option<String>),
    Set(String, Option<Vec<String>>),
    Tinyint(String, Option<Vec<String>>),
    Smallint(String, Option<i16>),
    Int(String, Option<i32>),
    Bigint(String, Option<i64>),
    BigintUnsigned(String, Option<u64>),
    Numeric(String, Option<f64>),
    Float(String, Option<f64>),
    Double(String, Option<f64>),
    Decimal(String, Option<f64>),
    Date(String, Option<chrono::NaiveDate>),
    Time(String, Option<chrono::NaiveTime>),
    Datetime(String, Option<chrono::NaiveDateTime>),
    Timestamp(String, Option<chrono::NaiveDateTime>),
    Year(String, Option<i32>),
    Blob(String, Option<Vec<u8>>),
    Json(String, Option<String>),
}

pub trait Column {
    fn name(&self) -> String;
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

#[derive(Clone,Debug)]
pub enum SqlColumnType {
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

impl FromStr for SqlColumnType {
    type Err = anyhow::Error;
    fn from_str(mysql_col_type: &str) -> Result<Self, anyhow::Error> {
        let typeName = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&mysql_col_type.replace(" ", "_")));
        match typeName.as_str() {
            "Char" => Ok(SqlColumnType::Char),
            "Varchar" => Ok(SqlColumnType::Varchar),
            "Tinytext" => Ok(SqlColumnType::Tinytext),
            "Text" => Ok(SqlColumnType::Text),
            "Mediumtext" => Ok(SqlColumnType::Mediumtext),
            "Longtext" => Ok(SqlColumnType::Longtext),
            "Enum" => Ok(SqlColumnType::Enum),
            "Set" => Ok(SqlColumnType::Set),
            "Tinyint" => Ok(SqlColumnType::Tinyint),
            "Smallint" => Ok(SqlColumnType::Smallint),
            "Int" => Ok(SqlColumnType::Int),
            "Bigint" => Ok(SqlColumnType::Bigint),
            "BigintUnsigned" => Ok(SqlColumnType::BigintUnsigned),
            "Numeric" => Ok(SqlColumnType::Numeric),
            "Float" => Ok(SqlColumnType::Float),
            "Double" => Ok(SqlColumnType::Double),
            "Decimal" => Ok(SqlColumnType::Decimal),
            "Date" => Ok(SqlColumnType::Date),
            "Time" => Ok(SqlColumnType::Time),
            "Datetime" => Ok(SqlColumnType::Datetime),
            "Timestamp" => Ok(SqlColumnType::Timestamp),
            "Year" => Ok(SqlColumnType::Year),
            "Blob" => Ok(SqlColumnType::Blob),
            "Json" => Ok(SqlColumnType::Json),
            _ => bail!("Unknown MysqlDataType"),
        }
    }
}

pub struct MysqlColumnDefinition{
    pub name: String,
    pub name_unmodified: String,
    pub column_definition: String,
    pub default_value: String,
    pub is_primary_key: bool,
}

pub struct TableFieldConstructInfo {
    pub field_name:String,
    pub field_type:String,
    pub default_value_on_new:String,
    pub import_statements:Vec<String>,
    pub sql_raw_type:String, //如Char,Varchar,Tinytext,Datetime,Timestamp...
}