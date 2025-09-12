use std::sync::Arc;
use once_cell::sync::OnceCell;
use crate::query::builder::Field;

pub trait Encryptor: Send + Sync {
    fn encrypt(&self, value: String) ->  String;
    fn decrypt(&self, value: String) ->  String;
    fn decrypt_field(&self, field: Field) ->  String;
}
pub static ENCRYPTOR: OnceCell<Arc<dyn Encryptor>> = OnceCell::new();
pub fn set_encryptor<E: Encryptor + 'static>(encryptor: E) {
    let _ = ENCRYPTOR.set(Arc::new(encryptor));
}

/// 获取全局的 Encryptor 实例
pub fn get_encryptor() -> Arc<dyn Encryptor> {
    ENCRYPTOR.get().expect("Encryptor has not been set").clone()
}