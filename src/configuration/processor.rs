use std::collections::{HashMap};
use std::error::Error;
use std::sync::{Arc, RwLock};
use once_cell::sync::OnceCell;
use crate::configuration::Field;

pub trait Processor: Send + Sync {
    fn before_save(&self, value: String) ->  Result<String, Box<dyn Error>>;
    fn after_fetch(&self, value: String) ->  Result<String, Box<dyn Error>>;
}

pub struct ProcessorSettings {
    pub processor: Arc<dyn Processor>,
    pub columns:Vec<Field>,
}
pub static PROCESSORS: OnceCell<RwLock<HashMap<Field,Vec<Arc<dyn Processor>>>>> = OnceCell::new();
pub async fn set_processors(processor_settings:ProcessorSettings) {
    let mut map = HashMap::new();
    for column in processor_settings.columns {
        map.entry(column).or_insert_with(Vec::new)
            .push(Arc::clone(&processor_settings.processor));
    }
    let config = RwLock::new(map);

    PROCESSORS.set(config);
}

pub fn get_processors() -> HashMap<Field,Vec<Arc<dyn Processor>>> {
    let read_guard = PROCESSORS.get()
        .expect("PROCESSORS has not been initialized.")
        .read()
        .expect("Failed to acquire read lock.");

    // 返回一个克隆的 HashMap
    read_guard.clone()
}