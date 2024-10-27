use crate::mapping::description::{Holding, Column, MappedEnum, SqlColumnType, SqlColumn};
use crate::query::builder::{Condition, QueryBuilder};
use chrono::{Local, NaiveDate, NaiveTime};
use serde::{Serialize,Deserialize};

pub trait Table{
    fn name(&self) -> String;
    fn columns(&self) -> Vec<SqlColumn>;
    fn primary_key(&self) -> Vec<SqlColumn>;
}

/*impl<'a> Table<'a> {
    fn new(name: &'a str, comment: &'a str) -> Table<'a> {
        Table { name, comment }
    }
}*/

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Enum<T/*:MappedEnum*/> {
    value: Option<T>,
    name: String,
    holding: Holding
}

impl<T/*:MappedEnum*/> Enum<T> {
    pub fn name(name: String) -> Self {
        Enum { name:name, value: None ,holding: Holding::Name }
    }

    fn value(value: Option<T>) -> Self {
        Enum { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn name_value(name: String, value: Option<T>) -> Self {
        Enum { name:name, value:value, holding: Holding::Value }
    }
}

pub struct Varchar2<'a> {
    name: &'a str,
    //value: Option<String>,
    //holding: Holding,
}

impl <'a> Varchar2<'a> {
    pub fn name(name: &'a str) -> Varchar2<'a> {
        Varchar2 { name:name, /*value: None*/ /*,holding: Holding::Name*/ }
    }
}

#[derive(Clone,Debug)]
pub struct Varchar {
    name: String,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl Varchar {
    pub fn name(name: String) -> Self {
        Varchar { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn value(value: Option<String>) -> Self {
        Varchar { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn val(&self) -> String {
        self.value.clone().unwrap()
    }

    pub fn name_value(name: String, value: Option<String>) -> Self {
        Varchar { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn name_query(name: String, sub_query: Option<QueryBuilder>) -> Self {
        Varchar { name:name, value:Some("".to_string()), holding: Holding::SubQuery, sub_query:sub_query }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Varchar>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value.unwrap().to_string()),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

impl From<&str> for Varchar {
    fn from(s: &str) -> Self {
        Varchar::value(Some(s.to_string()))
    }
}



impl Column for Varchar {
    fn name(&self) -> String {
        //let type_self: Varchar = self as Varchar;
        self.name.clone()
    }

/*
    fn sql_type(&self) -> SqlColumnType {
        SqlColumnType::Varchar
    }*/
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Char{
    name: String,
    value: String,
    holding: Holding,
}

impl Char {
    pub fn new(name: String) -> Self {
        Char { name:name, value: "".to_string() ,holding: Holding::Name }
    }

    fn value(value: String) -> Self {
        Char { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Char>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

impl Column for Char {
    fn name(&self) -> String {
        //let type_self: Char = self as Char;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Tinytext{
    name: String,
    value: String,
    holding: Holding,
}

impl crate::mapping::types::Tinytext {
    fn new(name: String) -> Self {
        crate::mapping::types::Tinytext { name:name, value: "".to_string() ,holding: Holding::Name }
    }

    fn value(value: String) -> Self {
        crate::mapping::types::Tinytext { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Tinytext>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

impl Column for Tinytext {
    fn name(&self) -> String {
        //let type_self: Tinytext = self as Tinytext;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Text{
    name: String,
    value: String,
    holding: Holding,
}

impl crate::mapping::types::Text {
    pub fn new(name: String) -> Self {
        crate::mapping::types::Text { name:name, value: "".to_string() ,holding: Holding::Name }
    }

    fn value(value: String) -> Self {
        crate::mapping::types::Text { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Text>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

impl Column for crate::mapping::types::Text {
    fn name(&self) -> String {
        //let type_self: Text = self as Text;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Mediumtext{
    name: String,
    value: String,
    holding: Holding,
}

impl crate::mapping::types::Mediumtext {
    fn new(name: String) -> Self {
        crate::mapping::types::Mediumtext { name:name, value: "".to_string() ,holding: Holding::Name }
    }

    fn value(value: String) -> Self {
        crate::mapping::types::Mediumtext { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Mediumtext>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

impl Column for crate::mapping::types::Mediumtext {
    fn name(&self) -> String {
        //let type_self: Mediumtext = self as Mediumtext;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Longtext{
    name: String,
    value: String,
    holding: Holding,
}

impl crate::mapping::types::Longtext {
    pub fn new(name: String) -> Self {
        crate::mapping::types::Longtext { name:name, value: "".to_string() ,holding: Holding::Name }
    }

    fn value(value: String) -> Self {
        crate::mapping::types::Longtext { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Longtext>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

impl Column for crate::mapping::types::Longtext {
    fn name(&self) -> String {
        //let type_self: Longtext = self as Longtext;
        self.name.clone()
    }

}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Int{
    value: Option<i32>,
    name: String,
    holding: Holding
}

impl Int {
    pub fn name(name: String) -> Self {
        Int { name:name, value: None ,holding: Holding::Name }
    }

    pub fn value(value: Option<i32>) -> Self {
        Int { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn name_value(name: String, value: Option<i32>) -> Self {
        Int { name:name, value:value, holding: Holding::Value }
    }
}

impl Column for Int {
    fn name(&self) -> String {
        //let type_self: Int = self as Int;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Year{
    value: i32,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Year {
    fn new(name: String) -> Self {
        crate::mapping::types::Year { name:name, value: 0 ,holding: Holding::Name }
    }

    fn value(value: i32) -> Self {
        crate::mapping::types::Year { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Year {
    fn name(&self) -> String {
        //let type_self: Year = self as Year;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Set<T>{
    value: Option<Vec<T>>,
    name: String,
    holding: Holding
}

impl<T> Set<T> {
    pub fn name(name: String) -> Self {
        Set { name:name, value:None ,holding: Holding::Name }
    }

    pub fn value(value: Option<Vec<T>>) -> Self {
        Set { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn name_value(name: String, value: Option<Vec<T>>) -> Self {
        Set { name:name, value:value, holding: Holding::Value }
    }
}

impl <T> Column for Set<T> {
    fn name(&self) -> String {
        //let type_self: Set<T> = self as Set<T>;
        self.name.clone()
    }
}

impl <T:MappedEnum> Column for Enum<T> {
    fn name(&self) -> String {
        //let type_self: &Set = self as &Set;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Tinyint{
    value: i8,
    name: String,
    holding: Holding
}

impl Tinyint {
    pub fn new(name: String) -> Self {
        Tinyint { name:name, value: 0 ,holding: Holding::Name }
    }

    fn value(value: i8) -> Self {
        Tinyint { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for Tinyint {
    fn name(&self) -> String {
        //let type_self: &Tinyint = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Smallint{
    value: i16,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Smallint {
    fn new(name: String) -> Self {
        crate::mapping::types::Smallint { name:name, value: 0 ,holding: Holding::Name }
    }

    fn value(value: i16) -> Self {
        crate::mapping::types::Smallint { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Smallint {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Smallint = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Bigint{
    value: i64,
    name: String,
    holding: Holding
}

impl Bigint {
    pub fn new(name: String) -> Self {
        crate::mapping::types::Bigint { name:name, value: 0 ,holding: Holding::Name }
    }

    fn value(value: i64) -> Self {
        crate::mapping::types::Bigint { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for Bigint {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Bigint = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct BigintUnsigned{
    value: u64,
    name: String,
    holding: Holding
}

impl crate::mapping::types::BigintUnsigned {
    fn new(name: String) -> Self {
        crate::mapping::types::BigintUnsigned { name:name, value: 0 ,holding: Holding::Name }
    }

    fn value(value: u64) -> Self {
        crate::mapping::types::BigintUnsigned { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::BigintUnsigned {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::BigintUnsigned = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Numeric{
    value: f64,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Numeric {
    fn new(name: String) -> Self {
        crate::mapping::types::Numeric { name:name, value: 0.0 ,holding: Holding::Name }
    }

    fn value(value: f64) -> Self {
        crate::mapping::types::Numeric { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Numeric {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Numeric = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Float{
    value: f32,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Float {
    pub fn new(name: String) -> Self {
        crate::mapping::types::Float { name:name, value: 0.0 ,holding: Holding::Name }
    }

    fn value(value: f32) -> Self {
        crate::mapping::types::Float { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Float {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Float = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Double{
    value: f64,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Double {
    pub fn new(name: String) -> Self {
        crate::mapping::types::Double { name:name, value: 0.0 ,holding: Holding::Name }
    }

    fn value(value: f64) -> Self {
        crate::mapping::types::Double { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Double {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Double = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Decimal{
    value: f64,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Decimal {
    fn new(name: String) -> Self {
        crate::mapping::types::Decimal { name:name, value: 0.0 ,holding: Holding::Name }
    }

    fn value(value: f64) -> Self {
        crate::mapping::types::Decimal { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Decimal {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Decimal = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Date{
    value: chrono::NaiveDate,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Date {
    pub fn new(name: String) -> Self {
        crate::mapping::types::Date { name:name, value: NaiveDate::default() ,holding: Holding::Name }
    }

    pub fn value(value: chrono::NaiveDate) -> Self {
        crate::mapping::types::Date { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Date {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Date = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Time{
    value: chrono::NaiveTime,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Time {
    pub fn new(name: String) -> Self {
        crate::mapping::types::Time { name:name, value: NaiveTime::default() ,holding: Holding::Name }
    }

    fn value(value: chrono::NaiveTime) -> Self {
        crate::mapping::types::Time { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Time {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Time = self;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct DateTime{
    value: chrono::DateTime<Local>,
    name: String,
    holding: Holding
}

impl DateTime {
    fn new(name: String) -> Self {
        DateTime { name:name, value: Local::now() ,holding: Holding::Name }
    }

    fn value(value: chrono::DateTime<Local>) -> Self {
        DateTime { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for DateTime {
    fn name(&self) -> String {
        //let type_self: &DateTime = self as &DateTime;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Datetime{
    pub value: Option<chrono::NaiveDateTime>,
    pub name: String,
    pub holding: Holding
}

impl crate::mapping::types::Datetime {
    pub fn name(name: String) -> Self {
        crate::mapping::types::Datetime { name:name, value: None ,holding: Holding::Name }
    }

    fn value(value: Option<chrono::NaiveDateTime>) -> Self {
        crate::mapping::types::Datetime { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn name_value(name: String, value: Option<chrono::NaiveDateTime>) -> Self {
        crate::mapping::types::Datetime { name:name, value:value, holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Datetime {
    fn name(&self) -> String {
       // let type_self: &crate::mapping::types::Datetime = self as &crate::mapping::types::Datetime;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Timestamp{
    value: Option<chrono::NaiveDateTime>,
    name: String,
    holding: Holding
}

impl crate::mapping::types::Timestamp {
    pub fn name(name: String) -> Self {
        crate::mapping::types::Timestamp { name:name, value: None ,holding: Holding::Name }
    }

    fn value(value: Option<chrono::NaiveDateTime>) -> Self {
        crate::mapping::types::Timestamp { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn name_value(name: String, value: Option<chrono::NaiveDateTime>) -> Self {
        crate::mapping::types::Timestamp { name:name, value:value, holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Timestamp {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Timestamp = self as &crate::mapping::types::Timestamp;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Json{
    name: String,
    value: String,
    holding: Holding,
}

impl crate::mapping::types::Json {
    pub fn new(name: String) -> Self {
        crate::mapping::types::Json { name:name, value: "".to_string() ,holding: Holding::Name }
    }

    fn value(value: String) -> Self {
        crate::mapping::types::Json { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Json>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

impl Column for crate::mapping::types::Json {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Json = self as &crate::mapping::types::Json;
        self.name.clone()
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Blob{
    name: String,
    value: Vec<u8>,
    holding: Holding,
}

impl crate::mapping::types::Blob {
    fn new(name: String) -> Self {
        crate::mapping::types::Blob { name:name, value: vec![] ,holding: Holding::Name }
    }

    fn value(value: Vec<u8>) -> Self {
        crate::mapping::types::Blob { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for crate::mapping::types::Blob {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Blob = self as &crate::mapping::types::Blob;
        self.name.clone()
    }
}