use serde::{Serialize,Deserialize};

#[derive(Debug, Deserialize,Serialize)]
pub struct PagingData<T> {
    pub data: Vec<T>,
    pub current_page: Option<i32>,
    pub page_size: Option<i32>,
    pub total_count: Option<i64>,
}

impl<T> PagingData<T> {
    pub fn default() -> Self {
        PagingData {
            data: Vec::new(),
            current_page: None,
            page_size: None,
            total_count: None,
        }
    }

    pub fn new(data: Vec<T>, current_page: Option<i32>, page_size: Option<i32>, total_count: Option<i64>) -> Self {
        PagingData {
            data,
            current_page,
            page_size,
            total_count,
        }
    }
}