use serde::{{Serialize, Deserialize}};

use std::fmt;

#[derive(Serialize,Deserialize,Clone,Debug,Copy)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum ProductPublished {
    是,
    否,
}

impl From<ProductPublished> for String {
    fn from(item: ProductPublished) -> Self {
        match item {
            ProductPublished::是 => "是".to_string(),
            ProductPublished::否 => "否".to_string(),
        }
    }
}

impl From<&str> for ProductPublished {
    fn from(s: &str) -> Self {
        match s {
            "是" => ProductPublished::是,
            "否" => ProductPublished::否,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for ProductPublished {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductPublished::是 => write!(f,"是"),
            ProductPublished::否 => write!(f,"否"),
        }
    }
}
impl ProductPublished {
    pub fn values() -> Vec<ProductPublished> {
        vec![ProductPublished::是,ProductPublished::否,]
    }
}
