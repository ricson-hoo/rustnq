use sqlx::Executor;
use url::Url;

use crate::query::db::{DbPool, DbPoolOptions};

/// 全局连接池（由 Cargo feature 决定 MySQL / PostgreSQL）
pub static POOL: tokio::sync::OnceCell<DbPool> = tokio::sync::OnceCell::const_new();

pub async fn init_pool(url: Url, timezone: Option<i8>) {
    let pool = DbPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(20))
        .after_connect(move |conn, _| {
            Box::pin(async move {
                if let Some(timezone) = timezone {
                    #[cfg(feature = "mysql")]
                    let sql = format!(
                        "SET time_zone='{}{}:00';",
                        if timezone > 0 { "+" } else { "-" },
                        timezone.abs()
                    );
                    #[cfg(feature = "postgres")]
                    let sql = format!(
                        "SET TIME ZONE INTERVAL '{}{} hours';",
                        if timezone > 0 { "+" } else { "-" },
                        timezone.abs()
                    );
                    let _ = conn.execute(sql.as_str()).await;
                }
                Ok(())
            })
        })
        .connect(url.as_str())
        .await
        .expect("Failed to connect to database");
    POOL.set(pool).unwrap();
}
