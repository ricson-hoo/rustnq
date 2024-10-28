#[allow(non_snake_case)]
pub mod fileType;
pub use fileType::FileType;
pub mod fileEntityType;
pub use fileEntityType::FileEntityType;
pub mod fileStatus;
pub use fileStatus::FileStatus;
pub mod productCategory;
pub use productCategory::ProductCategory;
pub mod productStatus;
pub use productStatus::ProductStatus;
pub mod productPublished;
pub use productPublished::ProductPublished;
pub mod productPromoting;
pub use productPromoting::ProductPromoting;
pub mod productTag;
pub use productTag::ProductTag;
pub trait PermissionType {}
pub trait MenuType {}
