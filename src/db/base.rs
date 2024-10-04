#[derive(Clone, Copy)]
pub enum Holding{
    Name,Value,Full
}

pub trait Selectable<'a> {
    fn name(&self) -> &'a str;
}

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