use serde::{{Serialize, Deserialize}};

use std::fmt;

#[derive(Serialize,Deserialize,Clone,Debug,Copy)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum FileStatus {
    正常,
    已删除,
}

impl From<FileStatus> for String {
    fn from(item: FileStatus) -> Self {
        match item {
            FileStatus::正常 => "正常".to_string(),
            FileStatus::已删除 => "已删除".to_string(),
        }
    }
}

impl From<&str> for FileStatus {
    fn from(s: &str) -> Self {
        match s {
            "正常" => FileStatus::正常,
            "已删除" => FileStatus::已删除,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for FileStatus {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileStatus::正常 => write!(f,"正常"),
            FileStatus::已删除 => write!(f,"已删除"),
        }
    }
}
impl FileStatus {
    pub fn values() -> Vec<FileStatus> {
        vec![FileStatus::正常,FileStatus::已删除,]
    }
}
