//! 数据库后端运行时枚举。
//!
//! MySQL 与 PostgreSQL 后端同时编译，运行时根据连接池类型自动分发。
//! SQL 方言差异集中于 [`DbDialect`](super::dialect::DbDialect) 模块。

#[derive(Clone, Debug)]
pub enum DbPool {
    Mysql(sqlx::MySqlPool),
    Postgres(sqlx::PgPool),
}

impl DbPool {
    pub fn as_mysql(&self) -> &sqlx::MySqlPool {
        match self {
            DbPool::Mysql(p) => p,
            DbPool::Postgres(_) => panic!("expected MySQL pool, found Postgres"),
        }
    }

    pub fn as_postgres(&self) -> &sqlx::PgPool {
        match self {
            DbPool::Postgres(p) => p,
            DbPool::Mysql(_) => panic!("expected Postgres pool, found MySQL"),
        }
    }
}

#[derive(Debug)]
pub enum DbRow {
    Mysql(sqlx::mysql::MySqlRow),
    Postgres(sqlx::postgres::PgRow),
}



#[derive(Debug)]
pub enum DbQueryResult {
    Mysql(sqlx::mysql::MySqlQueryResult),
    Postgres(sqlx::postgres::PgQueryResult),
}

impl DbQueryResult {
    pub fn rows_affected(&self) -> u64 {
        match self {
            DbQueryResult::Mysql(r) => r.rows_affected(),
            DbQueryResult::Postgres(r) => r.rows_affected(),
        }
    }

    pub fn last_insert_id(&self) -> Option<u64> {
        match self {
            DbQueryResult::Mysql(r) => Some(r.last_insert_id()),
            DbQueryResult::Postgres(_r) => None,
        }
    }
}
