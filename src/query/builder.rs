use crate::mapping::description::{Table, Column, Holding};
use std::{fmt, fmt::write, format, result};
use std::collections::HashMap;
use std::io::Write;
use serde::{Deserialize, Serialize};
use sqlx::{Column as MysqlColumn, Error, Row, TypeInfo, Value};
use crate::mapping::column_types::{Boolean, Bigint, Char, Tinytext, Varchar, Date, Decimal, Timestamp, Int, Datetime, Enum};
use sqlx_mysql::{MySqlQueryResult, MySqlRow, MySqlTypeInfo};
use sqlx_mysql::{MySqlPool, MySqlPoolOptions};
use url::Url;
use lazy_static::lazy_static;
use std::sync::Mutex;
use serde_json::{json, Number};
use serde_json::Value as JsonValue;
use std::future::Future;
use chrono::{DateTime, NaiveTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use sqlx::Executor;
use sqlx::Database;
use crate::configuration::{encryptor, get_encryptor};
use crate::query::pool::{POOL};
use crate::utils::stringUtils::to_camel_case;
use crate::mapping::description::SqlColumn;
use crate::query::builder::JoinType::{INNER, LEFT};
use crate::query::select;

#[derive(Debug, Serialize, Deserialize)]
pub enum BuildErrorType {
    MissingOperation,
    MissingCondition,
    MissingTargetTable,
    MissingPrimaryKey,
    MissingPrimaryKeyValue,
    MissingFields,
    MissingValues,
    OtherError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryBuildError {
    pub(crate) error: BuildErrorType,
    pub(crate) message: String,
}

impl QueryBuildError{
    pub fn new(error: BuildErrorType, message:String) -> Self{
        QueryBuildError{
            error, message
        }
    }
}

pub trait RowMappable{
    fn from_row(row: &MySqlRow) -> Self;
}

#[derive(Debug,Clone)]
pub struct Condition {
    pub query: String,
}

impl Condition {
    pub fn new(query: String) -> Condition {
        Condition { query }
    }

    pub fn and(self, other: Condition) -> Condition {
        Condition {
            query: format!("({}) AND ({})", self.query, other.query),
        }
    }

    pub fn and_not_exists(self, other: QueryBuilder) -> Condition {
        Condition {
            query: format!("({}) AND NOT EXISTS ({})", self.query, other.build().unwrap_or_default()),
        }
    }

    pub fn or(self, other: Condition) -> Condition {
        Condition {
            query: format!("({}) OR ({})", self.query, other.query),
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.query)
    }
}

#[derive(Debug,Clone)]
pub enum Operation{
    Select,Insert,Update,Insert_Or_Update,Delete
}

#[derive(Debug,Clone)]
pub(crate) struct TargetTable{
    pub name:String,
    pub columns:Vec<SqlColumn>,
    pub primary_key:Vec<SqlColumn>
}

impl TargetTable {
    pub fn new(table: & dyn Table) -> TargetTable {
        TargetTable{
            name: table.name(),
            columns: table.all_columns(),
            primary_key: table.primary_key(),
        }
    }
}

#[derive(Debug,Clone)]
pub enum JoinType {
    LEFT,INNER
}
impl fmt::Display for JoinType {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LEFT => write!(f,"LEFT"),
            INNER => write!(f,"INNER"),
        }
    }
}
#[derive(Debug,Clone)]
pub struct TableJoin{
    target_table: TargetTable,
    join_type:JoinType,
    condition:Option<Condition>,
}

impl TableJoin{
    pub fn new(target_table: TargetTable, join_type:JoinType, condition:Option<Condition>) -> Self {
        TableJoin{
            target_table:target_table.clone(), join_type,
            condition
        }
    }

    pub fn applyCondition(&mut self, condition: Condition) -> TableJoin {
        self.condition = Some(condition);
        self.clone()
    }
}

#[derive(Debug,Clone)]
pub struct LimitJoin{
    offset: i32,
    row_count: i32,
}
impl LimitJoin{
    pub fn new(offset: i32, row_count: i32) -> Self {
        LimitJoin{
            offset,
            row_count,
        }
    }
}
#[derive(Debug,Clone)]
pub struct Field {
    pub table: String,
    pub name: String,
    pub as_: Option<String>,
    pub is_encrypted: bool,
}

impl Field {
    pub fn new(table: &str, name: &str, as_: Option<String>, is_encrypted: bool) -> Self {
        Field{
            table:table.to_string(),name:name.to_string(), as_, is_encrypted,
        }
    }
}

#[derive(Debug,Clone)]
pub struct SubqueryField {
    pub query_builder: QueryBuilder,
    pub as_: Option<String>,
}

#[derive(Debug,Clone)]
pub enum SelectField {
    Field(Field),
    Subquery(SubqueryField),
    Untyped(String),
}

impl From<&SqlColumn> for SelectField {
    fn from(value: &SqlColumn) -> SelectField {
        match value {
            SqlColumn::Char(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(), def.alias(),def.is_encrypted()))),
            SqlColumn::Varchar(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Tinytext(col_def) =>col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Text(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Mediumtext(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Longtext(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Enum(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Set(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Boolean(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Tinyint(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Smallint(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Int(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Bigint(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::BigintUnsigned(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Numeric(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Float(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Double(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Decimal(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Date(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Time(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Datetime(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Timestamp(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Year(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Blob(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Json(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
        }
    }
}

impl From<SqlColumn> for SelectField {
    fn from(value: SqlColumn) -> SelectField {
        match value {
            SqlColumn::Char(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(), def.alias(),def.is_encrypted()))),
            SqlColumn::Varchar(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Tinytext(col_def) =>col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Text(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Mediumtext(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Longtext(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Enum(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Set(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Boolean(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Tinyint(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Smallint(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Int(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Bigint(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::BigintUnsigned(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Numeric(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Float(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Double(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Decimal(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Date(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Time(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Datetime(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Timestamp(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Year(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Blob(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
            SqlColumn::Json(col_def) => col_def.clone().map_or(SelectField::Untyped("".to_string()),|def|SelectField::Field(Field::new(&def.table(),&def.name(),def.alias(),def.is_encrypted()))),
        }
    }
}

impl From<&Varchar> for SelectField{
    fn from(varchar: &Varchar) -> SelectField {
        match varchar.holding() {
            Holding::Name=> {
                SelectField::Field(Field::new(&varchar.table(),&varchar.name(),varchar.alias(),varchar.is_encrypted()))
            },
            Holding::Value => {
                if varchar.alias().is_some() {
                    SelectField::Untyped(format!("'{}' as {}",&varchar.value().unwrap_or_default(), varchar.alias().unwrap()))
                }else {
                    SelectField::Untyped(format!("'{}'",&varchar.value().unwrap_or_default()))
                }
            },
            Holding::NameValue=> {
                SelectField::Field(Field::new(&varchar.table(),&varchar.name(),varchar.alias(),varchar.is_encrypted()))
            },
            Holding::SubQuery=> {
                if let Some(sub_query) = varchar.sub_query(){
                    SelectField::Subquery(SubqueryField{query_builder:sub_query, as_:Some(varchar.name())})
                }else{
                    SelectField::Subquery(SubqueryField{query_builder:select(vec![SelectField::Untyped("'?'".to_string())]), as_:Some(varchar.name())})
                }
            }
        }
    }
}
impl From<Varchar> for SelectField{
    fn from(varchar: Varchar) -> SelectField {
        match varchar.holding() {
            Holding::Name=> {
                SelectField::Field(Field::new(&varchar.table(),&varchar.name(),varchar.alias(),varchar.is_encrypted()))
            },
            Holding::Value => {
                if varchar.alias().is_some() {
                    SelectField::Untyped(format!("{} as {}",&varchar.value().unwrap_or_default(), varchar.alias().unwrap()))
                }else {
                    SelectField::Untyped(format!("{}",&varchar.value().unwrap_or_default()))
                }
            },
            Holding::NameValue=> {
                SelectField::Field(Field::new(&varchar.table(),&varchar.name(),varchar.alias(),varchar.is_encrypted()))
            },
            Holding::SubQuery=> {
                if let Some(sub_query) = varchar.sub_query(){
                    SelectField::Subquery(SubqueryField{query_builder:sub_query, as_:Some(varchar.name())})
                }else{
                    SelectField::Subquery(SubqueryField{query_builder:select(vec![SelectField::Untyped("".to_string())]), as_:Some(varchar.name())})
                }
            }
        }
    }
}
impl From<Int> for SelectField{
    fn from(value: Int) -> SelectField {
        match value.holding() {
            Holding::Name=> {
                SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
            },
            Holding::Value => {
                if value.alias().is_some() {
                    SelectField::Untyped(format!("{} as {}",&value.value().unwrap_or_default(), value.alias().unwrap()))
                }else {
                    SelectField::Untyped(format!("{}",&value.value().unwrap_or_default()))
                }
            },
            Holding::NameValue=> {
                SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
            },
            Holding::SubQuery=> {
                if let Some(sub_query) = value.sub_query(){
                    SelectField::Subquery(SubqueryField{query_builder:sub_query, as_:Some(value.name())})
                }else{
                    SelectField::Subquery(SubqueryField{query_builder:select(vec![SelectField::Untyped("".to_string())]), as_:Some(value.name())})
                }
            }
        }
    }
}
impl From<Boolean> for SelectField{
    fn from(value: Boolean) -> SelectField {
        match value.holding() {
            Holding::Name=> {
                SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
            },
            Holding::Value => {
                if value.alias().is_some() {
                    SelectField::Untyped(format!("{} as {}",&value.value().unwrap_or_default(), value.alias().unwrap()))
                }else {
                    SelectField::Untyped(format!("{}",&value.value().unwrap_or_default()))
                }
            },
            Holding::NameValue=> {
                SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
            },
            Holding::SubQuery=> {
                if let Some(sub_query) = value.sub_query(){
                    SelectField::Subquery(SubqueryField{query_builder:sub_query, as_:Some(value.name())})
                }else{
                    SelectField::Subquery(SubqueryField{query_builder:select(vec![SelectField::Untyped("".to_string())]), as_:Some(value.name())})
                }
            }
        }
    }
}
impl From<Datetime> for SelectField{
    fn from(value: Datetime) -> SelectField {
        match value.holding() {
            Holding::Name=> {
                SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
            },
            Holding::Value => {
                if value.alias().is_some() {
                    SelectField::Untyped(format!("{} as {}",&value.value().unwrap_or_default(), value.alias().unwrap()))
                }else {
                    SelectField::Untyped(format!("{}",&value.value().unwrap_or_default()))
                }
            },
            Holding::NameValue=> {
                SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
            },
            Holding::SubQuery=> {
                if let Some(sub_query) = value.sub_query(){
                    SelectField::Subquery(SubqueryField{query_builder:sub_query, as_:Some(value.name())})
                }else{
                    SelectField::Subquery(SubqueryField{query_builder:select(vec![SelectField::Untyped("".to_string())]), as_:Some(value.name())})
                }
            }
        }
    }
}

impl From<Bigint> for SelectField{
    fn from(value: Bigint) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl From<&Bigint> for SelectField{
    fn from(value: &Bigint) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl From<Date> for SelectField{
    fn from(value: Date) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl From<&Date> for SelectField{
    fn from(value: &Date) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl From<Decimal> for SelectField{
    fn from(value: Decimal) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl From<&Decimal> for SelectField{
    fn from(value: &Decimal) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl From<Timestamp> for SelectField{
    fn from(value: Timestamp) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl From<&Timestamp> for SelectField{
    fn from(value: &Timestamp) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl <T:Clone+Into<String>> From<Enum<T>> for SelectField{
    fn from(value: Enum<T>) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

impl <T:Clone+Into<String>> From<&Enum<T>> for SelectField{
    fn from(value: &Enum<T>) -> SelectField {
        SelectField::Field(Field::new(&value.table(),&value.name(),value.alias(),value.is_encrypted()))
    }
}

fn decrypt_field(field:Field) -> String {
    let encryptor = get_encryptor();
    encryptor.decrypt_field(field)
}

impl ToString for Field {
    fn to_string(&self) -> String {
        let mut qualified_field = if self.table.is_empty() {
            self.name.clone()
        }else {
            format!("{}.{}", self.table, self.name)
        };
        let mut alias = self.as_.clone();
        if self.is_encrypted {
            //这里如果是用于select的话，需要解密，如果是用于其它地方如Where ... 的话，需要加密。当前仅按select来处理.
            //如果要区分用途，需要在Field中增加一个context字段表示用途，如context:Select,Where ... ，目前先不增加
            qualified_field = decrypt_field(self.clone());
            if alias.is_none(){
                alias = Some(self.name.clone())
            }
        }
        if alias.is_some() {
            qualified_field = format!("{} AS {}", qualified_field, alias.unwrap());
        }
        qualified_field
    }
}

impl ToString for SubqueryField {
    fn to_string(&self) -> String {
        if let Ok(build_result) = self.query_builder.build() {
            if self.as_.is_some() {format!("({}) AS {}", build_result,self.as_.clone().unwrap_or_default())} else {build_result}
        }else {
            "[wrong subquery statement]".to_string()
        }
    }
}

impl ToString for SelectField {
    fn to_string(&self) -> String {
        match self {
            SelectField::Field(field) => field.to_string(),
            SelectField::Subquery(subquery) => subquery.to_string(),
            SelectField::Untyped(s) => s.clone(),
        }
    }
}



#[derive(Debug,Clone)]
pub struct QueryBuilder {
    operation:Operation,
    is_select_all:Option<bool>,
    distinct:Option<bool>,
    pub target_table: Option<TargetTable>,//Option<String>,
    select_fields: Vec<SelectField>,
    pending_join: Option<TableJoin>,
    joins: Vec<TableJoin>,
    //upsert_values: Vec<String>,//insert values or update values
    conditions: Vec<Condition>,
    limit:Option<LimitJoin>,
    order_by:Vec<SelectField>,
    group_by:Vec<SelectField>,
    update_values: Vec<(SelectField, SelectField)>, // 用于存储更新字段和值的元组
}

fn add_text_upsert_fields_values(name:String, value:Option<String>, insert_fields: &mut Vec<String>, insert_values: &mut Vec<String>, update_fields_values: &mut Vec<String>){
    if(!insert_fields.contains(&name)){
        update_fields_values.push(format!("{} = VALUES({})", &name, &name));
        insert_fields.push(name);
        if let Some(string_value) = value {
            insert_values.push(format!("'{}'", string_value));
        }else{
            insert_values.push("null".to_string());
        }
    }
}

fn add_non_text_upsert_fields_values(name:String, value:Option<String>, insert_fields: &mut Vec<String>, insert_values: &mut Vec<String>,update_fields_values: &mut Vec<String>){
    update_fields_values.push(format!("{} = VALUES({})", &name, &name));
    insert_fields.push(name);
    if let Some(string_value) = value {
        insert_values.push(format!("{}", string_value));
    }else{
        insert_values.push("null".to_string());
    }
}

fn construct_upsert_fields_values(columns:&Vec<SqlColumn>, insert_fields: &mut Vec<String>, insert_values: &mut Vec<String>,update_fields_values: &mut Vec<String>,skip_field_names:Vec<String>){
    for column_def in columns {
        match column_def {
            SqlColumn::Varchar(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),col.value(),insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Char(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),col.value(),insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Tinytext(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),col.value(),insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Text(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),col.value(),insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Mediumtext(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),col.value(),insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Longtext(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),col.value(),insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Enum(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),col.value_as_string(),insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Set(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),col.value_as_string(),insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Boolean(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { if value {Some("1".to_string())} else {Some("0".to_string())}} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Tinyint(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Smallint(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Int(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Bigint(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::BigintUnsigned(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Numeric(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Float(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Double(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Decimal(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Date(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.format("%Y-%m-%d").to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Time(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.format("%H:%M:%S").to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Datetime(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.format("%Y-%m-%d %H:%M:%S").to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Timestamp(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.format("%Y-%m-%d %H:%M:%S").to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Year(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { Some(value.to_string())} else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Blob(column_def) => {//todo:how?
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_non_text_upsert_fields_values(col.name(),if let Some(value) = col.value() { if let Ok(string) = String::from_utf8(value.clone()) {Some(string)} else {None} } else {None},insert_fields,insert_values,update_fields_values);
                    }
                }
            }
            SqlColumn::Json(column_def) => {
                if let Some(col) = column_def {
                    if !skip_field_names.contains(&col.name()) {
                        add_text_upsert_fields_values(col.name(), if let Some(value) = col.value() { Some(value.to_string()) } else { None }, insert_fields, insert_values, update_fields_values);
                    }
                }
            }
        }
    }
}

pub fn construct_upsert_primary_key_value(columns:&Vec<SqlColumn>, insert_fields: &mut Vec<String>, insert_values: &mut Vec<String>, primary_key_as_conditions: &mut Vec<String>) {
    for primary_key_def in columns {
        match primary_key_def {
            SqlColumn::Varchar(column_def) => {
                if let Some(col) = column_def {
                    if let Some(string_value) = col.value(){
                        insert_fields.push(col.name());
                        insert_values.push(format!("'{}'", &string_value));
                        primary_key_as_conditions.push(format!("{} = '{}'", col.name(), string_value));
                    }
                }/*else{
                    return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Primary key's value not found for upsert operation".to_string()));
                }*/
            }
            SqlColumn::Char(column_def) => {
                if let Some(col) = column_def {
                    if let Some(string_value) = col.value(){
                        insert_fields.push(col.name());
                        insert_values.push(format!("'{}'", &string_value));
                        primary_key_as_conditions.push(format!("{} = '{}'", col.name(), string_value));
                    }/*else{
                        return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Empty primary key value for upsert operation".to_string()));
                    }*/
                }/*else{
                    return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Primary key's value not found for upsert operation".to_string()));
                }*/
            }
            SqlColumn::Int(column_def) => {
                if let Some(col) = column_def {
                    if let Some(value) = col.value(){
                        insert_fields.push(col.name());
                        insert_values.push(value.to_string());
                        primary_key_as_conditions.push(format!("{} = '{}'", col.name(), &value));
                    }/*else{
                        return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Empty primary key value for upsert operation".to_string()));
                    }*/
                }/*else{
                    return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Primary key's value not found for upsert operation".to_string()));
                }*/
            }
            SqlColumn::Bigint(column_def) => {
                if let Some(col) = column_def {
                    if let Some(value) = col.value(){
                        insert_fields.push(col.name());
                        insert_values.push(value.to_string());
                        primary_key_as_conditions.push(format!("{} = '{}'", col.name(), &value));
                    }/*else{
                        return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Empty primary key value for upsert operation".to_string()));
                    }*/
                }/*else{
                    return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Primary key's value not found for upsert operation".to_string()));
                }*/
            }
            SqlColumn::BigintUnsigned(column_def) => {
                if let Some(col) = column_def {
                    if let Some(value) = col.value(){
                        insert_fields.push(col.name());
                        insert_values.push(value.to_string());
                        primary_key_as_conditions.push(format!("{} = '{}'", col.name(), &value));
                    }/*else{
                        return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Empty primary key value for upsert operation".to_string()));
                    }*/
                }/*else{
                    return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKeyValue, "Primary key's value not found for upsert operation".to_string()));
                }*/
            }
            _ => {}
        }
    }
}

impl QueryBuilder {

    pub fn select_all_fields() -> QueryBuilder {
        QueryBuilder { operation:Operation::Select, is_select_all: Some(true), distinct: None, target_table:None, select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![], */limit: None, order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn init_with_select_fields(fields: Vec<SelectField>) -> QueryBuilder {
        //let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder { operation:Operation::Select, is_select_all:None, distinct: None, target_table:None, select_fields:fields, pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![],*/ limit: None, order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn init_with_select_all_fields<A>(table: & A) -> QueryBuilder where A : Table {
        //let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder { operation:Operation::Select, is_select_all:Some(true), distinct: None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![],*/ limit: None, order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn init_with_select_distinct_fields(fields: Vec<SelectField>) -> QueryBuilder {
        //let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder { operation:Operation::Select, is_select_all:None, distinct: Some(true), target_table:None, select_fields:fields, pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![],*/ limit: None, order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn insert_into_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        //table.insert_query_builder()
        QueryBuilder { operation:Operation::Insert, is_select_all:None, distinct: None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![],*/ limit: None, order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn update_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        //table.update_query_builder()
        QueryBuilder { operation:Operation::Update,is_select_all:None, distinct: None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![], */limit: None, order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn upsert_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Insert_Or_Update,is_select_all:None, distinct: None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![],*/ limit: None, order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn delete_one_from<A>(table:& A) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, distinct: None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![], */limit: Some(LimitJoin::new(0, 1)), order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn delete_one_where<A>(table:& A,condition: Condition) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, distinct: None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![condition],/* upsert_values: vec![], */limit: Some(LimitJoin::new(0, 1)), order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn delete_all_where<A>(table:& A,condition: Condition) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, distinct: None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![condition],/* upsert_values: vec![], */limit: None, order_by: vec![], group_by: vec![], update_values: vec![] }
    }

    pub fn from<A>(mut self, table:& A) -> QueryBuilder where A : Table{
        self.target_table = Some(TargetTable::new(table));
        if let Some(select_all) = self.is_select_all {
            self.select_fields = table.all_columns().iter().map(|c|c.into()).collect::<Vec<SelectField>>();
        }
        /*if let Some(is_select_all) = self.is_select_all {
            if is_select_all {
                let fields_strs = table.columns().iter().map(|field| field.name()).collect();
                self.fields = fields_strs;
            }
        }*/
        self
    }

    pub fn left_join<A>(mut self, table:& A) -> QueryBuilder where A : Table{
        self.pending_join = Some(TableJoin::new(TargetTable::new(table),LEFT,None));
        self
    }

    pub fn inner_join<A>(mut self, table:& A) -> QueryBuilder where A : Table{
        self.pending_join = Some(TableJoin::new(TargetTable::new(table),INNER,None));
        self
    }

    pub fn add_join(mut self, join:TableJoin) -> QueryBuilder {
        self.joins.push(join);
        self
    }

    pub fn on(mut self, condition: Condition) -> QueryBuilder {
        if self.pending_join.is_some() {
            self.joins.push(self.pending_join.unwrap().applyCondition(condition));
            self.pending_join = None;
        }
        self
    }

    pub fn order_by<T: Into<SelectField>>(mut self, fields: Vec<T>) -> QueryBuilder{
        let fields: Vec<SelectField> = fields.into_iter().map(|field| field.into()).collect();
        self.order_by.extend(fields);
        self
    }

    pub fn group_by<T: Into<SelectField>>(mut self, fields: Vec<T>) -> QueryBuilder{
        let fields: Vec<SelectField> = fields.into_iter().map(|field| field.into()).collect();
        self.group_by.extend(fields);
        self
    }

    pub fn as_table(mut self, table: &str) -> InnerTable {
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);
            InnerTable {
                table_name: format!("({}) as {}", query_string, table),
                map_fields: self.select_fields.iter().map(|field| (field.clone().to_string(), Varchar::with_name(field.clone().to_string()))).collect::<HashMap<String, Varchar>>(),
            }
        }else{
            InnerTable{
                table_name: "".to_string(),
                map_fields: Default::default(),
            }
            
        }
    }

    pub fn add_select_fields(){

    }

    pub fn add_upsert_fields(){ //need key_values

    }


/*    pub fn with_entity(mut self, table:&'a A) -> QueryBuilder<'a> {
        self.target_table = Some(table);
        self
    }*/

    ///every call to where_, put a new condition or condition group to conditions
    pub fn where_(mut self, condition: Condition) -> QueryBuilder {
        self.conditions.push(condition);
        self
    }

    /// 设置要更新的字段和值
    pub fn set<T: Into<SelectField>>(mut self, field: T, value: T) -> QueryBuilder
    where
        T: Into<SelectField>,
    {
        let set_value: SelectField = value.into();
        self.update_values.push((field.into(), set_value));
        self
    }

    pub fn limit(mut self, limit: i32) -> QueryBuilder {
        self.limit = Some(LimitJoin::new(0, limit));
        self
    }

    pub fn limit_offset(mut self, limit: i32, offset: i32) -> QueryBuilder {
        self.limit = Some(LimitJoin::new(offset, limit));
        self
    }

    pub fn asVachar(mut self, name: &str) -> Varchar {
        Varchar::with_name_query(name.to_string(),Some(self))
    }

    pub fn as_(mut self, name: &str) -> Varchar {
        //Varchar::with_name_query(name.to_string(),Some(self))
        //SelectField::Subquery(SubqueryField{query_builder:self,as_:Some(name.to_string())})
        Varchar::with_name_query(name.to_string(),Some(self))
    }

    ///execute insert/update/delete and return the affected rows number
    pub async fn execute(&self) -> Result<MySqlQueryResult,Error> {
        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);
            let result = sqlx::query(&query_string).execute(pool).await? as MySqlQueryResult; // Pass the reference to sqlx::query()
            Ok(result)
        }else if let Err(e) = build_result {
            Err(Error::Configuration(e.message.into()))
        }else {
            Err(Error::Configuration("未知错误".into()))
        }
    }

    pub async fn fetch<T: Serialize + for<'de> serde::Deserialize<'de>>(&self) -> Result<Vec<T>, Error> {

        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);

            let jsons = sqlx::query(&query_string)
                .try_map(|row:MySqlRow| {
                    self.convert_to_json_value(row)
                })
                .fetch_all(pool)
                .await?;

            let mut result = Vec::new();
            for json in jsons {
                let item_parsed_result = serde_json::from_value::<T>(json.clone());
                match item_parsed_result {
                    Ok(item_parsed) => {
                        result.push(item_parsed);
                    }
                    Err(err) => {println!("error={:?}", err);}
                }
                // if let Ok(item_parsed) = item_parsed_result {
                //     result.push(item_parsed);
                // }else {
                //     println!("entity={:?}", json);
                //     // println!("注意类型不匹配");
                // }
            }
            Ok(result)
        }else if let Err(e) = build_result {
            Err(Error::Configuration(e.message.into()))
        }else {
            Err(Error::Configuration("未知错误".into()))
        }
    }

    pub async fn fetch_one<T: Serialize + for<'de> serde::Deserialize<'de>>(&mut self) -> Result<Option<T>,Error> {

        self.limit = Some(LimitJoin::new(0, 1));

        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);

            let json_result = sqlx::query(&query_string)
                .try_map(|row:MySqlRow| {
                    self.convert_to_json_value(row)
                })
                .fetch_one(pool)
                .await;

            match json_result {
                Ok(json)=>{
                    // println!("json of product {:#?}", json);
                    let json_parse_result = serde_json::from_value(json);
                    match json_parse_result {
                        Ok(json)=>{
                            Ok(Some(json))
                        },
                        Err(e) =>{
                            Err(Error::Decode(e.to_string().into()))
                        }
                    }
                },
                Err(e) => {
                    match e {
                        Error::RowNotFound => {
                            Ok(None)
                        }
                        _ => {
                            Err(e)
                        }
                    }
                }
            }
        }else if let Err(e) = build_result {
            Err(Error::Encode(e.message.into()))
        }else {
            Err(Error::Encode("未知错误".into()))
        }
    }
    pub async fn fetch_count(&self) -> Result<i64, Error> {
        let pool = POOL.get().unwrap();
        let build_result = self.build();
        if let Ok(query_string) = build_result {
            println!("query string {}", query_string);

            let value = sqlx::query(&query_string)
                .try_map(|row:MySqlRow| {
                    self.convert_to_number(row)
                })
                .fetch_one(pool)
                .await?;
            Ok(value)
        }else if let Err(e) = build_result {
            Err(Error::Configuration(e.message.into()))
        }else {
            Err(Error::Configuration("未知错误".into()))
        }
    }

    ///将mysql数据行转为JsonValue
    fn convert_to_json_value(&self, row:MySqlRow)-> Result<JsonValue, Error>{
        // println!("row of product {:#?}", row);
        let mut json_obj = json!({});
        let columns = row.columns();
        let mut i=0;
        for column in columns {
            let column_name = column.name();
            let camel_case_column_name = to_camel_case(&column_name);
            let mut type_name = column.type_info().name();
            let type_detail = format!("{:?}", column.type_info());
            if type_detail.contains("ColumnFlags(SET)"){
                type_name = "SET";
            }
            // println!("type_name of {} {} {}",column_name, type_name, type_detail);
            match type_name {
                "VARCHAR" => {
                    let value_result: Result<Option<String>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            json_obj[column_name] = serde_json::Value::String(value.clone());
                            json_obj[camel_case_column_name] = serde_json::Value::String(value);
                        }else {
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "INT" | "BIGINT" => {
                    let value_result: Result<Option<i32>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            json_obj[column_name] = value.clone().into();
                            json_obj[camel_case_column_name] = value.into();
                        }else {
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "DECIMAL" => {
                    let value_result: Result<Option<rust_decimal::Decimal>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            // 将 Decimal 转换为 f64
                            if let Some(float_value) = value.to_f64() {
                                json_obj[column_name] = float_value.clone().into();
                                json_obj[camel_case_column_name] = float_value.into();
                            } else {
                                json_obj[column_name] = serde_json::Value::Null;
                                json_obj[camel_case_column_name] = serde_json::Value::Null;
                            }
                        }else {
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "BOOLEAN" => {
                    let value_result: Result<Option<i8>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            json_obj[column_name] = if value>0 {serde_json::Value::Bool(true)} else {serde_json::Value::Bool(false)};
                            json_obj[camel_case_column_name] = if value>0 {serde_json::Value::Bool(true)} else {serde_json::Value::Bool(false)};
                        }else {
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "ENUM" => {
                    let value_result: Result<Option<String>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            json_obj[column_name] = serde_json::Value::String(value.clone());
                            json_obj[camel_case_column_name] = serde_json::Value::String(value);
                        }else {
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "SET" => {
                    let value_result: Result<Option<String>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            let mut values:Vec<serde_json::Value> = vec![];
                            if !value.is_empty() {
                                values = value.split(',')
                                    .map(|s| serde_json::Value::String(s.trim().to_string()))  // Optional: trim whitespace and convert to String
                                    .collect::<Vec<_>>();
                            }
                            json_obj[column_name] = serde_json::Value::Array(values.clone());
                            json_obj[camel_case_column_name] = serde_json::Value::Array(values);
                        }else {
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "DATETIME" => {
                    // Handle DATETIME type
                    let value_result: Result<Option<chrono::NaiveDateTime>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            let timestamp = value.and_utc().timestamp();
                            json_obj[column_name] = serde_json::Value::Number(serde_json::Number::from(timestamp));
                            json_obj[camel_case_column_name] = serde_json::Value::Number(serde_json::Number::from(timestamp));

                        }else{
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "TIMESTAMP" => {
                    let value_result: Result<Option<DateTime<Utc>>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            let cur_value = value.timestamp_millis();
                            json_obj[column_name] = serde_json::Value::Number(serde_json::Number::from(cur_value));
                            json_obj[camel_case_column_name] = serde_json::Value::Number(serde_json::Number::from(cur_value));
                        }else{
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "DATE" => {
                    let value_result: Result<Option<chrono::NaiveDate>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            let timestamp = value.and_time(NaiveTime::default()).and_utc().timestamp_millis();
                            json_obj[column_name] = serde_json::Value::Number(serde_json::Number::from(timestamp));
                            json_obj[camel_case_column_name] = serde_json::Value::Number(serde_json::Number::from(timestamp));

                        }else{
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "CHAR" | "TEXT" | "LONGTEXT" => {
                    // Handle CHAR type
                    let value_result: Result<Option<String>, _> = row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            json_obj[column_name] = serde_json::Value::String(value.clone());
                            json_obj[camel_case_column_name] = serde_json::Value::String(value);
                        }else {
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null;
                        }
                    } else if let Err(err) = value_result {
                        eprintln!("Error deserializing value for column '{}': {}", column_name, err);
                    }
                }
                "VARBINARY" => {
                    let value_result: Result<Option<Vec<u8>>, _>= row.try_get(i);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            match String::from_utf8(value) {
                                Ok(utf8_str) => {
                                    json_obj[column_name] = serde_json::Value::String(utf8_str.to_string());
                                    json_obj[camel_case_column_name] = serde_json::Value::String(utf8_str.to_string());
                                }
                                Err(_) => {
                                    json_obj[column_name] = serde_json::Value::Null;
                                    json_obj[camel_case_column_name] = serde_json::Value::Null
                                }
                            }
                        }else {
                            json_obj[column_name] = serde_json::Value::Null;
                            json_obj[camel_case_column_name] = serde_json::Value::Null
                        }
                    }
                }
                &_ => {
                    println!("type_name of {} {} {}",column_name, type_name, type_detail);
                    json_obj[column_name] = serde_json::Value::Null;
                    json_obj[camel_case_column_name] = serde_json::Value::Null;
                }
            }

            i += 1;
        }
        Ok(json_obj)
    }
    fn convert_to_number(&self,row:MySqlRow) -> Result<i64, Error>{
        let mut count_res :i64 = 0;
        let colum = row.columns().get(0);
        if let Some(colum) = colum{
            let column_name = colum.name();
            let type_name = colum.type_info().name();
            match type_name {
                "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" => {
                    let value_result:Result<Option<i64>, _> = row.try_get(0);
                    if let Ok(value) = value_result {
                        if let Some(value) = value {
                            count_res =value.clone().into();
                        }
                    }
                }
                &_ => {
                    println!("type_name of {} {}",column_name, type_name);
                }
            }
        }
        Ok(count_res)
    }

    fn populate_select_fields_as_string(&self) -> String {
        self.select_fields.clone().into_iter().map(|field| field.to_string()).collect::<Vec<String>>().join(",")
    }

    pub fn build(&self) -> Result<String,QueryBuildError> {
        let mut queryString = "".to_string();
        match self.operation {
            Operation::Select => {
                if !self.select_fields.is_empty() {
                    if self.distinct.is_some() && self.distinct.unwrap() {
                        queryString = format!("select distinct {}",self.populate_select_fields_as_string());//self.select_fields.join(", ")
                    }else {
                        queryString = format!("select {}",self.populate_select_fields_as_string());//self.select_fields.join(", ")
                    }
                }else {
                    if let Some(true) = self.is_select_all {
                        queryString = "select *".to_string();
                    }else{
                        return Err(QueryBuildError::new(BuildErrorType::MissingFields,"please provide at lease on field for select operation".to_string()));
                    }
                }
                if self.target_table.is_some() {
                    queryString = format!("{} from {}",queryString, self.target_table.clone().unwrap().name);
                }else {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable,"please provide table name to select from".to_string()));
                }
                if !self.joins.is_empty() {
                    // Traverse joins and generate JOIN statements for each TableJoin
                    for (i, join) in self.joins.iter().enumerate() {
                        // Generate JOIN statements based on joinotype
                        queryString.push_str(&format!(" {} JOIN {} ON {} ", join.join_type.to_string(), join.target_table.name, join.clone().condition.unwrap().query));
                    }
                }
                if self.conditions.len() > 0 {
                    queryString = format!("{} where {}",queryString, self.conditions.iter()
                            .map(|condition| condition.query.clone())
                            .collect::<Vec<String>>()
                            .join(" AND "));
                }
                if self.group_by.len() > 0 {
                    queryString = format!("{} group by {}",queryString, self.group_by.iter()
                            .map(|str| str.clone().to_string())
                            .collect::<Vec<String>>()
                            .join(", "));
                }
                if self.order_by.len() > 0 {
                    queryString = format!("{} order by {}",queryString, self.order_by.iter()
                            .map(|str| str.clone().to_string())
                            .collect::<Vec<String>>()
                            .join(", "));
                }
                if self.limit.is_some() {
                    queryString = format!("{} limit {}, {}",queryString,self.clone().limit.unwrap().offset,self.clone().limit.unwrap().row_count);
                }
            },
            Operation::Insert => {
                if self.target_table.is_none() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable, "please provide table name for insert operation".to_string()));
                }
                let target_table = self.target_table.clone().unwrap();

                if target_table.primary_key.is_empty() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKey, "Primary key not found for upsert operation".to_string()));
                }
                let mut insert_fields: Vec<String> = Vec::new();
                let mut insert_values: Vec<String> = Vec::new();
                construct_upsert_primary_key_value(&target_table.primary_key,&mut insert_fields, &mut insert_values,&mut vec![]);
                construct_upsert_fields_values(&target_table.columns, &mut insert_fields, &mut insert_values, &mut vec![], target_table.primary_key.iter().map(|it|it.get_col_name()).collect::<Vec<String>>());
                //decrypt?
                queryString = format!("INSERT INTO {} ({}) VALUES ({})", &target_table.name, insert_fields.join(", "), insert_values.join(", "));
            },
            Operation::Update => {
                if self.target_table.is_none() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable, "please provide table name for update operation".to_string()));
                }
                let target_table = self.target_table.clone().unwrap();

                if target_table.primary_key.is_empty() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKey, "Primary key not found for upsert operation".to_string()));
                }

                if self.conditions.is_empty() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingCondition, "please provide at least one condition for update operation".to_string()));
                }

                let mut primary_key_conditions = Vec::<String>::new();
                let mut update_fields_values: Vec<String> = Vec::new();
                if self.update_values.is_empty() {
                    construct_upsert_fields_values(&target_table.columns, &mut vec![], &mut vec![], &mut update_fields_values, target_table.primary_key.iter().map(|it|it.get_col_name()).collect::<Vec<String>>());
                }else{
                    update_fields_values = self.update_values
                                        .iter()
                                        .map(|(field, value)| format!("{} = {}", field.clone().to_string(), value.clone().to_string()))
                                        .collect();
                }
                construct_upsert_primary_key_value(&target_table.primary_key,&mut vec![], &mut vec![], &mut primary_key_conditions);
                //decrypt?
                queryString = format!("update {} set {} where {}", self.target_table.clone().unwrap().name, update_fields_values.join(", "), primary_key_conditions.iter()
                    .map(|condition| condition.clone())
                    .collect::<Vec<String>>()
                    .join(" AND "));
            },
            Operation::Insert_Or_Update => {
                if self.target_table.is_none() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable, "please provide table name for upsert operation".to_string()));
                }
                let target_table = self.target_table.clone().unwrap();

                if target_table.primary_key.is_empty() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingPrimaryKey, "Primary key not found for upsert operation".to_string()));
                }
                let mut insert_fields: Vec<String> = Vec::new();
                let mut insert_values: Vec<String> = Vec::new();
                let mut update_fields_values: Vec<String> = Vec::new();

                construct_upsert_primary_key_value(&target_table.primary_key,&mut insert_fields, &mut insert_values, &mut vec![]);
                construct_upsert_fields_values(&target_table.columns, &mut insert_fields, &mut insert_values, &mut update_fields_values,target_table.primary_key.iter().map(|it|it.get_col_name()).collect::<Vec<String>>());
                //decrypt?
                queryString = format!("INSERT INTO {} ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {};", &target_table.name, insert_fields.join(", "), insert_values.join(", "), update_fields_values.join(", "));
            },
            Operation::Delete => {
                if self.target_table.is_none() {
                    return Err(QueryBuildError::new(BuildErrorType::MissingTargetTable, "please provide table name for delete operation".to_string()));
                }
                if self.conditions.len() <= 0 {
                    return Err(QueryBuildError::new(BuildErrorType::MissingCondition, "please provide filters for  delete operation".to_string()));
                }
                queryString = format!("delete from {} where {}", self.target_table.clone().unwrap().name, self.conditions.iter()
                    .map(|condition| condition.query.clone())
                    .collect::<Vec<String>>()
                    .join(" AND "));
            },
            _ => {
                return Err(QueryBuildError::new(BuildErrorType::MissingOperation,"please provide one of these operation Select, Insert, Update, Delete, Insert_Or_Update".to_string()));
            }
        }
        // println!("buider: {:#?}",self);
        //println!("queryString: {:#?}",queryString);
        println!("");
        Ok(queryString.to_string())
    }
}

pub struct InnerTable {
    pub table_name: String,
    pub map_fields: HashMap<String, Varchar>,
}

impl Table for InnerTable {
    fn name(&self) -> String {
        self.table_name.clone()
    }
    fn all_columns(&self) -> Vec<SqlColumn> {
        vec![]
    }
    fn primary_key(&self) -> Vec<SqlColumn> {
        vec![]
    }
    fn update_primary_key(&mut self, primary_key: Vec<SqlColumn>) -> () {

    }
}