use crate::mapping::description::{Holding, Column, MappedEnum};
use crate::query::builder::Condition;
use chrono::Local;


#[derive(Debug)]
pub struct Table<'a>{
    pub name: &'a str,
    pub comment: &'a str
}

impl<'a> Table<'a> {
    pub fn new(name: &'a str, comment: &'a str) -> Table<'a> {
        Table { name, comment }
    }
}

pub struct Varchar{
    name: String,
    value: String,
    holding: Holding,
}

impl Varchar {
    fn value(value: String) -> Self {
        Varchar { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Varchar>,
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

impl Column for Varchar {
    fn name(&self) -> &str {
        let type_self: &Varchar = self as &Varchar;
        &type_self.name
    }

    fn new(name: String) -> Self {
        Varchar { name:name, value: "".to_string() ,holding: Holding::Name }
    }
}

/*impl <'a> Column<'a> for Varchar<'a> {
    fn name(&self) -> &'a str {
        todo!()
    }

    fn value(&self) -> &'a str {
        todo!()
    }
}*/

pub struct Int{
    value: i32,
    name: String,
    holding: Holding
}

impl Int {
    fn value(value: i32) -> Self {
        Int { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

pub struct Enum<T:MappedEnum> {
    value: Option<T>,
    name: String,
    holding: Holding
}

impl<T:MappedEnum> Enum<T> {
    fn value(value: T) -> Self {
        Enum { value:Some(value), name:"".to_string() ,holding: Holding::Value }
    }
}



pub struct Set<T>{
    value: Vec<T>,
    name: String,
    holding: Holding
}

impl<T> Set<T> {
    fn value(value: Vec<T>) -> Self {
        Set { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

pub struct DateTime{
    value: chrono::DateTime<Local>,
    name: String,
    holding: Holding
}

impl DateTime {
    fn value(value: chrono::DateTime<Local>) -> Self {
        DateTime { value:value, name:"".to_string() ,holding: Holding::Value }
    }
}

impl Column for Int {
    fn name(&self) -> &str {
        let type_self: &Int = self as &Int;
        &type_self.name
    }

    fn new(name: String) -> Self {
        Int { name:name, value: 0 ,holding: Holding::Name }
    }
}

impl <T> Column for Set<T> {
    fn name(&self) -> &str {
        //let type_self: Set<T> = self as Set<T>;
        &self.name
    }

    fn new(name: String) -> Self {
        Set { name:name, value:vec![] ,holding: Holding::Name }
    }
}

impl <T:MappedEnum> Column for Enum<T> {
    fn name(&self) -> &str {
        //let type_self: &Set = self as &Set;
        &self.name
    }

    fn new(name: String) -> Self {
        Enum { name:name, value: None ,holding: Holding::Name }
    }
}

impl Column for DateTime {
    fn name(&self) -> &str {
        let type_self: &DateTime = self as &DateTime;
        &type_self.name
    }

    fn new(name: String) -> Self {
        DateTime { name:name, value: Local::now() ,holding: Holding::Name }
    }
}


