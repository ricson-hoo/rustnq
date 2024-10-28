use serde::{{Serialize, Deserialize}};

use std::fmt;

#[derive(Serialize,Deserialize,Clone,Debug,Copy)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum ProductCategory {
    服装,
}

impl From<ProductCategory> for String {
    fn from(item: ProductCategory) -> Self {
        match item {
            ProductCategory::服装 => "服装".to_string(),
        }
    }
}

impl From<&str> for ProductCategory {
    fn from(s: &str) -> Self {
        match s {
            "服装" => ProductCategory::服装,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for ProductCategory {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductCategory::服装 => write!(f,"服装"),
        }
    }
}
impl ProductCategory {
    pub fn values() -> Vec<ProductCategory> {
        vec![ProductCategory::服装,]
    }
}
