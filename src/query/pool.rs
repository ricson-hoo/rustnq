use std::fmt::format;
use sqlx::{database, Executor};
use sqlx::Error;
use url::Url;
use sqlx_mysql::{MySql, MySqlPool, MySqlPoolOptions};
use sqlx::pool::PoolConnectionMetadata;
pub static POOL: tokio::sync::OnceCell<MySqlPool> = tokio::sync::OnceCell::const_new();

pub async fn init_pool(url: Url, timezone: Option<i8>) {
    let pool = MySqlPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(20))
        .after_connect(move |conn,_| {
            Box::pin(async move {
                if let Some(timezone) = timezone {
                    let _ = conn
                        .execute(format!("SET time_zone='{}{}:00';", if timezone > 0 { "+" } else { "-" }, timezone.abs()).as_str())
                        .await;
                }
                Ok(())
            })
        })
        .connect(url.as_str())
        .await.expect("Failed to connect to database");
    POOL.set(pool).unwrap();
}