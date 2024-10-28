use serde::{{Serialize, Deserialize}};

use std::fmt;

#[derive(Serialize,Deserialize,Clone,Debug,Copy)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum ProductTag {
    流行,
    性价比,
}

impl From<ProductTag> for String {
    fn from(item: ProductTag) -> Self {
        match item {
            ProductTag::流行 => "流行".to_string(),
            ProductTag::性价比 => "性价比".to_string(),
        }
    }
}

impl From<&str> for ProductTag {
    fn from(s: &str) -> Self {
        match s {
            "流行" => ProductTag::流行,
            "性价比" => ProductTag::性价比,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for ProductTag {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductTag::流行 => write!(f,"流行"),
            ProductTag::性价比 => write!(f,"性价比"),
        }
    }
}
impl ProductTag {
    pub fn values() -> Vec<ProductTag> {
        vec![ProductTag::流行,ProductTag::性价比,]
    }
}
