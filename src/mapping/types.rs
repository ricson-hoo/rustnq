use crate::mapping::description::{Holding, Column, MappedEnum, SqlColumn};
use crate::query::builder::{Condition, QueryBuilder};
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Serialize,Deserialize};

pub trait And {
    fn and(&self, other:& (impl Column + Clone+ 'static)) -> Vec<String>;
}

impl And for Vec<String> {
    fn and(&self, other:& (impl Column + Clone+ 'static)) -> Vec<String> {
        let mut new_vec = self.clone();
        new_vec.push(other.name());
        new_vec
    }
}


#[derive(Clone,Debug)]
pub struct Enum<T> {
    value: Option<T>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl<T> Enum<T> {
    pub fn with_name(name: String) -> Self {
        Enum { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    fn with_value(value: Option<T>) -> Self {
        Enum { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<T>) -> Self {
        Enum { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

}

impl <T:Clone + 'static> Column for Enum<T> { //impl <T:MappedEnum> Column for Enum<T>
    fn name(&self) -> String {
        self.name.clone()
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

    pub fn and(&self, other:& (impl Column + Clone+ 'static)) -> Vec<String> {
        vec![self.name.clone(), other.name()]
    }

    pub fn with_name(name: String) -> Self {
        Varchar { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        Varchar { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Varchar { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_query(name: String, sub_query: Option<QueryBuilder>) -> Self {
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
        Varchar::with_value(Some(s.to_string()))
    }
}

impl<T> From<Enum<T>> for Varchar where std::string::String: From<T> {
    fn from(a_enum: Enum<T>) -> Varchar {
        if let Some(enum_value) = a_enum.value {
            let str_value:String = String::from(enum_value);
            Varchar::with_value(Some(str_value))
        }else {
            Varchar::with_value(None)
        }
    }
}

impl<T> From<Set<T>> for Varchar where  std::string::String: From<T>,T:Clone {
    fn from(set: Set<T>) -> Varchar {
        let mut str_set:Vec<String> = vec![];
        let set_value = set.value;
        if let Some(set_value) = set_value {
            for a_enum in set_value{
                str_set.push(String::from(a_enum))
            }
        }
        let string_value: String = str_set.join(",");
        Varchar::with_value(Some(string_value))
    }
}

impl Column for Varchar {
    fn name(&self) -> String {
        //let type_self: Varchar = self as Varchar;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Char{
    name: String,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

fn build_equal_condition_for_string_type(self_name:String,input_holding:Holding,input_name:String,input_value:Option<String>) -> Condition {
    let output = match input_holding {
        Holding::Name => format!(" = {}",input_name),
        Holding::Value => match input_value {
            Some(value) => format!(" = '{}'",value),
            None => "is null".to_string()
        },
        _ => "build_equal_condition_for_string_type to do ".to_string() //subquery
    };
    Condition::new(format!("{} {}", self_name, output))
}

impl Char {
    pub fn with_name(name: String) -> Self {
        Char { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        Char { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Char { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Char>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.name.clone(), input.holding,input.name,input.value)
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

#[derive(Clone,Debug)]
pub struct Tinytext{
    name: String,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Tinytext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Tinytext { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value:Option<String>) -> Self {
        crate::mapping::types::Tinytext { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Tinytext { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Tinytext>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.name.clone(), input.holding,input.name,input.value)
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

#[derive(Clone,Debug)]
pub struct Text{
    name: String,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Text {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Text { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::types::Text { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Text { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Text>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.name.clone(), input.holding,input.name,input.value)
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

#[derive(Clone,Debug)]
pub struct Mediumtext{
    name: String,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Mediumtext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Mediumtext { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::types::Mediumtext { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Mediumtext { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Mediumtext>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.name.clone(), input.holding,input.name,input.value)
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

#[derive(Clone,Debug)]
pub struct Longtext{
    name: String,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Longtext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Longtext { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::types::Longtext { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Longtext { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Longtext>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.name.clone(), input.holding,input.name,input.value)
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

impl Column for crate::mapping::types::Longtext {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Int{
    value: Option<i32>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl Int {
    pub fn with_name(name: String) -> Self {
        Int { name:name, value: None ,holding: Holding::Name, sub_query:None}
    }

    pub fn with_value(value: Option<i32>) -> Self {
        Int { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<i32>) -> Self {
        Int { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<i32> {
        self.value.clone()
    }
}

impl Column for Int {
    fn name(&self) -> String {
        //let type_self: Int = self as Int;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Year{
    value: Option<i32>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Year {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Year { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<i32>) -> Self {
        crate::mapping::types::Year { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<i32>) -> Self {
        Year { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<i32> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Year {
    fn name(&self) -> String {
        //let type_self: Year = self as Year;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Set<T:Clone>{
    value: Option<Vec<T>>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl<T:Clone> Set<T> {
    pub fn with_name(name: String) -> Self {
        Set { name:name, value:None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<Vec<T>>) -> Self {
        Set { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<Vec<T>>) -> Self {
        Set { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<Vec<T>> {
        self.value.clone()
    }
}

impl <T:Clone> Column for Set<T> {
    fn name(&self) -> String {
        //let type_self: Set<T> = self as Set<T>;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Boolean{
    value: Option<bool>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Boolean {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Boolean { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    fn with_value(value: Option<bool>) -> Self {
        crate::mapping::types::Boolean { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<bool>) -> Self {
        crate::mapping::types::Boolean { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<bool> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Boolean {
    fn name(&self) -> String {
        //let type_self: &Tinyint = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Tinyint{
    value: Option<i8>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl Tinyint {
    pub fn with_name(name: String) -> Self {
        Tinyint { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    fn with_value(value: Option<i8>) -> Self {
        Tinyint { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<i8>) -> Self {
        Tinyint { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<i8> {
        self.value.clone()
    }
}

impl Column for Tinyint {
    fn name(&self) -> String {
        //let type_self: &Tinyint = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Smallint{
    value: Option<i16>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Smallint {
    fn with_name(name: String) -> Self {
        crate::mapping::types::Smallint { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    fn with_value(value: Option<i16>) -> Self {
        crate::mapping::types::Smallint { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<i16>) -> Self {
        Smallint { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<i16> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Smallint {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Smallint = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Bigint{
    value: Option<i64>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl Bigint {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Bigint { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<i64>) -> Self {
        crate::mapping::types::Bigint { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<i64>) -> Self {
        Bigint { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<i64> {
        self.value.clone()
    }
}

impl Column for Bigint {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Bigint = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct BigintUnsigned{
    value: Option<u64>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::BigintUnsigned {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::BigintUnsigned { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<u64>) -> Self {
        crate::mapping::types::BigintUnsigned { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<u64>) -> Self {
        BigintUnsigned { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<u64> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::BigintUnsigned {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::BigintUnsigned = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Numeric{
    value: Option<f64>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Numeric {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Numeric { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<f64>) -> Self {
        crate::mapping::types::Numeric { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Numeric { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Numeric {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Numeric = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Float{
    value: Option<f32>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Float {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Float { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    fn with_value(value: Option<f32>) -> Self {
        crate::mapping::types::Float { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<f32>) -> Self {
        Float { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<f32> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Float {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Float = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Double{
    value: Option<f64>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Double {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Double { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    fn with_value(value: Option<f64>) -> Self {
        crate::mapping::types::Double { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Double { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Double {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Double = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Decimal{
    value: Option<f64>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Decimal {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Decimal { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<f64>) -> Self {
        crate::mapping::types::Decimal { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Decimal { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Decimal {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Decimal = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Date{
    value: Option<chrono::NaiveDate>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Date {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Date { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<chrono::NaiveDate>) -> Self {
        crate::mapping::types::Date { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::NaiveDate>) -> Self {
        Date { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<NaiveDate> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Date {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Date = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Time{
    value: Option<chrono::NaiveTime>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Time {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Time { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: chrono::NaiveTime) -> Self {
        crate::mapping::types::Time { value:Some(value), name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }
    pub fn with_name_value(name: String, value: Option<chrono::NaiveTime>) -> Self {
        crate::mapping::types::Time { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<NaiveTime> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Time {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Time = self;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Datetime{
    pub value: Option<chrono::DateTime<Local>>,
    pub name: String,
    sub_query: Option<QueryBuilder>,
    pub holding: Holding
}

impl crate::mapping::types::Datetime {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Datetime { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    fn with_value(value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::types::Datetime { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::types::Datetime { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<DateTime<Local>> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Datetime {
    fn name(&self) -> String {
       // let type_self: &crate::mapping::types::Datetime = self as &crate::mapping::types::Datetime;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Timestamp{
    value: Option<chrono::DateTime<Local>>,
    name: String,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Timestamp {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Timestamp { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::types::Timestamp { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::types::Timestamp { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<DateTime<Local>> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Timestamp {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Timestamp = self as &crate::mapping::types::Timestamp;
        self.name.clone()
    }
}

#[derive(Clone,Debug)]
pub struct Json{
    name: String,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Json {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Json { name:name, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::types::Json { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        crate::mapping::types::Json { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::types::Json>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.name.clone(),input.holding,input.name,input.value)
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

#[derive(Clone,Debug)]
pub struct Blob{
    name: String,
    value: Option<Vec<u8>>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Blob {
    fn with_name(name: String) -> Self {
        crate::mapping::types::Blob { name:name, value: None ,holding: Holding::Name, sub_query:None}
    }

    fn with_value(value: Option<Vec<u8>>) -> Self {
        crate::mapping::types::Blob { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_value(name: String, value: Option<Vec<u8>>) -> Self {
        crate::mapping::types::Blob { name:name, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<Vec<u8>> {
        self.value.clone()
    }
}

impl Column for crate::mapping::types::Blob {
    fn name(&self) -> String {
        //let type_self: &crate::mapping::types::Blob = self as &crate::mapping::types::Blob;
        self.name.clone()
    }
}