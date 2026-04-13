use rustnq::mapping::description::SqlColumn;
use rustnq::query::builder::Condition;
use rustnq::mapping::description::Table;
use shared::entity::Product;
use rustnq::mapping::types::Varchar;
use rustnq::mapping::types::Int;
use shared::entity::enums::ProductCategory;
use rustnq::mapping::types::Enum;
use shared::entity::enums::ProductStatus;
use shared::entity::enums::ProductPublished;
use shared::entity::enums::ProductPromoting;
use rustnq::mapping::types::Datetime;
use shared::entity::enums::ProductTag;
use rustnq::mapping::types::Set;

#[derive(Clone,Debug)]
pub struct ProductTable {
    pub id:Varchar,
    pub name:Varchar,
    pub summary:Varchar,
    pub description:Varchar,
    pub moq:Int,
    pub shipping:Varchar,
    pub material:Varchar,
    pub category:Enum<ProductCategory>,
    pub supplier_id:Varchar,
    pub status:Enum<ProductStatus>,
    pub published:Enum<ProductPublished>,
    pub promoting:Enum<ProductPromoting>,
    pub created_on:Datetime,
    pub modified_on:Datetime,
    pub created_by:Varchar,
    pub cover_url:Varchar,
    pub tag:Set<ProductTag>,
    pub weight:Int,
}
impl ProductTable {
    pub fn new() ->Self {
        ProductTable {
            id:Varchar::name("id".to_string()),
            name:Varchar::name("name".to_string()),
            summary:Varchar::name("summary".to_string()),
            description:Varchar::name("description".to_string()),
            moq:Int::name("moq".to_string()),
            shipping:Varchar::name("shipping".to_string()),
            material:Varchar::name("material".to_string()),
            category:Enum::<ProductCategory>::name("category".to_string()),
            supplier_id:Varchar::name("supplier_id".to_string()),
            status:Enum::<ProductStatus>::name("status".to_string()),
            published:Enum::<ProductPublished>::name("published".to_string()),
            promoting:Enum::<ProductPromoting>::name("promoting".to_string()),
            created_on:Datetime::name("created_on".to_string()),
            modified_on:Datetime::name("modified_on".to_string()),
            created_by:Varchar::name("created_by".to_string()),
            cover_url:Varchar::name("cover_url".to_string()),
            tag:Set::<ProductTag>::name("tag".to_string()),
            weight:Int::name("weight".to_string()),
        }
    }
    pub fn new_with_value(entity:Product) ->Self {
        ProductTable {
            id:Varchar::name_value("id".to_string(), entity.id),
            name:Varchar::name_value("name".to_string(), entity.name),
            summary:Varchar::name_value("summary".to_string(), entity.summary),
            description:Varchar::name_value("description".to_string(), entity.description),
            moq:Int::name_value("moq".to_string(), entity.moq),
            shipping:Varchar::name_value("shipping".to_string(), entity.shipping),
            material:Varchar::name_value("material".to_string(), entity.material),
            category:Enum::<ProductCategory>::name_value("category".to_string(), entity.category),
            supplier_id:Varchar::name_value("supplier_id".to_string(), entity.supplier_id),
            status:Enum::<ProductStatus>::name_value("status".to_string(), entity.status),
            published:Enum::<ProductPublished>::name_value("published".to_string(), entity.published),
            promoting:Enum::<ProductPromoting>::name_value("promoting".to_string(), entity.promoting),
            created_on:Datetime::name_value("created_on".to_string(), entity.created_on),
            modified_on:Datetime::name_value("modified_on".to_string(), entity.modified_on),
            created_by:Varchar::name_value("created_by".to_string(), entity.created_by),
            cover_url:Varchar::name_value("cover_url".to_string(), entity.cover_url),
            tag:Set::<ProductTag>::name_value("tag".to_string(), entity.tag),
            weight:Int::name_value("weight".to_string(), entity.weight),
        }
    }
}
impl Table for ProductTable {
    fn name(&self) -> String {
        "product".to_string()
    }
    fn columns(&self) -> Vec<SqlColumn> {
        vec![
            SqlColumn::Varchar(Some(self.id.clone())),
            SqlColumn::Varchar(Some(self.name.clone())),
            SqlColumn::Varchar(Some(self.summary.clone())),
            SqlColumn::Varchar(Some(self.description.clone())),
            SqlColumn::Int(Some(self.moq.clone())),
            SqlColumn::Varchar(Some(self.shipping.clone())),
            SqlColumn::Varchar(Some(self.material.clone())),
            SqlColumn::Varchar(Some(self.category.clone().into())),
            SqlColumn::Varchar(Some(self.supplier_id.clone())),
            SqlColumn::Varchar(Some(self.status.clone().into())),
            SqlColumn::Varchar(Some(self.published.clone().into())),
            SqlColumn::Varchar(Some(self.promoting.clone().into())),
            SqlColumn::Datetime(Some(self.created_on.clone())),
            SqlColumn::Datetime(Some(self.modified_on.clone())),
            SqlColumn::Varchar(Some(self.created_by.clone())),
            SqlColumn::Varchar(Some(self.cover_url.clone())),
            SqlColumn::Varchar(Some(self.tag.clone().into())),
            SqlColumn::Int(Some(self.weight.clone())),
        ]
    }
    fn primary_key(&self) -> Vec<SqlColumn> {
        vec![
            SqlColumn::Varchar(Some(self.id.clone())),
        ]
    }
}
