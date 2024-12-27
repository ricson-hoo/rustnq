use url::Url;
use sqlx_mysql::{MySqlPool, MySqlPoolOptions};

pub static POOL: tokio::sync::OnceCell<MySqlPool> = tokio::sync::OnceCell::const_new();

pub async fn init_pool(url: Url) {
    let pool = MySqlPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(20))
        .connect(url.as_str())
        .await.expect("Failed to connect to database");
    POOL.set(pool).unwrap();
}