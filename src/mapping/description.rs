use std::str::FromStr;
use std::error::Error;
use anyhow::bail;
use chrono::Local;
use crate::mapping::types::*;
use crate::utils::stringUtils;
use serde::{Serialize,Deserialize};
use sqlx::encode::IsNull;

pub trait Table{
    fn name(&self) -> String;
    fn columns(&self) -> Vec<SqlColumn>;
    fn primary_key(&self) -> Vec<SqlColumn>;
}

pub trait EntityEnum {}

#[derive(Serialize,Deserialize,Clone,Copy,Debug)]
pub enum Holding{
    Name,Value,NameValue,SubQuery
}

#[derive(Clone,Debug)]
pub enum SqlColumn<T = ()> {
    Char(Option<Char>),
    Varchar(Option<Varchar>),
    Tinytext(Option<Tinytext>),
    Text(Option<Text>),
    Mediumtext(Option<Mediumtext>),
    Longtext(Option<Longtext>),
    Enum(Option<Enum<T>>),
    Set(Option<Set<T>>),
    Boolean(Option<Boolean>),
    Tinyint(Option<Tinyint>),
    Smallint(Option<Smallint>),
    Int(Option<Int>),
    Bigint(Option<Bigint>),
    BigintUnsigned(Option<BigintUnsigned>),
    Numeric(Option<Numeric>),
    Float(Option<Float>),
    Double(Option<Double>),
    Decimal(Option<Decimal>),
    Date(Option<Date>),
    Time(Option<Time>),
    Datetime(Option<Datetime>),
    Timestamp(Option<Timestamp>),
    Year(Option<Year>),
    Blob(Option<Blob>),
    Json(Option<Json>),
}

#[derive(Debug,Clone)]
pub enum RustDataType {
    String,
    Enum,
    Vec,
    bool,
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
    chronoDateTimeLocal,
}

pub trait Column {
    fn name(&self) -> String;
    //fn value(&self) -> String;//?
}

pub trait MappedEnum {
    fn name(&self) -> &str;
}

impl FromStr for SqlColumn {
    type Err = anyhow::Error;
    fn from_str(mysql_col_type: &str) -> Result<Self, anyhow::Error> {
        let typeName = stringUtils::begin_with_upper_case(&stringUtils::to_camel_case(&mysql_col_type.replace(" ", "_")));
        match typeName.as_str() {
            "Char" => Ok(SqlColumn::Char(None)),
            "Varchar" => Ok(SqlColumn::Varchar(None)),
            "Tinytext" => Ok(SqlColumn::Tinytext(None)),
            "Text" => Ok(SqlColumn::Text(None)),
            "Mediumtext" => Ok(SqlColumn::Mediumtext(None)),
            "Longtext" => Ok(SqlColumn::Longtext(None)),
            "Enum" => Ok(SqlColumn::Enum(None)),
            "Set" => Ok(SqlColumn::Set(None)),
            "Tinyint" => Ok(SqlColumn::Tinyint(None)),
            "Smallint" => Ok(SqlColumn::Smallint(None)),
            "Int" => Ok(SqlColumn::Int(None)),
            "Bigint" => Ok(SqlColumn::Bigint(None)),
            "BigintUnsigned" => Ok(SqlColumn::BigintUnsigned(None)),
            "Numeric" => Ok(SqlColumn::Numeric(None)),
            "Float" => Ok(SqlColumn::Float(None)),
            "Double" => Ok(SqlColumn::Double(None)),
            "Decimal" => Ok(SqlColumn::Decimal(None)),
            "Date" => Ok(SqlColumn::Date(None)),
            "Time" => Ok(SqlColumn::Time(None)),
            "Datetime" => Ok(SqlColumn::Datetime(None)),
            "Timestamp" => Ok(SqlColumn::Timestamp(None)),
            "Year" => Ok(SqlColumn::Year(None)),
            "Blob" => Ok(SqlColumn::Blob(None)),
            "Json" => Ok(SqlColumn::Json(None)),
            _ => bail!("Unknown MysqlDataType"),
        }
    }
}

impl ToString for SqlColumn {
    fn to_string(&self) -> String {
        match self {
            SqlColumn::Char(_) => "SqlColumn::Char".to_string(),
            SqlColumn::Varchar(_) => "SqlColumn::Varchar".to_string(),
            SqlColumn::Tinytext(_) => "SqlColumn::Tinytext".to_string(),
            SqlColumn::Text(_) => "SqlColumn::Text".to_string(),
            SqlColumn::Mediumtext(_) => "SqlColumn::Mediumtext".to_string(),
            SqlColumn::Longtext(_) => "SqlColumn::Longtext".to_string(),
            SqlColumn::Enum(_) => "SqlColumn::Enum".to_string(),
            SqlColumn::Set(_) => "SqlColumn::Set".to_string(),
            SqlColumn::Boolean(_) => "SqlColumn::Boolean".to_string(),
            SqlColumn::Tinyint(_) => "SqlColumn::Tinyint".to_string(),
            SqlColumn::Smallint(_) => "SqlColumn::Smallint".to_string(),
            SqlColumn::Int(_) => "SqlColumn::Int".to_string(),
            SqlColumn::Bigint(_) => "SqlColumn::Bigint".to_string(),
            SqlColumn::BigintUnsigned(_) => "SqlColumn::BigintUnsigned".to_string(),
            SqlColumn::Numeric(_) => "SqlColumn::Numeric".to_string(),
            SqlColumn::Float(_) => "SqlColumn::Float".to_string(),
            SqlColumn::Double(_) => "SqlColumn::Double".to_string(),
            SqlColumn::Decimal(_) => "SqlColumn::Decimal".to_string(),
            SqlColumn::Date(_) => "SqlColumn::Date".to_string(),
            SqlColumn::Time(_) => "SqlColumn::Time".to_string(),
            SqlColumn::Datetime(_) => "SqlColumn::Datetime".to_string(),
            SqlColumn::Timestamp(_) => "SqlColumn::Timestamp".to_string(),
            SqlColumn::Year(_) => "SqlColumn::Year".to_string(),
            SqlColumn::Blob(_) => "SqlColumn::Blob".to_string(),
            SqlColumn::Json(_) => "SqlColumn::Json".to_string(),
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
    pub sql_column_type:Option<SqlColumn>,
    pub sql_column_type_modified:bool
}