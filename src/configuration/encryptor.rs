use std::sync::Arc;
use std::cell::RefCell;
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

thread_local! {
    /// build() 阶段传入的 pool 专属 encryptor，优先级高于全局 ENCRYPTOR。
    /// build() 入口设置，退出时由 Drop guard 清理。
    static BUILD_ENCRYPTOR: RefCell<Option<Arc<dyn Encryptor>>> = RefCell::new(None);
}

/// build() 阶段获取当前 encryptor：优先 pool 专属，fallback 全局。
pub fn resolve_encryptor() -> Arc<dyn Encryptor> {
    BUILD_ENCRYPTOR.with(|cell| {
        cell.borrow().clone().unwrap_or_else(|| get_encryptor())
    })
}

/// 设置 build() 阶段的 pool 专属 encryptor，返回 Drop guard（出作用域自动清）。
pub fn set_build_encryptor(enc: Option<Arc<dyn Encryptor>>) -> BuildEncryptorGuard {
    BUILD_ENCRYPTOR.with(|cell| *cell.borrow_mut() = enc);
    BuildEncryptorGuard
}

pub struct BuildEncryptorGuard;
impl Drop for BuildEncryptorGuard {
    fn drop(&mut self) {
        BUILD_ENCRYPTOR.with(|cell| *cell.borrow_mut() = None);
    }
}