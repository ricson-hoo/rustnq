use serde::Deserialize;
use serde::Serialize;
use crate::entity::enums::ProductCategory;
use crate::entity::enums::ProductStatus;
use crate::entity::enums::ProductPublished;
use crate::entity::enums::ProductPromoting;
use chrono;
use crate::entity::enums::ProductTag;

#[derive(Serialize,Deserialize,Clone,Debug)]
#[allow(non_snake_case)]
pub struct Product<D = ()> {
    pub id:Option<String>,
    pub name:Option<String>,
    pub summary:Option<String>,
    pub description:Option<String>,
    pub moq:Option<i32>,
    pub shipping:Option<String>,
    pub material:Option<String>,
    pub category:Option<ProductCategory>,
    pub supplier_id:Option<String>,
    pub status:Option<ProductStatus>,
    pub published:Option<ProductPublished>,
    pub promoting:Option<ProductPromoting>,
    #[serde(deserialize_with = "crate::serde::deserialize_datetime")]
    #[serde(serialize_with = "crate::serde::serialize_datetime")]
    pub created_on:Option<chrono::NaiveDateTime>,
    #[serde(deserialize_with = "crate::serde::deserialize_datetime")]
    #[serde(serialize_with = "crate::serde::serialize_datetime")]
    pub modified_on:Option<chrono::NaiveDateTime>,
    pub created_by:Option<String>,
    pub cover_url:Option<String>,
    pub tag:Option<Vec<ProductTag>>,
    pub weight:Option<i32>,
    pub _associated: Option<D>,
}
impl Product {
    pub fn new() -> Product {
        Product {
            id:None,
            name:None,
            summary:None,
            description:None,
            moq:None,
            shipping:None,
            material:None,
            category:None,
            supplier_id:None,
            status:None,
            published:None,
            promoting:None,
            created_on:None,
            modified_on:None,
            created_by:None,
            cover_url:None,
            tag:None,
            weight:None,
            _associated: None,
        }
    }
}
