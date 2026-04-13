use serde::{{Serialize, Deserialize}};

use std::fmt;

#[derive(Serialize,Deserialize,Clone,Debug,Copy)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum ProductStatus {
    正常,
    已删除,
}

impl From<ProductStatus> for String {
    fn from(item: ProductStatus) -> Self {
        match item {
            ProductStatus::正常 => "正常".to_string(),
            ProductStatus::已删除 => "已删除".to_string(),
        }
    }
}

impl From<&str> for ProductStatus {
    fn from(s: &str) -> Self {
        match s {
            "正常" => ProductStatus::正常,
            "已删除" => ProductStatus::已删除,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for ProductStatus {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductStatus::正常 => write!(f,"正常"),
            ProductStatus::已删除 => write!(f,"已删除"),
        }
    }
}
impl ProductStatus {
    pub fn values() -> Vec<ProductStatus> {
        vec![ProductStatus::正常,ProductStatus::已删除,]
    }
}
