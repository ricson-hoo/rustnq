use crate::db::base::Holding;
use crate::query::base::Condition;

#[derive(Clone, Copy)]
pub struct Varchar<'a>{
    name: &'a str,
    value: &'a str,
    holding: Holding,
}

impl <'a> Varchar<'a> {
    pub fn value(value: &'a str) -> Self {
        Varchar { value, name:"" ,holding: Holding::Value }
    }

    pub fn name(name: &'static str) -> Self {
        Varchar { name, value: "" ,holding: Holding::Name }
    }

    pub fn equal<T>(&self, input: T) -> Condition
    where
        T: Into<Varchar<'a>>,
    {
        let varchar = input.into();
        let output = match varchar.holding {
            Holding::Name => varchar.name,
            Holding::Value => &format!("'{}'",varchar.value),
            _ => ""
        };
        Condition::new(format!("{} = {}", self.name, output))
    }

    pub fn like(&self, pattern: &'static str) -> Condition
    {
        Condition::new(format!("{} LIKE '{}'", self.name, pattern))
    }
}

pub struct Int{
    value: i32,
    name: &'static str,
    holding: Holding
}
