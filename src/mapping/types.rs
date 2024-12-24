use crate::mapping::description::{Holding, Column, MappedEnum, SqlColumn};
use crate::query::builder::{Condition, QueryBuilder};
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Serialize,Deserialize};
use std::str::FromStr;

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
pub struct Enum<T:Clone+Into<String>> {
    value: Option<T>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl<T:Clone+Into<String>> Enum<T>{
    pub fn with_name(name: String) -> Self {
        Enum { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    fn with_value(value: Option<T>) -> Self {
        Enum { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<T>) -> Self {
        Enum { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<T> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn value_as_string(&self) -> Option<String> {
        match &self.value {
            Some(value) => Some(value.clone().into()),
            None => None
        }
    }

    pub fn equal(&self, input: T) -> Condition
    /*where
        T: Into<Varchar>,*/
    {
        /*let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value.unwrap().to_string()),
            _ => "".to_string()
        };*/
        Condition::new(format!("{} = '{}'", self.name, input.into()))
    }

    pub fn equals(&self, input: Enum<T>) -> Condition
    /*where
        T: Into<Varchar>,*/
    {
        /*let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value.unwrap().to_string()),
            _ => "".to_string()
        };*/
        Condition::new(format!("{} = {}", self.name, input.name))
    }

    pub fn in_(&self, input_list: Vec<T>) -> Condition
    {
        Condition::new(format!("{} in ({})", self.name, input_list.into_iter()
            .map(|input| format!("'{}'", input.into().to_string()))
            .collect::<Vec<String>>()
            .join(" , ")))
    }

}

impl <T:Clone> Column for Enum<T> where String: From<T>{ //impl <T:MappedEnum> Column for Enum<T>
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(),self.alias.clone().unwrap())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Varchar {
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl Varchar {

    pub fn and(&self, other:& (impl Column + Clone+ 'static)) -> Vec<String> {
        vec![self.name.clone(), other.name()]
    }

    pub fn with_name(name: String) -> Self {
        Varchar { name:name, alias:None, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        Varchar { value:value, alias:None, name:"".to_string() ,holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Varchar { name:name, alias:None, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn with_name_query(name: String, sub_query: Option<QueryBuilder>) -> Self {
        Varchar { name:name, alias:None, value:Some("".to_string()), holding: Holding::SubQuery, sub_query:sub_query }
    }

    pub fn as_(&mut self, alias:&str) -> Self {
       self.alias = Some(alias.to_string());
       self.clone()
    }

    pub fn optional_as(mut self, alias:Option<String>) -> Self {
        if(alias.is_some()){
            self.alias = alias;
        }
        self.clone()
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
    pub fn is_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NULL", self.name))
    }
    pub fn is_not_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NOT NULL", self.name))
    }
    pub fn is_not_empty(&self) -> Condition
    {
        Condition::new(format!("{} !=''", self.name))
    }
    pub fn is_empty(&self) -> Condition
    {
        Condition::new(format!("{} =''", self.name))
    }
    pub fn ne<T>(&self, input: T) -> Condition
    where
        T: Into<Varchar>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => format!("'{}'",varchar.value.unwrap().to_string()),
            _ => "".to_string()
        };
        Condition::new(format!("{} != {}", self.name, output))
    }
}


impl From<&str> for Varchar {
    fn from(s: &str) -> Self {
        Varchar::with_value(Some(s.to_string()))
    }
}

impl From<String> for Varchar {
    fn from(s: String) -> Self {
        Varchar::with_value(Some(s))
    }
}

impl From<Varchar> for String{
    fn from(value: Varchar) -> String {
        value.name().clone()
    }
}

impl From<&Varchar> for String{
    fn from(value: &Varchar) -> String {
        value.name().clone()
    }
}

impl<T:Clone> From<Enum<T>> for Varchar where std::string::String: From<T> {
    fn from(a_enum: Enum<T>) -> Varchar {
        if let Some(enum_value) = a_enum.value.clone() {
            let str_value:String = String::from(enum_value);
            Varchar::with_name_value(a_enum.name(),Some(str_value))
        }else {
            Varchar::with_name(a_enum.name())
        }
    }
}

impl<T> From<Set<T>> for Varchar where  std::string::String: From<T>,T:Clone {
    fn from(set: Set<T>) -> Varchar {
        let mut str_set:Vec<String> = vec![];
        let set_value = set.value.clone();
        if let Some(set_value) = set_value {
            for a_enum in set_value{
                str_set.push(String::from(a_enum))
            }
        }
        let string_value: String = str_set.join(",");
        Varchar::with_name_value(set.name(),Some(string_value))
    }
}

impl From<&Int> for Varchar {
    fn from(i: &Int) -> Self {
        Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
    }
}

impl From<Int> for Varchar {
    fn from(i: Int) -> Self {
        Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
    }
}

impl Column for Varchar {
    fn name(&self) -> String {
        match self.holding {
            Holding::SubQuery=> {
                if self.sub_query.is_some() {
                    if let Ok(query) = self.sub_query.clone().unwrap().build() {
                        format!("({}) as {}",query, self.name.clone())
                    }else {
                        format!("'' as {}",self.name.clone())
                    }
                } else {
                    format!("'' as {}",self.name.clone())
                }
            },
            _ => {
                if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap())} else {self.name.clone()}
            }
        }
    }
}

#[derive(Clone,Debug)]
pub struct Char{
    name: String,
    alias:Option<String>,
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
        Char { name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        Char { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Char { name:name, alias: None, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
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
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Tinytext{
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Tinytext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Tinytext { name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value:Option<String>) -> Self {
        crate::mapping::types::Tinytext { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Tinytext { name:name, alias: None, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
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
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Text{
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Text {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Text { name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::types::Text { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Text { name:name, alias: None, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
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
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Mediumtext{
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Mediumtext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Mediumtext { name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::types::Mediumtext { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Mediumtext { name:name, alias: None, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
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
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Longtext{
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Longtext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Longtext { name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::types::Longtext { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Longtext { name:name, alias: None, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
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
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Int{
    value: Option<i32>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl Int {
    pub fn with_name(name: String) -> Self {
        Int { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<i32>) -> Self {
        Int { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<i32>) -> Self {
        Int { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<i32> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Int>,
    {
        let int = input.into();
        let output = match int.holding {
            Holding::Name => int.name,
            Holding::Value => format!("'{}'",int.value.unwrap().to_string()),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }
    pub fn is_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NULL", self.name))
    }
    pub fn is_not_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NOT NULL", self.name))
    }
    pub fn is_not_empty(&self) -> Condition
    {
        Condition::new(format!("{} !=''", self.name))
    }
    pub fn is_empty(&self) -> Condition
    {
        Condition::new(format!("{} =''", self.name))
    }
}

// 为 Int 实现 From<i32>
impl From<i32> for Int {
    fn from(v: i32) -> Self {
        Int::with_value(Some(v))
    }
}

impl Column for Int {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

impl From<&Varchar> for Int {
    fn from(v: &Varchar) -> Self {
        Int::with_name_value(v.name.clone(),v.value().map(|s| i32::from_str(&s).ok()).flatten())
    }
}

impl From<Varchar> for Int {
    fn from(v: Varchar) -> Self {
        Int::with_name_value(v.name.clone(),v.value().map(|s| i32::from_str(&s).ok()).flatten())
    }
}

#[derive(Clone,Debug)]
pub struct Year{
    value: Option<i32>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Year {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Year { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<i32>) -> Self {
        crate::mapping::types::Year { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<i32>) -> Self {
        Year { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<i32> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Year {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Set<T:Clone+Into<String>>{
    value: Option<Vec<T>>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl<T:Clone+Into<String>> Set<T> {
    pub fn with_name(name: String) -> Self {
        Set { name:name, value:None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<Vec<T>>) -> Self {
        Set { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<Vec<T>>) -> Self {
        Set { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<Vec<T>> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn value_as_string(&self) -> Option<String> {
        match &self.value {
            Some(value) => Some(value.iter().map(|val| val.clone().into()).collect::<Vec<String>>().join(",")),
            None => None
        }
    }
}

impl <T:Clone+Into<String>> Column for Set<T> {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Boolean{
    value: Option<bool>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Boolean {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Boolean { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    fn with_value(value: Option<bool>) -> Self {
        crate::mapping::types::Boolean { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<bool>) -> Self {
        crate::mapping::types::Boolean { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<bool> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Boolean {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Tinyint{
    value: Option<i8>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl Tinyint {
    pub fn with_name(name: String) -> Self {
        Tinyint { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<i8>) -> Self {
        Tinyint { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<i8>) -> Self {
        Tinyint { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<i8> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Tinyint>,
    {
        let tinyint = input.into();
        let output = match tinyint.holding {
            Holding::Name => {tinyint.name}
            Holding::Value => {format!("{}", tinyint.value.unwrap_or_default())}
            _=> "".to_string()
        };
        Condition::new(format!("{} = {}", self.name, output))
    }
}

impl Column for Tinyint {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Smallint{
    value: Option<i16>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Smallint {
    fn with_name(name: String) -> Self {
        crate::mapping::types::Smallint { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    fn with_value(value: Option<i16>) -> Self {
        crate::mapping::types::Smallint { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<i16>) -> Self {
        Smallint { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<i16> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Smallint {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Bigint{
    value: Option<i64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl Bigint {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Bigint { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<i64>) -> Self {
        crate::mapping::types::Bigint { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<i64>) -> Self {
        Bigint { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<i64> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for Bigint {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

impl From<&str> for Bigint {
    fn from(s: &str) -> Self {
        match i64::from_str(s) {
            Ok(num) => Bigint::with_value(Some(num)), // Convert string to i64
            Err(_) => Bigint::with_value(None), // Handle conversion error
        }
    }
}

impl From<String> for Bigint {
    fn from(s: String) -> Self {
        match i64::from_str(&s) {
            Ok(num) => Bigint::with_value(Some(num)), // Convert string to i64
            Err(_) => Bigint::with_value(None), // Handle conversion error
        }
    }
}

impl From<Bigint> for String{
    fn from(value: Bigint) -> String {
        value.name().clone()
    }
}

impl From<&Bigint> for String{
    fn from(value: &Bigint) -> String {
        value.name().clone()
    }
}

#[derive(Clone,Debug)]
pub struct BigintUnsigned{
    value: Option<u64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::BigintUnsigned {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::BigintUnsigned { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<u64>) -> Self {
        crate::mapping::types::BigintUnsigned { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<u64>) -> Self {
        BigintUnsigned { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<u64> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::BigintUnsigned {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Numeric{
    value: Option<f64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Numeric {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Numeric { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<f64>) -> Self {
        crate::mapping::types::Numeric { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Numeric { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Numeric {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Float{
    value: Option<f32>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Float {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Float { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    fn with_value(value: Option<f32>) -> Self {
        crate::mapping::types::Float { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<f32>) -> Self {
        Float { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<f32> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Float {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Double{
    value: Option<f64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Double {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Double { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    fn with_value(value: Option<f64>) -> Self {
        crate::mapping::types::Double { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Double { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Double {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Decimal{
    value: Option<f64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Decimal {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Decimal { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<f64>) -> Self {
        crate::mapping::types::Decimal { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Decimal { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Decimal {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Date{
    value: Option<chrono::NaiveDate>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Date {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Date { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<chrono::NaiveDate>) -> Self {
        crate::mapping::types::Date { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::NaiveDate>) -> Self {
        Date { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<NaiveDate> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn between(&self, date1: chrono::NaiveDate,date2: chrono::NaiveDate) -> Condition {
        Condition::new(format!("{} BETWEEN '{}' AND '{}'", self.name, date1.format("%Y-%m-%d").to_string(), date2.format("%Y-%m-%d").to_string()))
    }
}

impl From<&Date> for Varchar {
    fn from(i: &Date) -> Self {
        Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
    }
}

impl From<Date> for Varchar {
    fn from(i: Date) -> Self {
        Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
    }
}

impl Column for crate::mapping::types::Date {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Time{
    value: Option<chrono::NaiveTime>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Time {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Time { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: chrono::NaiveTime) -> Self {
        crate::mapping::types::Time { value:Some(value), name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }
    pub fn with_name_value(name: String, value: Option<chrono::NaiveTime>) -> Self {
        crate::mapping::types::Time { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<NaiveTime> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Time {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Datetime{
    pub value: Option<chrono::DateTime<Local>>,
    pub name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    pub holding: Holding
}

impl crate::mapping::types::Datetime {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Datetime { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    fn with_value(value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::types::Datetime { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::types::Datetime { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<DateTime<Local>> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Datetime {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Timestamp{
    value: Option<chrono::DateTime<Local>>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding
}

impl crate::mapping::types::Timestamp {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Timestamp { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None }
    }

    pub fn with_value(value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::types::Timestamp { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::types::Timestamp { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn value(&self) -> Option<DateTime<Local>> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl From<Timestamp> for Varchar {
    fn from(i: Timestamp) -> Self {
        Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string()))
    }
}

impl Column for crate::mapping::types::Timestamp {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Json{
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Json {
    pub fn with_name(name: String) -> Self {
        crate::mapping::types::Json { name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::types::Json { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        crate::mapping::types::Json { name:name, alias: None, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
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
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Blob{
    name: String,
    alias:Option<String>,
    value: Option<Vec<u8>>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
}

impl crate::mapping::types::Blob {
    fn with_name(name: String) -> Self {
        crate::mapping::types::Blob { name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None}
    }

    fn with_value(value: Option<Vec<u8>>) -> Self {
        crate::mapping::types::Blob { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None }
    }

    pub fn with_name_value(name: String, value: Option<Vec<u8>>) -> Self {
        crate::mapping::types::Blob { name:name, alias: None, value:value, holding: Holding::Value, sub_query:None }
    }

    pub fn value(&self) -> Option<Vec<u8>> {
        self.value.clone()
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::types::Blob {
    fn name(&self) -> String {
        if self.alias.is_some() {format!("{} as {}",self.name.clone(), self.alias.clone().unwrap().clone())} else {self.name.clone()}
    }
}