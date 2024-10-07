#[derive(Clone, Copy)]
pub enum Holding{
    Name,Value,Full
}

pub trait Selectable<'a> {
    fn name(&self) -> &'a str;
}

