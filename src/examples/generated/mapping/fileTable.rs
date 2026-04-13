use rustnq::mapping::description::SqlColumn;
use rustnq::query::builder::Condition;
use rustnq::mapping::description::Table;
use shared::entity::File;
use rustnq::mapping::types::Varchar;
use shared::entity::enums::FileType;
use rustnq::mapping::types::Enum;
use shared::entity::enums::FileEntityType;
use rustnq::mapping::types::Int;
use rustnq::mapping::types::Timestamp;
use shared::entity::enums::FileStatus;

#[derive(Clone,Debug)]
pub struct FileTable {
    pub id:Varchar,
    pub type_:Enum<FileType>,
    pub entity_type:Enum<FileEntityType>,
    pub entity_id:Varchar,
    pub path:Varchar,
    pub url:Varchar,
    pub weight:Int,
    pub created_on:Timestamp,
    pub status:Enum<FileStatus>,
    pub title:Varchar,
    pub name:Varchar,
}
impl FileTable {
    pub fn new() ->Self {
        FileTable {
            id:Varchar::name("id".to_string()),
            type_:Enum::<FileType>::name("type".to_string()),
            entity_type:Enum::<FileEntityType>::name("entity_type".to_string()),
            entity_id:Varchar::name("entity_id".to_string()),
            path:Varchar::name("path".to_string()),
            url:Varchar::name("url".to_string()),
            weight:Int::name("weight".to_string()),
            created_on:Timestamp::name("created_on".to_string()),
            status:Enum::<FileStatus>::name("status".to_string()),
            title:Varchar::name("title".to_string()),
            name:Varchar::name("name".to_string()),
        }
    }
    pub fn new_with_value(entity:File) ->Self {
        FileTable {
            id:Varchar::name_value("id".to_string(), entity.id),
            type_:Enum::<FileType>::name_value("type".to_string(), entity.type_),
            entity_type:Enum::<FileEntityType>::name_value("entity_type".to_string(), entity.entity_type),
            entity_id:Varchar::name_value("entity_id".to_string(), entity.entity_id),
            path:Varchar::name_value("path".to_string(), entity.path),
            url:Varchar::name_value("url".to_string(), entity.url),
            weight:Int::name_value("weight".to_string(), entity.weight),
            created_on:Timestamp::name_value("created_on".to_string(), entity.created_on),
            status:Enum::<FileStatus>::name_value("status".to_string(), entity.status),
            title:Varchar::name_value("title".to_string(), entity.title),
            name:Varchar::name_value("name".to_string(), entity.name),
        }
    }
}
impl Table for FileTable {
    fn name(&self) -> String {
        "file".to_string()
    }
    fn columns(&self) -> Vec<SqlColumn> {
        vec![
            SqlColumn::Varchar(Some(self.id.clone())),
            SqlColumn::Varchar(Some(self.type_.clone().into())),
            SqlColumn::Varchar(Some(self.entity_type.clone().into())),
            SqlColumn::Varchar(Some(self.entity_id.clone())),
            SqlColumn::Varchar(Some(self.path.clone())),
            SqlColumn::Varchar(Some(self.url.clone())),
            SqlColumn::Int(Some(self.weight.clone())),
            SqlColumn::Timestamp(Some(self.created_on.clone())),
            SqlColumn::Varchar(Some(self.status.clone().into())),
            SqlColumn::Varchar(Some(self.title.clone())),
            SqlColumn::Varchar(Some(self.name.clone())),
        ]
    }
    fn primary_key(&self) -> Vec<SqlColumn> {
        vec![
            SqlColumn::Varchar(Some(self.id.clone())),
        ]
    }
}
