use serde::{{Serialize, Deserialize}};

use std::fmt;

#[derive(Serialize,Deserialize,Clone,Debug,Copy)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum FileEntityType {
    Content,
    Product,
    Organization,
    User,
}

impl From<FileEntityType> for String {
    fn from(item: FileEntityType) -> Self {
        match item {
            FileEntityType::Content => "Content".to_string(),
            FileEntityType::Product => "Product".to_string(),
            FileEntityType::Organization => "Organization".to_string(),
            FileEntityType::User => "User".to_string(),
        }
    }
}

impl From<&str> for FileEntityType {
    fn from(s: &str) -> Self {
        match s {
            "Content" => FileEntityType::Content,
            "Product" => FileEntityType::Product,
            "Organization" => FileEntityType::Organization,
            "User" => FileEntityType::User,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for FileEntityType {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileEntityType::Content => write!(f,"Content"),
            FileEntityType::Product => write!(f,"Product"),
            FileEntityType::Organization => write!(f,"Organization"),
            FileEntityType::User => write!(f,"User"),
        }
    }
}
impl FileEntityType {
    pub fn values() -> Vec<FileEntityType> {
        vec![FileEntityType::Content,FileEntityType::Product,FileEntityType::Organization,FileEntityType::User,]
    }
}
