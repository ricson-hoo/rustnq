use serde::{{Serialize, Deserialize}};

use std::fmt;

#[derive(Serialize,Deserialize,Clone,Debug,Copy)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum ProductPromoting {
    是,
    否,
}

impl From<ProductPromoting> for String {
    fn from(item: ProductPromoting) -> Self {
        match item {
            ProductPromoting::是 => "是".to_string(),
            ProductPromoting::否 => "否".to_string(),
        }
    }
}

impl From<&str> for ProductPromoting {
    fn from(s: &str) -> Self {
        match s {
            "是" => ProductPromoting::是,
            "否" => ProductPromoting::否,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for ProductPromoting {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductPromoting::是 => write!(f,"是"),
            ProductPromoting::否 => write!(f,"否"),
        }
    }
}
impl ProductPromoting {
    pub fn values() -> Vec<ProductPromoting> {
        vec![ProductPromoting::是,ProductPromoting::否,]
    }
}
