use std::sync::Mutex;
use url::Url;
use lazy_static::lazy::Lazy;
use sqlx_mysql::{MySqlPool, MySqlPoolOptions};
use std::sync::Arc;
use std::ops::Deref;
use std::ops::DerefMut;

//pub static POOL: OnceCell<MySqlPool> = OnceCell::new();
//pub static DATABASE: Lazy<Database> = Lazy::new(Database::init);
pub static POOL: tokio::sync::OnceCell<MySqlPool> = tokio::sync::OnceCell::const_new();

pub async fn init_pool(url: Url) {
    let pool = MySqlPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(20))
        .connect(url.as_str())
        .await.expect("Failed to connect to database");
    POOL.set(pool).unwrap();
}