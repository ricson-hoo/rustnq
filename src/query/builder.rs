use crate::mapping::description::{Table,Column};
use std::{fmt, fmt::write, format, result};
use std::io::Write;
use serde::{Deserialize, Serialize};
use sqlx::{Column as MysqlColumn, Error, Row, TypeInfo, Value};
use crate::mapping::types::{Char, Tinytext, Varchar};
use sqlx_mysql::{MySqlQueryResult, MySqlRow, MySqlTypeInfo};
use sqlx_mysql::{MySqlPool, MySqlPoolOptions};
use url::Url;
use lazy_static::lazy_static;
use std::sync::Mutex;
use serde_json::json;
use serde_json::Value as JsonValue;
use std::future::Future;
use sqlx::Executor;
use sqlx::Database;
use sqlx::IntoArguments;
use crate::query::pool::{POOL};
use crate::utils::stringUtils::to_camel_case;
use crate::mapping::description::SqlColumn;
use crate::query::builder::JoinType::{INNER, LEFT};

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
            columns: table.columns(),
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
pub struct QueryBuilder {
    operation:Operation,
    is_select_all:Option<bool>,
    pub target_table: Option<TargetTable>,//Option<String>,
    select_fields: Vec<String>,
    pending_join: Option<TableJoin>,
    joins: Vec<TableJoin>,
    //upsert_values: Vec<String>,//insert values or update values
    conditions: Vec<Condition>,
    limit:Option<i32>,
}

fn add_text_upsert_fields_values(name:String, value:Option<String>, insert_fields: &mut Vec<String>, insert_values: &mut Vec<String>,update_fields_values: &mut Vec<String>){
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
        QueryBuilder { operation:Operation::Select, is_select_all: Some(true), target_table:None, select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![], */limit: None }
    }

    pub fn init_with_select_fields(fields: Vec<String>) -> QueryBuilder {
        //let fields_strs = fields.iter().map(|field| field.name()).collect();
        QueryBuilder { operation:Operation::Select, is_select_all:None, target_table:None, select_fields:fields, pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![],*/ limit: None }
    }

    pub fn insert_into_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        //table.insert_query_builder()
        QueryBuilder { operation:Operation::Insert, is_select_all:None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![],*/ limit: None }
    }

    pub fn update_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        //table.update_query_builder()
        QueryBuilder { operation:Operation::Update,is_select_all:None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![], */limit: None }
    }

    pub fn upsert_table_with_value<A>(table:& A) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Insert_Or_Update,is_select_all:None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![],*/ limit: None }
    }

    pub fn delete_one_from<A>(table:& A) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![],/* upsert_values: vec![], */limit: Some(1) }
    }

    pub fn delete_one_where<A>(table:& A,condition: Condition) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![condition],/* upsert_values: vec![], */limit: Some(1) }
    }

    pub fn delete_all_where<A>(table:& A,condition: Condition) -> QueryBuilder where A : Table{
        QueryBuilder { operation:Operation::Delete,is_select_all:None, target_table:Some(TargetTable::new(table)), select_fields:vec![], pending_join: None, joins: vec![], conditions: vec![condition],/* upsert_values: vec![], */limit: None }
    }

    pub fn from<A>(mut self, table:& A) -> QueryBuilder where A : Table{
        self.target_table = Some(TargetTable::new(table));
        if let Some(select_all) = self.is_select_all {
            self.select_fields = table.columns().iter().map(|c|c.get_col_name()).collect::<Vec<String>>();
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

    pub fn limit(mut self, limit:i32) -> QueryBuilder {
        self.limit = Some(limit);
        self
    }

    pub fn asVachar(mut self, name: &str) -> Varchar {
        Varchar::with_name_query(name.to_string(),Some(self))
    }

    /*pub fn fetch_one_into<T: RowMappable>(&self) -> T {
        // 假设这里是查询并获取到的一行数据row
        let row: MySqlRow = MySqlRow::fmt();// = get_row_from_query_result();

        T::from_row(&row)
    }

    pub fn fetch_into<T: RowMappable>(&self) -> Vec<T> {
        // 假设这里是查询并获取到的一行数据row
        let row: MySqlRow;// = get_row_from_query_result();

        vec![T::from_row(&row)]
    }*/

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

    /*pub async fn execute_return<T>(&self) -> Result<T,Error> {
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
    }*/

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
                if let Ok(item_parsed) = item_parsed_result {
                    result.push(item_parsed);
                }else {
                    println!("{:?}", json);
                }
            }
            Ok(result)
        }else if let Err(e) = build_result {
            Err(Error::Configuration(e.message.into()))
        }else {
            Err(Error::Configuration("未知错误".into()))
        }
    }

    pub async fn fetch_one<T: Serialize + for<'de> serde::Deserialize<'de>>(&mut self) -> Result<Option<T>,Error> {

        self.limit = Some(1);

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
                    println!("json of product {:#?}", json);
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
                    Err(e)
                }
            }
        }else if let Err(e) = build_result {
            Err(Error::Encode(e.message.into()))
        }else {
            Err(Error::Encode("未知错误".into()))
        }
    }

    ///将mysql数据行转为JsonValue
    fn convert_to_json_value(&self, row:MySqlRow)-> Result<JsonValue, Error>{
        println!("row of product {:#?}", row);
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
            println!("type_name of {} {} {}",column_name, type_name, type_detail);
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
                "INT" => {
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
                "CHAR" => {
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
                &_ => {
                    json_obj[column_name] = serde_json::Value::Null;
                    json_obj[camel_case_column_name] = serde_json::Value::Null;
                }
            }

            i += 1;
        }
        Ok(json_obj)
    }

    pub fn build(&self) -> Result<String,QueryBuildError> {
        let mut queryString = "".to_string();
        match self.operation {
            Operation::Select => {
                if(!self.select_fields.is_empty()){
                    queryString = format!("select {}",self.select_fields.join(", "));
                }else {
                    return Err(QueryBuildError::new(BuildErrorType::MissingFields,"please provide at lease on field for select operation".to_string()));
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
                if self.limit.is_some() {
                    queryString = format!("{} limit {}",queryString,self.limit.unwrap());
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

                let mut primary_key_conditions = Vec::<String>::new();
                let mut update_fields_values: Vec<String> = Vec::new();
                construct_upsert_primary_key_value(&target_table.primary_key,&mut vec![], &mut vec![], &mut primary_key_conditions);
                construct_upsert_fields_values(&target_table.columns, &mut vec![], &mut vec![], &mut update_fields_values, target_table.primary_key.iter().map(|it|it.get_col_name()).collect::<Vec<String>>());

                /*for (key, value) in primary_key_fields.iter().zip(primary_key_values) {
                    primary_key_conditions.push(format!("{} = {}", key, value));
                }*/

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
        println!("buider: {:#?}",self);
        println!("queryString: {:#?}",queryString);
        Ok(queryString.to_string())
    }

}