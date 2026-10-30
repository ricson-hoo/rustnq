use sqlx::Executor;
use std::sync::Arc;
use std::sync::Mutex;
use crate::configuration::Encryptor;
use crate::query::db::{DbPool};
use crate::query::dialect::DbDialect;

pub trait DbConf: Send + Sync + std::fmt::Debug + 'static {
    fn url(&self) -> &str;
    fn dialect(&self) -> DbDialect;
    fn timezone(&self) -> Option<i8> { Some(8) }
    fn max_connections(&self) -> u32 { 200 }
    fn is_readonly(&self) -> bool { false }
    fn encryptor(&self) -> Option<Arc<dyn Encryptor>> { None }
    fn pool_key(&self) -> &str { "main" }
}

    pub trait DbKey {
        fn db_key(&self) -> &str;
    }

struct PoolEntry {
    conf: Arc<dyn DbConf + Send + Sync>,
    pool: DbPool,
}

use lazy_static::lazy_static;

lazy_static! {
    static ref POOLS: Mutex<Vec<PoolEntry>> = Mutex::new(Vec::new());
}

pub async fn init_pools<P: DbConf>(configs: Vec<P>) {
    for conf in configs {
        let url = conf.url().to_string();
        {
            let pools = POOLS.lock().unwrap();
            if pools.iter().any(|e| e.conf.url() == url) {
                continue;
            }
        }
        let pool = build_pool(conf.url(), conf.dialect(), conf.timezone(), conf.max_connections(), conf.is_readonly()).await;
        let mut pools = POOLS.lock().unwrap();
        pools.push(PoolEntry { conf: Arc::new(conf), pool });
    }
}

pub(crate) fn resolve_pool(
    conf: Option<&(dyn DbConf + Send + Sync)>,
) -> (DbPool, std::sync::Arc<dyn DbConf>) {
    let pools = POOLS.lock().unwrap();
    let entry = if let Some(c) = conf {
        pools.iter()
            .find(|e| e.conf.url() == c.url())
            .expect("pool not registered")
    } else {
        pools.first().expect("no pool configured")
    };
    (entry.pool.clone(), entry.conf.clone())
}

async fn build_pool(url: &str, dialect: DbDialect, timezone: Option<i8>, max_connections: u32, readonly: bool) -> DbPool {
    match dialect {
        DbDialect::MySql => {
            let mut builder = sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(max_connections)
                .acquire_timeout(std::time::Duration::from_secs(20))
                .idle_timeout(std::time::Duration::from_secs(300))
                .max_lifetime(std::time::Duration::from_secs(1800))
                .test_before_acquire(true);

            builder = builder.after_connect(move |conn, _meta| {
                Box::pin(async move {
                    if let Some(tz) = timezone {
                        let sign = if tz >= 0 { "+" } else { "-" };
                        let _ = conn.execute(
                            format!("SET time_zone='{}{}:00';", sign, tz.abs()).as_str()
                        ).await;
                    }
                    if readonly {
                        let _ = conn.execute("SET SESSION transaction_read_only = ON;").await;
                    }
                    Ok(())
                })
            });

            DbPool::Mysql(builder.connect(url).await.expect("Failed to connect to MySQL database"))
        }
        DbDialect::Postgres => {
            let mut builder = sqlx::postgres::PgPoolOptions::new()
                .max_connections(max_connections)
                .acquire_timeout(std::time::Duration::from_secs(20))
                .idle_timeout(std::time::Duration::from_secs(300))
                .max_lifetime(std::time::Duration::from_secs(1800))
                .test_before_acquire(true);

            builder = builder.after_connect(move |conn, _meta| {
                Box::pin(async move {
                    if let Some(tz) = timezone {
                        let _ = conn.execute(
                            format!("SET TIME ZONE '{}{:02}:00';", if tz >= 0 { "+" } else { "-" }, tz.abs()).as_str()
                        ).await;
                    }
                    if readonly {
                        let _ = conn.execute("SET SESSION default_transaction_read_only = on;").await;
                    }
                    Ok(())
                })
            });

            DbPool::Postgres(builder.connect(url).await.expect("Failed to connect to PostgreSQL database"))
        }
    }
}

pub fn get_db(url: &str) -> DbPool {
    POOLS.lock().unwrap()
        .iter()
        .find(|e| e.conf.url() == url)
        .map(|e| e.pool.clone())
        .expect("pool not registered")
}

pub fn default_db() -> DbPool {
    POOLS.lock().unwrap()
        .first()
        .map(|e| e.pool.clone())
        .expect("no pool configured")
}

pub fn pool_by_key(key: &str) -> DbPool {
    POOLS.lock().unwrap()
        .iter()
        .find(|e| e.conf.pool_key() == key)
        .map(|e| e.pool.clone())
        .unwrap_or_else(default_db)
}

pub fn pool_of(k: impl DbKey) -> DbPool {
    pool_by_key(k.db_key())
}

pub(crate) fn resolve_conf_by_key(key: &str) -> Arc<dyn DbConf + Send + Sync> {
    POOLS.lock().unwrap()
        .iter()
        .find(|e| e.conf.pool_key() == key)
        .map(|e| e.conf.clone())
        .expect("pool not registered")
}