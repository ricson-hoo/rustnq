use crate::mapping::description::{Holding, Column, MappedEnum, SqlColumn};
use crate::query::builder::{Condition, Field, QueryBuilder, SelectField};
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Serialize,Deserialize};
use std::fmt;
use std::str::FromStr;
use crate::utils::date_sub_unit::DateSubUnit;
use crate::configuration::{encryptor, get_encryptor};

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
    table: Option<String>,
    value: Option<T>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl<T:Clone+Into<String>> Enum<T>{
    pub fn with_name(name: String) -> Self {
        Enum { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    fn with_value(value: Option<T>) -> Self {
        Enum { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<T>) -> Self {
        Enum { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Enum {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<T>) -> Self {
        Enum {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
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

    pub fn is_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NULL", self.qualified_name()))
    }

    pub fn equal(&self, input: T) -> Condition
    {
        Condition::new(format!("{} = '{}'", self.qualified_name(), input.into()))
    }
    pub fn ne(&self, input: T) -> Condition
    {
        Condition::new(format!("{} != '{}'", self.qualified_name(), input.into()))
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
        Condition::new(format!("{} = {}", self.qualified_name(), input.qualified_name()))
    }

    pub fn in_(&self, input_list: Vec<T>) -> Condition
    {
        Condition::new(format!("{} in ({})", self.qualified_name(), input_list.into_iter()
            .map(|input| format!("'{}'", input.into().to_string()))
            .collect::<Vec<String>>()
            .join(" , ")))
    }

    pub fn not_in(&self, input_list: Vec<T>) -> Condition
    {
        Condition::new(format!("{} not in ({})", self.qualified_name(), input_list.into_iter()
            .map(|input| format!("'{}'", input.into().to_string()))
            .collect::<Vec<String>>()
            .join(" , ")))
    }

}

impl <T:Clone+Into<String>> Column for Enum<T>{
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }

    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Varchar {
    table: Option<String>,
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl Varchar {

    /*pub fn and(&self, other:& (impl Column + Clone+ 'static)) -> Vec<String> {
        vec![self.qualified_name().clone(), other.qualified_name()]
    }*/

    pub fn with_name(name: String) -> Self {
        Varchar {table:None, name:name, alias:None, value: None ,holding: Holding::Name, sub_query:None,is_encrypted:false }
    }

    pub fn with_value(value: Option<String>) -> Self {
        Varchar {table:None, value:value, alias:None, name:"".to_string() ,holding: Holding::Value, sub_query:None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Varchar {table:None, name:name, alias:None, value:value, holding: Holding::Value, sub_query:None,is_encrypted:false }
    }

    pub fn with_name_query(name: String, sub_query: Option<QueryBuilder>) -> Self {
        Varchar {table:None, name:name, alias:None, value:Some("".to_string()), holding: Holding::SubQuery, sub_query:sub_query,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Varchar {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<String>) -> Self {
        Varchar {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
       self.alias = Some(alias.to_string());
       self.clone()
    }

    pub fn sub_(&mut self, field: Varchar) -> Self {
       self.name = format!("{} - {}", self.name, field.name);
       self.clone()
    }

    pub fn div<T: std::fmt::Display>(&mut self, value: T) -> Self {
       self.name = format!("{} DIV {}", self.name, value);
       self.clone()
    }

    pub fn le<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} <= ({})", self.qualified_name(), value.to_string()))
    }


    pub fn gt<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} > ({})", self.qualified_name(), value.to_string()))
    }

    pub fn ge<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} >= ({})", self.qualified_name(), value.to_string()))
    }

    pub fn in_<T:Clone+Into<String>>(&self, input_list: Vec<T>) -> Condition
    {
        Condition::new(format!("{} in ({})", self.qualified_name(), input_list.into_iter()
            .map(|input| format!("'{}'", input.into().to_string()))
            .collect::<Vec<String>>()
            .join(" , ")))
    }

    pub fn optional_as(mut self, alias:Option<String>) -> Self {
        if(alias.is_some()){
            self.alias = alias;
        }
        self.clone()
    }

    pub fn holding(&self) -> Holding {
        self.holding.clone()
    }

    pub fn sub_query(&self) -> Option<QueryBuilder> {
        self.sub_query.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Varchar>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.table.clone(), self.name.clone(), self.is_encrypted, input.holding, input.table,input.name,input.value)
        /*let output = match varchar.holding {
            Holding::Name => varchar.name.clone(),
            Holding::Value => format!("'{}'",varchar.value.unwrap().to_string()),
            _ => "".to_string()
        };
        let mut name = varchar.name;
        if self.table.is_some() {
            name = format!("{}.{}",self.table.unwrap(),self.name)
        }
        Condition::new(format!("{} = {}", name, output))*/
    }

    pub fn like(&self, pattern: String) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.qualified_name(), pattern))
    }
    pub fn desc(&self) -> SelectField
    {
        SelectField::Field(Field::new(&*self.table(), &format!("{} desc", &*self.name().to_string()), self.alias(), self.is_encrypted()))
    }
    pub fn is_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NULL", self.qualified_name()))
    }
    pub fn is_not_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NOT NULL", self.qualified_name()))
    }
    pub fn is_not_empty(&self) -> Condition
    {
        Condition::new(format!("{} !=''", self.qualified_name()))
    }
    pub fn is_empty(&self) -> Condition
    {
        Condition::new(format!("{} =''", self.qualified_name()))
    }
    pub fn ne<T>(&self, input: T) -> Condition
    where
        T: Into<Varchar>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.qualified_name(),
            Holding::Value => format!("'{}'",varchar.value.unwrap().to_string()),
            _ => "".to_string()
        };
        Condition::new(format!("{} != {}", self.qualified_name(), output))
    }
}

impl From<i32> for Varchar {
    fn from(s: i32) -> Self {
        Varchar::with_value(Some(s.to_string()))
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
        /*if let Some(enum_value) = a_enum.value.clone() {
            let str_value:String = String::from(enum_value);
            Varchar::with_name_value(a_enum.name(),Some(str_value))
        }else {
            Varchar::with_name(a_enum.name())
        }*/
        Varchar{
            table: a_enum.table,
            name: a_enum.name,
            alias: a_enum.alias,
            value: if let Some(enum_value) = a_enum.value.clone() {Some(String::from(enum_value))} else {None},
            sub_query: a_enum.sub_query,
            holding: a_enum.holding,
            is_encrypted: a_enum.is_encrypted,
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
        /*Varchar::with_name_value(set.name(),Some(string_value))*/
        Varchar{
            table: set.table,
            name: set.name,
            alias: set.alias,
            value: if string_value.is_empty() {None} else {Some(string_value)},
            sub_query: set.sub_query,
            holding: set.holding,
            is_encrypted: set.is_encrypted,
        }
    }
}

impl From<&Int> for Varchar {
    fn from(i: &Int) -> Self {
        //Varchar::with_qualified_name_value(i.table(),i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
        Varchar{
            table: i.table.clone(),
            name: i.name.clone(),
            alias: i.alias.clone(),
            value: i.value().map(|v| v.to_string()),
            sub_query: i.sub_query.clone(),
            holding: i.holding.clone(),
            is_encrypted: i.is_encrypted,
        }
    }
}

impl From<Int> for Varchar {
    fn from(i: Int) -> Self {
        //Varchar::with_qualified_name_value(i.table(),i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
        Varchar{
            table: i.table.clone(),
            name: i.name.clone(),
            alias: i.alias.clone(),
            value: i.value().clone().map(|v| v.to_string()),
            sub_query: i.sub_query.clone(),
            holding: i.holding.clone(),
            is_encrypted: i.is_encrypted,
        }
    }
}

impl From<Longtext> for Varchar  {
    fn from(value: Longtext) -> Self {
        Varchar{
            table: value.table.clone(),
            name: value.name.clone(),
            alias: value.alias.clone(), 
            value: value.value.clone(),
            sub_query: value.sub_query.clone(),
            holding: value.holding.clone(),
            is_encrypted: value.is_encrypted,
        }
    }
}

impl From<&Longtext> for Varchar   {
    fn from(value: &Longtext) -> Self {
        Varchar{
            table: value.table.clone(),
            name: value.name.clone(),
            alias: value.alias.clone(),
            value: value.value.clone(),
            sub_query: value.sub_query.clone(),
            holding: value.holding.clone(),
            is_encrypted: value.is_encrypted, 
        }
    }
}

impl Column for Varchar {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    /*fn qualified_name(&self) -> String {
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
    }*/
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Char{
    table: Option<String>,
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

fn encrypt_value(value:String) -> String {
    let encryptor = get_encryptor();
    encryptor.encrypt(value)
}

fn build_equal_condition_for_string_type(self_table:Option<String>, self_name:String,self_is_encrypted: bool, input_holding:Holding,input_table:Option<String>, input_name:String,input_value:Option<String>) -> Condition {
    let mut self_name = self_name.clone();
    if self_table.is_some() {
        self_name = format!("{}.{}",self_table.unwrap(),self_name);
    }
    let mut input_name = input_name.clone();
    if input_table.is_some() {
        input_name = format!("{}.{}",input_table.unwrap(),input_name);
    }
    let output = match input_holding {
        Holding::Name => format!(" = {}",input_name),
        Holding::Value => match input_value {
            Some(value) => format!(" = {}",if self_is_encrypted {encrypt_value(value)} else {format!("'{}'",value)}),
            None => "is null".to_string()
        },
        _ => "build_equal_condition_for_string_type to do ".to_string() //subquery
    };
    Condition::new(format!("{} {}", self_name, output))
}

impl Char {
    pub fn with_name(name: String) -> Self {
        Char {table:None,  name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None,is_encrypted:false }
    }

    pub fn with_value(value: Option<String>) -> Self {
        Char {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Char {table:None,  name:name, alias: None, value:value, holding: Holding::Value, sub_query:None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Char {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<String>) -> Self {
        Char {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
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
        build_equal_condition_for_string_type(self.table.clone(), self.name.clone(), self.is_encrypted.clone(), input.holding, input.table,input.name,input.value)
    }

    pub fn like(&self, pattern: String) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.qualified_name(), pattern))
    }
}

impl Column for Char {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Tinytext{
    table: Option<String>,
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Tinytext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Tinytext {table:None,  name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None,is_encrypted:false }
    }

    pub fn with_value(value:Option<String>) -> Self {
        crate::mapping::column_types::Tinytext {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Tinytext {table:None,  name:name, alias: None, value:value, holding: Holding::Value, sub_query:None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Tinytext {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<String>) -> Self {
        Tinytext {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::column_types::Tinytext>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.table.clone(),self.name.clone(), self.is_encrypted.clone(), input.holding,input.table,input.name,input.value)
    }

    pub fn like(&self, pattern: String) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.qualified_name(), pattern))
    }
}

impl Column for Tinytext {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Text{
    table: Option<String>,
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Text {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Text {table:None,  name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None,is_encrypted:false }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::column_types::Text {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Text {table:None,  name:name, alias: None, value:value, holding: Holding::Value, sub_query:None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Text {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<String>) -> Self {
        Text {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::column_types::Text>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.table.clone(),self.name.clone(), self.is_encrypted.clone(), input.holding,input.table, input.name,input.value)
    }

    pub fn like(&self, pattern: String) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.qualified_name(), pattern))
    }
}

impl Column for crate::mapping::column_types::Text {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

impl From<Text> for Varchar {
    fn from(v: Text) -> Self {
        Varchar{
            table: v.table,
            name: v.name,
            alias: v.alias,
            value: if let Some(v_value) = v.value.clone() {Some(String::from(v_value))} else {None},
            sub_query: v.sub_query,
            holding: v.holding,
            is_encrypted: v.is_encrypted,
        }
    }
}

#[derive(Clone,Debug)]
pub struct Mediumtext{
    table: Option<String>,
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Mediumtext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Mediumtext {table:None,  name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None,is_encrypted:false }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::column_types::Mediumtext {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Mediumtext {table:None,  name:name, alias: None, value:value, holding: Holding::Value, sub_query:None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Mediumtext {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<String>) -> Self {
        Mediumtext {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::column_types::Mediumtext>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.table.clone(), self.name.clone(), self.is_encrypted.clone(), input.holding,input.table,input.name,input.value)
    }

    pub fn like(&self, pattern: String) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.qualified_name(), pattern))
    }
}

impl Column for crate::mapping::column_types::Mediumtext {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Longtext{
    table: Option<String>,
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Longtext {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Longtext {table:None,  name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None,is_encrypted:false }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::column_types::Longtext {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        Longtext {table:None,  name:name, alias: None, value:value, holding: Holding::Value, sub_query:None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Longtext {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<String>) -> Self {
        Longtext {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::column_types::Longtext>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.table.clone(), self.name.clone(), self.is_encrypted.clone(), input.holding,input.table,input.name,input.value)
    }

    pub fn like(&self, pattern: String) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.qualified_name(), pattern))
    }
}

impl From<Varchar> for crate::mapping::column_types::Longtext {
    fn from(v: Varchar) -> Self {
       Longtext::with_name_value(v.name.clone(), v.value.map(|s| s.to_string())) 
    } 
}

impl From<&Varchar> for crate::mapping::column_types::Longtext {
    fn from(value: &Varchar) -> Self {
        Longtext::with_name_value(value.name.clone(), value.value.as_ref().map(|s| s.to_string()))
    }
}

impl From<String> for crate::mapping::column_types::Longtext  {
    fn from(value: String) -> Self {
        Longtext::with_value(Some(value))
    }
}

impl From<&str> for crate::mapping::column_types::Longtext   {
    fn from(value: &str) -> Self {
        Longtext::with_value(Some(value.to_string()))
    }
}

impl Column for crate::mapping::column_types::Longtext {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Int{
    table: Option<String>,
    value: Option<i32>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl Int {
    pub fn with_name(name: String) -> Self {
        Int {table:None,  name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_value(value: Option<i32>) -> Self {
        Int {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<i32>) -> Self {
        Int {table:None,  name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Int {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<i32>) -> Self {
        Int {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<i32> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn gt<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} > ({})", self.qualified_name(), value.to_string()))
    }

    pub fn lt<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} < ({})", self.qualified_name(), value.to_string()))
    }

    pub fn ge<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} >= ({})", self.qualified_name(), value.to_string()))
    }

    pub fn le<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} <= ({})", self.qualified_name(), value.to_string()))
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Int>,
    {
        let int = input.into();
        let output = match int.holding {
            Holding::Name => int.qualified_name(),
            Holding::Value => format!("'{}'",int.value.unwrap().to_string()),
            _ => "".to_string()
        };
        Condition::new(format!("{} = {}", self.qualified_name(), output))
    }
    pub fn is_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NULL", self.qualified_name()))
    }
    pub fn is_not_null(&self) -> Condition
    {
        Condition::new(format!("{} IS NOT NULL", self.qualified_name()))
    }
    pub fn is_not_empty(&self) -> Condition
    {
        Condition::new(format!("{} !=''", self.qualified_name()))
    }
    pub fn is_empty(&self) -> Condition
    {
        Condition::new(format!("{} =''", self.qualified_name()))
    }
    pub fn holding(&self) -> Holding {
        self.holding.clone()
    }
    pub fn sub_query(&self) -> Option<QueryBuilder> {
        self.sub_query.clone()
    }

    pub fn desc(&self) -> SelectField{
        SelectField::Field(Field::new(&*self.table(), &format!("{} desc", &*self.name().to_string()), self.alias(), self.is_encrypted()))
    }
}

// 为 Int 实现 From<i32>
impl From<i32> for Int {
    fn from(v: i32) -> Self {
        Int::with_value(Some(v))
    }
}

impl From<String> for Int {
    fn from(v: String) -> Self {
        let res = v.parse::<i32>();
        match res {
            Ok(num) => {
                Int::with_value(Some(num))
            }
            Err(err) => {
                Int::with_value(None)
            }
        }
    }
}

impl Column for Int {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

impl From<&Varchar> for Int {
    fn from(v: &Varchar) -> Self {
        if v.table.is_none() {
            match v.holding {
                Holding::Name => {Int::with_name(v.name.clone())}
                _ => {Int::with_name_value(v.name.clone(), v.value().map(|s| i32::from_str(&s).ok()).flatten())}
            }
        }else {
            match v.holding {
                Holding::Name => {Int::with_qualified_name(v.table.clone().unwrap(), v.name.clone())}
                _ => {Int::with_qualified_name_value(v.table.clone().unwrap(), v.name.clone(), v.value().map(|s| i32::from_str(&s).ok()).flatten())}
            }
        }
    }
}

impl From<Varchar> for Int {
    fn from(v: Varchar) -> Self {
        if v.table.is_none() {
            match v.holding {
                Holding::Name => {Int::with_name(v.name.clone())}
                _ => {Int::with_name_value(v.name.clone(), v.value().map(|s| i32::from_str(&s).ok()).flatten())}
            }
        }else {
            match v.holding {
                Holding::Name => {Int::with_qualified_name(v.table.clone().unwrap(), v.name.clone())}
                _ => {Int::with_qualified_name_value(v.table.clone().unwrap(), v.name.clone(), v.value().map(|s| i32::from_str(&s).ok()).flatten())}
            }
        }
    }
}

#[derive(Clone,Debug)]
pub struct Year{
    table: Option<String>,
    value: Option<i32>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Year {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Year {table:None,  name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_value(value: Option<i32>) -> Self {
        crate::mapping::column_types::Year {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<i32>) -> Self {
        Year {table:None,  name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Year {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<i32>) -> Self {
        Year {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<i32> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::Year {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Set<T:Clone+Into<String>>{
    table: Option<String>,
    value: Option<Vec<T>>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl<T:Clone+Into<String>> Set<T> {
    pub fn with_name(name: String) -> Self {
        Set {table:None,  name:name, value:None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_value(value: Option<Vec<T>>) -> Self {
        Set {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<Vec<T>>) -> Self {
        Set {table:None,  name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Set {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<Vec<T>>) -> Self {
        Set {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<Vec<T>> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn find_in_set(&mut self, value:String) -> Condition {
        let mut self_name = self.name.clone();
        if self.table.is_some() {
            self_name = format!("{}.{}",self.table.clone().unwrap(),self_name);
        }
        Condition::new(format!("FIND_IN_SET('{}', {}) > 0", value, self_name))
    }

    pub fn value_as_string(&self) -> Option<String> {
        match &self.value {
            Some(value) => Some(value.iter().map(|val| val.clone().into()).collect::<Vec<String>>().join(",")),
            None => None
        }
    }
}

impl <T:Clone+Into<String>> Column for Set<T> {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Boolean{
    table: Option<String>,
    value: Option<bool>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Boolean {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Boolean {table:None,  name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    fn with_value(value: Option<bool>) -> Self {
        crate::mapping::column_types::Boolean {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<bool>) -> Self {
        crate::mapping::column_types::Boolean {table:None,  name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Boolean {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<bool>) -> Self {
        Boolean {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<bool> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn holding(&self) -> Holding {
        self.holding.clone()
    }

    pub fn sub_query(&self) -> Option<QueryBuilder> {
        self.sub_query.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Boolean>,
    {
        let tinyint = input.into();
        let output = match tinyint.holding {
            Holding::Name => {tinyint.qualified_name()}
            Holding::Value => {format!("{}", if tinyint.value.unwrap_or_default() {"1"} else {"0"})}
            _=> "".to_string()
        };
        Condition::new(format!("{} = {}", self.qualified_name(), output))
    }
}

impl From<bool> for Boolean {
    fn from(v: bool) -> Self {
        Boolean::with_value(Some(v))
    }
}

impl From<Boolean> for Varchar {
    fn from(v: Boolean) -> Self {
        Varchar{
            table: v.table,
            name: v.name,
            alias: v.alias,
            value: if let Some(v_value) = v.value.clone() {Some(v_value.to_string())} else {None},
            sub_query: v.sub_query,
            holding: v.holding,
            is_encrypted: v.is_encrypted,
        }
    }
}

impl Column for crate::mapping::column_types::Boolean {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Tinyint{
    table: Option<String>,
    value: Option<i8>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl Tinyint {
    pub fn with_name(name: String) -> Self {
        Tinyint {table:None,  name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_value(value: Option<i8>) -> Self {
        Tinyint {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<i8>) -> Self {
        Tinyint {table:None,  name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Tinyint {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<i8>) -> Self {
        Tinyint {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<i8> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
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
            Holding::Name => {tinyint.qualified_name()}
            Holding::Value => {format!("{}", tinyint.value.unwrap_or_default())}
            _=> "".to_string()
        };
        Condition::new(format!("{} = {}", self.qualified_name(), output))
    }
}

impl Column for Tinyint {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

impl From<Tinyint> for Varchar {
    fn from(v: Tinyint) -> Self {
        Varchar{
            table: v.table,
            name: v.name,
            alias: v.alias,
            value: if let Some(v_value) = v.value.clone() {Some(v_value.to_string())} else {None},
            sub_query: v.sub_query,
            holding: v.holding,
            is_encrypted: v.is_encrypted,
        }
    }
}
impl From<bool> for Tinyint {
    fn from(v: bool) -> Self {
        Tinyint::with_value(if v { Some(1) } else { Some(0) })
    }
}
impl From<i8> for Tinyint {
    fn from(v: i8) -> Self {
        Tinyint::with_value(Some(v))
    }
}

#[derive(Clone,Debug)]
pub struct Smallint{
    table: Option<String>,
    value: Option<i16>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Smallint {
    fn with_name(name: String) -> Self {
        crate::mapping::column_types::Smallint {table:None,  name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    fn with_value(value: Option<i16>) -> Self {
        crate::mapping::column_types::Smallint {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_name_value(name: String, value: Option<i16>) -> Self {
        Smallint {table:None,  name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Smallint {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<i16>) -> Self {
        Smallint {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<i16> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::Smallint {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Bigint{
    table: Option<String>,
    value: Option<i64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl Bigint {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Bigint { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_value(value: Option<i64>) -> Self {
        crate::mapping::column_types::Bigint { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<i64>) -> Self {
        Bigint { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Bigint {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<i64>) -> Self {
        Bigint {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<i64> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for Bigint {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
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
    table: Option<String>,
    value: Option<u64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::BigintUnsigned {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::BigintUnsigned { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_value(value: Option<u64>) -> Self {
        crate::mapping::column_types::BigintUnsigned { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<u64>) -> Self {
        BigintUnsigned { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        BigintUnsigned {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<u64>) -> Self {
        BigintUnsigned {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<u64> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::BigintUnsigned {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Numeric{
    table: Option<String>,
    value: Option<f64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Numeric {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Numeric { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_value(value: Option<f64>) -> Self {
        crate::mapping::column_types::Numeric { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Numeric { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Numeric {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<f64>) -> Self {
        Numeric {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::Numeric {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Float{
    table: Option<String>,
    value: Option<f32>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Float {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Float { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    fn with_value(value: Option<f32>) -> Self {
        crate::mapping::column_types::Float { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<f32>) -> Self {
        Float { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Float {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<f32>) -> Self {
        Float {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<f32> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::Float {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Double{
    table: Option<String>,
    value: Option<f64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Double {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Double {table:None, name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted: false }
    }

    fn with_value(value: Option<f64>) -> Self {
        crate::mapping::column_types::Double {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false}
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Double {table:None,  name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted: false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Double {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<f64>) -> Self {
        Double {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::Double {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Decimal{
    table: Option<String>,
    value: Option<f64>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Decimal {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Decimal { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_value(value: Option<f64>) -> Self {
        crate::mapping::column_types::Decimal { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<f64>) -> Self {
        Decimal { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Decimal {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<f64>) -> Self {
        Decimal {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<f64> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::Decimal {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

// 为Decimal类型添加比较方法
impl crate::mapping::column_types::Decimal {
    pub fn ge<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} >= ({})", self.qualified_name(), value.to_string()))
    }

    pub fn gt<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} > ({})", self.qualified_name(), value.to_string()))
    }

    pub fn lt<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} < ({})", self.qualified_name(), value.to_string()))
    }

    pub fn le<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} <= ({})", self.qualified_name(), value.to_string()))
    }
}


impl From<Decimal> for Varchar {
    fn from(v: Decimal) -> Self {
        Varchar{
            table: v.table,
            name: v.name,
            alias: v.alias,
            value: if let Some(v_value) = v.value.clone() {Some(v_value.to_string())} else {None},
            sub_query: v.sub_query,
            holding: v.holding,
            is_encrypted: v.is_encrypted,
        }
    }
}

#[derive(Clone,Debug)]
pub struct Date{
    table: Option<String>,
    value: Option<chrono::NaiveDate>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Date {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Date { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_value(value: Option<chrono::NaiveDate>) -> Self {
        crate::mapping::column_types::Date { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::NaiveDate>) -> Self {
        Date { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Date {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<NaiveDate>) -> Self {
        Date {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<NaiveDate> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
    pub fn desc(&self) -> SelectField{
        SelectField::Field(Field::new(&*self.table(), &format!("{} desc", &*self.name().to_string()), self.alias(), self.is_encrypted()))
    }
    pub fn add(&mut self, value: i32, unit: DateSubUnit) -> Self {
        self.name = format!("DATE_ADD ({}, INTERVAL {} {})", self.name, value, unit);
        self.table = None;
        self.clone()
    }

    pub fn ge<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} >= '{}'", self.qualified_name(), value.to_string()))
    }

    pub fn equal<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} = '{}'", self.qualified_name(), value.to_string()))
    }

    pub fn gt<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} > ({})", self.qualified_name(), value.to_string()))
    }

    pub fn lt<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} < ({})", self.qualified_name(), value.to_string()))
    }

    pub fn lt_<T: Into<SelectField>>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} < {}", self.qualified_name(), value.into().to_string()))
    }

    pub fn le<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} <= '{}'", self.qualified_name(), value.to_string()))
    }

    pub fn between(&self, date1: chrono::NaiveDate,date2: chrono::NaiveDate) -> Condition {
        Condition::new(format!("{} BETWEEN '{}' AND '{}'", self.qualified_name(), date1.format("%Y-%m-%d").to_string(), date2.format("%Y-%m-%d").to_string()))
    }
}

impl From<&Date> for Varchar {
    fn from(i: &Date) -> Self {
        //Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
        Varchar{
            table: i.table.clone(),
            name: i.name.clone(),
            alias: i.alias.clone(),
            value: i.value().clone().map(|v| v.to_string()),
            sub_query: i.sub_query.clone(),
            holding: i.holding.clone(),
            is_encrypted: i.is_encrypted,
        }
    }
}

impl From<Date> for Varchar {
    fn from(i: Date) -> Self {
        //Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
        Varchar{
            table: i.table.clone(),
            name: i.name.clone(),
            alias: i.alias.clone(),
            value: i.value().clone().map(|v| v.to_string()),
            sub_query: i.sub_query.clone(),
            holding: i.holding.clone(),
            is_encrypted: i.is_encrypted,
        }
    }
}

impl From<Timestamp> for Date {
    fn from(i: Timestamp) -> Self {
        //Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
        Date{
            table: i.table.clone(),
            name: i.name.clone(),
            alias: i.alias.clone(),
            value: i.value().clone().map(|v| v.date_naive()),
            sub_query: i.sub_query.clone(),
            holding: i.holding.clone(),
            is_encrypted: i.is_encrypted,
        }
    }
}

impl Column for crate::mapping::column_types::Date {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Time{
    table: Option<String>,
    value: Option<chrono::NaiveTime>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Time {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Time { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_value(value: chrono::NaiveTime) -> Self {
        crate::mapping::column_types::Time { value:Some(value), name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }
    pub fn with_name_value(name: String, value: Option<chrono::NaiveTime>) -> Self {
        crate::mapping::column_types::Time { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }


    pub fn with_qualified_name(table:String, name: String) -> Self {
        Time {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<NaiveTime>) -> Self {
        Time {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<NaiveTime> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::Time {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Datetime{
    table: Option<String>,
    pub value: Option<chrono::DateTime<Local>>,
    pub name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    pub holding: Holding,
    is_encrypted:bool
}


impl crate::mapping::column_types::Datetime {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Datetime { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    fn with_value(value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::column_types::Datetime { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::column_types::Datetime { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Datetime {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<DateTime<Local>>) -> Self {
        Datetime {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<DateTime<Local>> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn desc(&self) -> SelectField{
        SelectField::Field(Field::new(&*self.table(), &format!("{} desc", &*self.name().to_string()), self.alias(), self.is_encrypted()))
    }

    pub fn gt<T: ToString>(&self, value: T) -> Condition
    {
        Condition::new(format!("{} > ({})", self.qualified_name(), value.to_string()))
    }
    pub fn holding(&self) -> Holding {
        self.holding.clone()
    }
    pub fn sub_query(&self) -> Option<QueryBuilder> {
        self.sub_query.clone()
    }
}

impl Column for crate::mapping::column_types::Datetime {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Timestamp{
    table: Option<String>,
    value: Option<chrono::DateTime<Local>>,
    name: String,
    alias:Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Timestamp {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Timestamp { name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_value(value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::column_types::Timestamp { value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_name_value(name: String, value: Option<chrono::DateTime<Local>>) -> Self {
        crate::mapping::column_types::Timestamp { name:name, value:value, holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false,table:None }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Timestamp {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<DateTime<Local>>) -> Self {
        Timestamp {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<DateTime<Local>> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn desc(&self) -> SelectField{
        SelectField::Field(Field::new(&*self.table(), &format!("{} desc", &*self.name().to_string()), self.alias(), self.is_encrypted()))
    }
}

impl From<Timestamp> for Varchar {
    fn from(i: Timestamp) -> Self {
        //Varchar::with_name_value(i.name.clone(),i.value().map(|v| v.to_string())).optional_as(i.alias.clone())
        Varchar{
            table: i.table.clone(),
            name: i.name.clone(),
            alias: i.alias.clone(),
            value: i.value().clone().map(|v| v.to_string()),
            sub_query: i.sub_query.clone(),
            holding: i.holding.clone(),
            is_encrypted: i.is_encrypted,
        }
    }
}

impl Column for crate::mapping::column_types::Timestamp {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Json{
    table: Option<String>,
    name: String,
    alias:Option<String>,
    value: Option<String>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Json {
    pub fn with_name(name: String) -> Self {
        crate::mapping::column_types::Json {table:None,  name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None, is_encrypted:false }
    }

    pub fn with_value(value: Option<String>) -> Self {
        crate::mapping::column_types::Json {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false}
    }

    pub fn with_name_value(name: String, value: Option<String>) -> Self {
        crate::mapping::column_types::Json {table:None,  name:name, alias: None, value:value, holding: Holding::Value, sub_query:None, is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Json {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<String>) -> Self {
        Json {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }

    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<crate::mapping::column_types::Json>,
    {
        let input = input.into();
        build_equal_condition_for_string_type(self.table.clone(), self.name.clone(), self.is_encrypted.clone(), input.holding,input.table, input.name,input.value)
    }

    pub fn like(&self, pattern: String) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.qualified_name(), pattern))
    }
}

impl Column for crate::mapping::column_types::Json {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}

#[derive(Clone,Debug)]
pub struct Blob{
    table: Option<String>,
    name: String,
    alias:Option<String>,
    value: Option<Vec<u8>>,
    sub_query: Option<QueryBuilder>,
    holding: Holding,
    is_encrypted:bool
}

impl crate::mapping::column_types::Blob {
    fn with_name(name: String) -> Self {
        crate::mapping::column_types::Blob {table:None,  name:name, alias: None, value: None ,holding: Holding::Name, sub_query:None,is_encrypted:false }
    }

    fn with_value(value: Option<Vec<u8>>) -> Self {
        crate::mapping::column_types::Blob {table:None,  value:value, name:"".to_string() ,holding: Holding::Value, sub_query:None, alias: None, is_encrypted:false}
    }

    pub fn with_name_value(name: String, value: Option<Vec<u8>>) -> Self {
        crate::mapping::column_types::Blob {table:None,  name:name, alias: None, value:value, holding: Holding::Value, sub_query:None, is_encrypted:false }
    }

    pub fn with_qualified_name(table:String, name: String) -> Self {
        Blob {table:Some(table), name:name, value: None ,holding: Holding::Name, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn with_qualified_name_value(table:String, name: String, value: Option<Vec<u8>>) -> Self {
        Blob {table:Some(table), name:name, value:value, holding: Holding::Value, sub_query:None, alias: None,is_encrypted:false }
    }

    pub fn value(&self) -> Option<Vec<u8>> {
        self.value.clone()
    }

    pub fn set_encrypted(mut self, is_encrypted:bool) -> Self {
        self.is_encrypted = is_encrypted;
        self
    }
    pub fn as_(&mut self, alias:&str) -> Self {
        self.alias = Some(alias.to_string());
        self.clone()
    }
}

impl Column for crate::mapping::column_types::Blob {
    fn table(&self) -> String {
        self.table.clone().unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn alias(&self) -> Option<String> {
        self.alias.clone()
    }

    fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }
    fn qualified_name(&self) -> String {
        if self.table.is_some() {format!("{}.{}",self.table.clone().unwrap(),self.name.clone())} else {self.name.clone()}
    }
}