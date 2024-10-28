use serde::Deserialize;
use serde::Serialize;
use crate::entity::enums::FileType;
use crate::entity::enums::FileEntityType;
use chrono;
use crate::entity::enums::FileStatus;

#[derive(Serialize,Deserialize,Clone,Debug)]
#[allow(non_snake_case)]
pub struct File<D = ()> {
    pub id:Option<String>,
    #[serde(rename = "type")] pub type_:Option<FileType>,
    pub entity_type:Option<FileEntityType>,
    pub entity_id:Option<String>,
    pub path:Option<String>,
    pub url:Option<String>,
    pub weight:Option<i32>,
    #[serde(deserialize_with = "crate::serde::deserialize_datetime")]
    #[serde(serialize_with = "crate::serde::serialize_datetime")]
    pub created_on:Option<chrono::NaiveDateTime>,
    pub status:Option<FileStatus>,
    pub title:Option<String>,
    pub name:Option<String>,
    pub _associated: Option<D>,
}
impl File {
    pub fn new() -> File {
        File {
            id:None,
            type_:None,
            entity_type:None,
            entity_id:None,
            path:None,
            url:None,
            weight:None,
            created_on:None,
            status:None,
            title:None,
            name:None,
            _associated: None,
        }
    }
}
