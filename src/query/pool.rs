use sqlx::Executor;
use url::Url;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::query::db::{DbPool, DbPoolOptions};

/// 全局长连接池（读写）
pub static POOL: tokio::sync::OnceCell<DbPool> = tokio::sync::OnceCell::const_new();

/// 全局备连接池（只读 / fallback）
pub static POOL_B: tokio::sync::OnceCell<DbPool> = tokio::sync::OnceCell::const_new();

/// 标记 POOL_B 是否为只读模式。
pub static IS_POOL_B_READONLY: AtomicBool = AtomicBool::new(false);

pub async fn init_pool(url: Url, timezone: Option<i8>, max_connections: u32) {
    let pool = build_pool(url.as_str(), timezone, max_connections, false).await;
    POOL.set(pool).unwrap();
}

/// 初始化备连接池（只读 / fallback）。
///
/// - `readonly`: 开启后每个连接执行 `SET SESSION transaction_read_only = ON`，
///   从数据库会话层面阻止写操作。
pub async fn init_pool_b(url: Url, timezone: Option<i8>, max_connections: u32, readonly: bool) {
    if readonly {
        IS_POOL_B_READONLY.store(true, Ordering::Release);
    }
    let pool = build_pool(url.as_str(), timezone, max_connections, readonly).await;
    POOL_B.set(pool).unwrap();
}

/// 开发环境退化：POOL_B 指向与 POOL 相同的池（单库场景）。
///
/// 注意：此项仅在 `POOL` 已初始化后调用；
/// 由于 `POOL_B` 和 `POOL` 共享同一个 `DbPool` 实例（Clone），
/// 连接数不会翻倍，行为等价于单池。
pub fn alias_pool_b_to_pool_a() {
    let pool_a = POOL.get().expect("POOL must be initialized before aliasing");
    POOL_B.set(pool_a.clone()).unwrap();
}

async fn build_pool(url: &str, timezone: Option<i8>, max_connections: u32, readonly: bool) -> DbPool {
    let mut builder = DbPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(std::time::Duration::from_secs(20))
        .idle_timeout(std::time::Duration::from_secs(300))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .test_before_acquire(true);

    builder = builder.after_connect(move |conn, _meta| {
        Box::pin(async move {
            if let Some(tz) = timezone {
                #[cfg(feature = "mysql")]
                let tz_sql = format!(
                    "SET time_zone='{}{}:00';",
                    if tz > 0 { "+" } else { "-" },
                    tz.abs()
                );
                #[cfg(feature = "postgres")]
                let tz_sql = format!(
                    "SET TIME ZONE INTERVAL '{}{} hours';",
                    if tz > 0 { "+" } else { "-" },
                    tz.abs()
                );
                let _ = conn.execute(tz_sql.as_str()).await;
            }
            if readonly {
                let _ = conn.execute("SET SESSION transaction_read_only = ON;").await;
            }
            Ok(())
        })
    });

    builder.connect(url).await.expect("Failed to connect to database")
}
