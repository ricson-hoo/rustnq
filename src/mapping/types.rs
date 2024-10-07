use crate::mapping::description::{Holding, Selectable};
use crate::query::builder::Condition;

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
    pub fn value(value: String) -> Self {
        Varchar { value:value, name:"".to_string() ,holding: Holding::Value }
    }

    pub fn name(name: String) -> Self {
        Varchar { name:name, value: "".to_string() ,holding: Holding::Name }
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

impl Selectable for Varchar {
    fn name(&self) -> &str {
        let type_self: &Varchar = self as &Varchar;
        &type_self.name
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
    name: &'static str,
    holding: Holding
}

impl Selectable for Int {
    fn name(&self) -> &str {
        let type_self: &Int = self as &Int;
        &type_self.name
    }
}

