use std::str::FromStr;
use std::error::Error;
use anyhow::bail;
use chrono::Local;
use crate::mapping::types::*;
use crate::utils::stringUtils;
use serde::{Serialize,Deserialize};
use sqlx::encode::IsNull;

#[derive(Serialize,Deserialize,Clone,Copy,Debug)]
pub enum Holding{
    Name,Value,NameValue,SubQuery
}

#[derive(Clone,Debug)]
pub enum SqlColumn<T = ()> {
    Char(Char),
    Varchar(Varchar),
    Tinytext(Tinytext),
    Text(Text),
    Mediumtext(Mediumtext),
    Longtext(Longtext),
    Enum(Enum<T>),
    Set(Set<T>),
    Tinyint(Tinyint),
    Smallint(Smallint),
    Int(Int),
    Bigint(Bigint),
    BigintUnsigned(BigintUnsigned),
    Numeric(Numeric),
    Float(Float),
    Double(Double),
    Decimal(Decimal),
    Date(Date),
    Time(Time),
    Datetime(Datetime),
    Timestamp(Timestamp),
    Year(Year),
    Blob(Blob),
    Json(Json),
}

//#[derive(Clone,Debug)]
/*pub enum SqlColumnType {
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
}*/

#[derive(Debug,Clone)]
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

pub trait Column {
    fn name(&self) -> String;
    //fn value(&self) -> String;//?
}

pub trait MappedEnum {
    fn name(&self) -> &str;
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
    pub initial_assignment_with_name:String,
    pub initial_assignment_with_name_and_value:String,
    pub import_statements:Vec<String>,
    pub sql_raw_type:String, //如Char,Varchar,Tinytext,Datetime,Timestamp...
}