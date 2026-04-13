#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Field {
    pub table: &'static str,
    pub name: &'static str,
}

impl Field {
    pub fn new(table: &'static str, name: &'static str) -> Field {
        Field {
            table,
            name,
        }
    }
}
