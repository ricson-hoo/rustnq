//! 数据库后端类型别名。
//!
//! 通过 Cargo feature（`mysql` / `postgres`）在编译期选定后端：
//! - `mysql`（默认）：`DbPool = MySqlPool`、`DbRow = MySqlRow`、`DbQueryResult = MySqlQueryResult`
//! - `postgres`：`DbPool = PgPool`、`DbRow = PgRow`、`DbQueryResult = PgQueryResult`
//!
//! 消费项目（如 zmtcm-backend-rs）：
//! ```toml
//! rustnq = { path = "...", default-features = false, features = ["postgres"] }
//! ```

// 两个后端 feature 互斥。依赖树中如有多个 rustnq 消费方，必须统一使用同一个后端
// feature（不要依赖默认 features），否则这里会报错而不是产生模糊的重复定义错误。
#[cfg(all(feature = "mysql", feature = "postgres"))]
compile_error!("rustnq: features `mysql` and `postgres` are mutually exclusive; 请让依赖树中所有 rustnq 消费方统一使用同一个后端 feature（推荐显式声明 default-features = false）");

#[cfg(feature = "postgres")]
pub use sqlx_postgres::{
    PgPool as DbPool, PgPoolOptions as DbPoolOptions, PgQueryResult as DbQueryResult,
    PgRow as DbRow, Postgres as Db,
};

#[cfg(feature = "mysql")]
pub use sqlx_mysql::{
    MySql as Db, MySqlPool as DbPool, MySqlPoolOptions as DbPoolOptions,
    MySqlQueryResult as DbQueryResult, MySqlRow as DbRow,
};
