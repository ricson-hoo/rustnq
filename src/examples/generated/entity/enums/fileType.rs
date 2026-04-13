use serde::{{Serialize, Deserialize}};

use std::fmt;

#[derive(Serialize,Deserialize,Clone,Debug,Copy)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum FileType {
    Cover,
    Image,
    Avatar,
    Audio,
    Video,
    Attachment,
}

impl From<FileType> for String {
    fn from(item: FileType) -> Self {
        match item {
            FileType::Cover => "Cover".to_string(),
            FileType::Image => "Image".to_string(),
            FileType::Avatar => "Avatar".to_string(),
            FileType::Audio => "Audio".to_string(),
            FileType::Video => "Video".to_string(),
            FileType::Attachment => "Attachment".to_string(),
        }
    }
}

impl From<&str> for FileType {
    fn from(s: &str) -> Self {
        match s {
            "Cover" => FileType::Cover,
            "Image" => FileType::Image,
            "Avatar" => FileType::Avatar,
            "Audio" => FileType::Audio,
            "Video" => FileType::Video,
            "Attachment" => FileType::Attachment,
            &_ => todo!(),
        }
    }
}
impl fmt::Display for FileType {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileType::Cover => write!(f,"Cover"),
            FileType::Image => write!(f,"Image"),
            FileType::Avatar => write!(f,"Avatar"),
            FileType::Audio => write!(f,"Audio"),
            FileType::Video => write!(f,"Video"),
            FileType::Attachment => write!(f,"Attachment"),
        }
    }
}
impl FileType {
    pub fn values() -> Vec<FileType> {
        vec![FileType::Cover,FileType::Image,FileType::Avatar,FileType::Audio,FileType::Video,FileType::Attachment,]
    }
}
