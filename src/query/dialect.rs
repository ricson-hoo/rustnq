//! 数据库方言（MySQL / PostgreSQL）。
//!
//! 方言在**运行时**由连接池类型决定（默认 `mysql`，PostgreSQL 项目用
//! `default-features = false, features = ["postgres"]`），与 `codegen` 的
//! `TableIntrospector` 双后端设计保持一致。
//!
//! 所有 SQL 语法差异（标识符包裹、LIMIT、upsert、日期函数、类型归一化）
//! 都集中在本模块，查询代码本身无需感知后端差异。

/// 支持的数据库方言（运行时由连接池类型决定）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbDialect {
    MySql,
    Postgres,
}

thread_local! {
    static CURRENT_DIALECT: std::cell::RefCell<Option<DbDialect>> = std::cell::RefCell::new(None);
}

impl DbDialect {
    /// 当前线程方言（由连接池在执行操作前设置）
    pub fn current() -> DbDialect {
        CURRENT_DIALECT.with(|d| {
            d.borrow().unwrap_or(DbDialect::MySql)
        })
    }

    /// 在当前线程设置方言，返回一个 guard，离开作用域时自动恢复
    pub fn set_current(dialect: DbDialect) -> DialectGuard {
        let prev = CURRENT_DIALECT.with(|d| d.borrow_mut().replace(dialect));
        DialectGuard {
            prev,
            _no_send_sync: std::marker::PhantomData,
        }
    }

    pub fn is_mysql(&self) -> bool {
        matches!(self, DbDialect::MySql)
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self, DbDialect::Postgres)
    }

    /// 标识符（字段名/表名）包裹：
    /// - MySQL：保留字用反引号
    /// - PostgreSQL：保留字用双引号（普通小写名直接用裸名，避免双引号导致的大小写敏感）
    ///
    /// 只包裹“普通标识符”（字母、数字、下划线组成）：
    /// `max(price)`、`count(*)` 这类表达式保持原样，避免被包裹后变成非法 SQL。
    /// 支持 `schema.table` / `表.字段` 形式，会逐段包裹。
    pub fn wrap_identifier(&self, name: &str) -> String {
        if !is_plain_identifier(name) {
            return name.to_string();
        }
        if name.contains('.') {
            return name
                .split('.')
                .map(|part| self.wrap_identifier(part))
                .collect::<Vec<String>>()
                .join(".");
        }
        #[cfg(feature = "postgres")]
        {
            if PG_RESERVED_KEYWORDS.contains(&name.to_lowercase().as_str()) {
                return format!("\"{}\"", name);
            }
            return name.to_string();
        }
        #[cfg(feature = "mysql")]
        {
            if MYSQL_KEYWORDS.contains(&name.to_lowercase().as_str()) {
                return format!("`{}`", name);
            }
            return name.to_string();
        }
        #[allow(unreachable_code)]
        name.to_string()
    }

    /// 列引用（可能为 `表.字段`）包裹
    pub fn wrap_qualified_name(&self, table: Option<&str>, name: &str) -> String {
        match table {
            Some(table) if !table.is_empty() => format!(
                "{}.{}",
                self.wrap_identifier(table),
                self.wrap_identifier(name)
            ),
            _ => self.wrap_identifier(name),
        }
    }

    /// 表引用包裹，支持以下写法（表名/别名分别是关键字时也会被包裹）：
    /// - `user`
    /// - `user AS u` / `user u`
    /// - `user FORCE INDEX (idx_user)`（MySQL 提示语法，除表名外原样保留）
    pub fn wrap_table_reference(&self, name: &str) -> String {
        let mut tokens: Vec<String> = name.split_whitespace().map(|s| s.to_string()).collect();
        if tokens.is_empty() {
            return name.to_string();
        }
        tokens[0] = self.wrap_identifier(&tokens[0]);
        if tokens.len() >= 2 {
            // 形如 `table AS alias` 或 `table alias`
            let alias_index = if tokens[1].eq_ignore_ascii_case("as") {
                2
            } else {
                1
            };
            if tokens.len() > alias_index && is_plain_identifier(&tokens[alias_index]) {
                let alias = tokens[alias_index].clone();
                tokens[alias_index] = self.wrap_identifier(&alias);
            }
        }
        tokens.join(" ")
    }

    /// LIMIT 子句
    pub fn limit_clause(&self, offset: i32, limit: i32) -> String {
        match self {
            DbDialect::Postgres => format!("limit {} offset {}", limit, offset),
            DbDialect::MySql => format!("limit {}, {}", offset, limit),
        }
    }

    /// 行值类型归一化：后端类型名 → 统一 [`ValueKind`]
    pub fn value_kind(&self, type_name: &str) -> ValueKind {
        match self {
            DbDialect::Postgres => pg_value_kind(type_name),
            DbDialect::MySql => mysql_value_kind(type_name),
        }
    }

    // ---------- 日期函数（statement.rs 使用） ----------

    /// 今天
    pub fn curdate(&self) -> &'static str {
        match self {
            DbDialect::Postgres => "CURRENT_DATE",
            DbDialect::MySql => "CURDATE()",
        }
    }

    /// TIMESTAMPDIFF(unit, date, CURDATE())：date 距今经过的 unit 数
    pub fn timestamp_diff(&self, unit: &str, date: &str) -> String {
        match self {
            DbDialect::Postgres => {
                match unit {
                    "DAY" => format!("(CURRENT_DATE - ({}::date))", date),
                    "MONTH" => format!(
                        "((EXTRACT(YEAR FROM CURRENT_DATE)::int - EXTRACT(YEAR FROM ({}::date))::int) * 12 + EXTRACT(MONTH FROM CURRENT_DATE)::int - EXTRACT(MONTH FROM ({}::date))::int)",
                        date, date
                    ),
                    "YEAR" => format!(
                        "(EXTRACT(YEAR FROM CURRENT_DATE)::int - EXTRACT(YEAR FROM ({}::date))::int)",
                        date
                    ),
                    _ => format!("(CURRENT_DATE - ({}::date))", date),
                }
            }
            DbDialect::MySql => format!("TIMESTAMPDIFF ({}, {}, CURDATE())", unit, date),
        }
    }

    /// 取年份
    pub fn year(&self, field: &str) -> String {
        match self {
            DbDialect::Postgres => format!("EXTRACT(YEAR FROM {})", field),
            DbDialect::MySql => format!("YEAR({})", field),
        }
    }

    /// 取日期部分
    pub fn date(&self, field: &str) -> String {
        match self {
            DbDialect::Postgres => format!("({})::date", field),
            DbDialect::MySql => format!("DATE({})", field),
        }
    }

    /// 取月份
    pub fn month(&self, field: &str) -> String {
        match self {
            DbDialect::Postgres => format!("EXTRACT(MONTH FROM {})", field),
            DbDialect::MySql => format!("MONTH({})", field),
        }
    }

    /// 日期往前推 n 个 unit（DATE_SUB / PG interval 减法）
    pub fn date_sub(&self, date: &str, value: i32, unit: &str) -> String {
        match self {
            DbDialect::Postgres => {
                match unit {
                    "DAY" => format!("({} - {})", date, value),
                    "MONTH" => format!("({} - INTERVAL '{} months')", date, value),
                    "YEAR" => format!("({} - INTERVAL '{} years')", date, value),
                    _ => format!("({} - {})", date, value),
                }
            }
            DbDialect::MySql => format!("DATE_SUB ({}, INTERVAL {} {})", date, value, unit),
        }
    }

    /// 日期往后推 n 个 unit（DATE_ADD / PG interval 加法）
    pub fn date_add(&self, date: &str, value: i32, unit: &str) -> String {
        match self {
            DbDialect::Postgres => {
                match unit {
                    "DAY" => format!("({} + {})", date, value),
                    "MONTH" => format!("({} + INTERVAL '{} months')", date, value),
                    "YEAR" => format!("({} + INTERVAL '{} years')", date, value),
                    _ => format!("({} + {})", date, value),
                }
            }
            DbDialect::MySql => format!("DATE_ADD ({}, INTERVAL {} {})", date, value, unit),
        }
    }

    /// 与某年份的年差绝对值（ABS(x - YEAR(field))）
    pub fn year_diff(&self, another_year: i32, field: &str) -> String {
        match self {
            DbDialect::Postgres => format!(
                "ABS({} - EXTRACT(YEAR FROM {})::int)",
                another_year, field
            ),
            DbDialect::MySql => format!("ABS({} - YEAR({}))", another_year, field),
        }
    }

    /// 本季度第一天
    pub fn quarter_start(&self) -> String {
        match self {
            DbDialect::Postgres => "DATE_TRUNC('quarter', CURRENT_DATE)::date".to_string(),
            DbDialect::MySql => "MAKEDATE(YEAR(CURDATE()), 1) + INTERVAL (QUARTER(CURDATE())-1) QUARTER".to_string(),
        }
    }

    /// 下季度第一天
    pub fn quarter_end(&self) -> String {
        match self {
            DbDialect::Postgres => "(DATE_TRUNC('quarter', CURRENT_DATE) + INTERVAL '3 months')::date".to_string(),
            DbDialect::MySql => "MAKEDATE(YEAR(CURDATE()), 1) + INTERVAL QUARTER(CURDATE()) QUARTER".to_string(),
        }
    }

    /// 本月第一天
    pub fn month_start(&self) -> String {
        match self {
            DbDialect::Postgres => "DATE_TRUNC('month', CURRENT_DATE)::date".to_string(),
            DbDialect::MySql => "DATE_FORMAT(CURDATE(), '%Y-%m-01')".to_string(),
        }
    }

    /// 下月第一天
    pub fn month_end(&self) -> String {
        match self {
            DbDialect::Postgres => "(DATE_TRUNC('month', CURRENT_DATE) + INTERVAL '1 month')::date".to_string(),
            DbDialect::MySql => "DATE_FORMAT(CURDATE(), '%Y-%m-01') + INTERVAL 1 MONTH".to_string(),
        }
    }

    /// 本周第一天（周日为一周起点，与 MySQL DAYOFWEEK 语义一致）
    pub fn week_start(&self) -> String {
        match self {
            DbDialect::Postgres => "(CURRENT_DATE - EXTRACT(DOW FROM CURRENT_DATE)::int)".to_string(),
            DbDialect::MySql => "DATE_SUB(CURDATE(), INTERVAL DAYOFWEEK(CURDATE())-1 DAY)".to_string(),
        }
    }

    /// 下周日
    pub fn week_end(&self) -> String {
        match self {
            DbDialect::Postgres => "(CURRENT_DATE - EXTRACT(DOW FROM CURRENT_DATE)::int + 7)".to_string(),
            DbDialect::MySql => "DATE_SUB(CURDATE(), INTERVAL DAYOFWEEK(CURDATE())-1 DAY) + INTERVAL 7 DAY".to_string(),
        }
    }

    /// 明日
    pub fn tomorrow(&self) -> String {
        match self {
            DbDialect::Postgres => "(CURRENT_DATE + 1)".to_string(),
            DbDialect::MySql => "CURDATE() + INTERVAL 1 DAY".to_string(),
        }
    }

    /// 行内聚合拼接（MySQL group_concat / PG string_agg）
    pub fn group_concat(&self, fields: &str) -> String {
        match self {
            DbDialect::Postgres => format!("string_agg({}, ',')", fields),
            DbDialect::MySql => format!("group_concat({})", fields),
        }
    }

    /// UNIX_TIMESTAMP(field) — epoch seconds
    pub fn unix_timestamp(&self, field: &str) -> String {
        match self {
            DbDialect::Postgres => format!("EXTRACT(EPOCH FROM {})", field),
            DbDialect::MySql => format!("UNIX_TIMESTAMP({})", field),
        }
    }

    /// UNIX_TIMESTAMP(field) * 1000 — epoch milliseconds
    pub fn unix_timestamp_ms(&self, field: &str) -> String {
        match self {
            DbDialect::Postgres => format!("(EXTRACT(EPOCH FROM {}) * 1000)::bigint", field),
            DbDialect::MySql => format!("CAST(UNIX_TIMESTAMP({}) * 1000 AS SIGNED)", field),
        }
    }

    /// RAND() / RANDOM() — random value
    pub fn rand(&self) -> &'static str {
        match self {
            DbDialect::Postgres => "RANDOM()",
            DbDialect::MySql => "RAND()",
        }
    }

    /// DATEDIFF(date1, date2) — date difference in days
    pub fn datediff(&self, date1: &str, date2: &str) -> String {
        match self {
            DbDialect::Postgres => format!("({}::date - {}::date)", date1, date2),
            DbDialect::MySql => format!("DATEDIFF({}, {})", date1, date2),
        }
    }

    /// DATE_FORMAT / TO_CHAR — date formatting
    pub fn date_format(&self, field: &str, fmt: &str) -> String {
        match self {
            DbDialect::Postgres => format!("TO_CHAR({}, '{}')", field, Self::mysql_to_pg_date_format(fmt)),
            DbDialect::MySql => format!("DATE_FORMAT({}, '{}')", field, fmt),
        }
    }

    /// Convert MySQL DATE_FORMAT pattern to PostgreSQL TO_CHAR pattern
    fn mysql_to_pg_date_format(mysql_fmt: &str) -> String {
        mysql_fmt
            .replace("%Y", "YYYY")
            .replace("%y", "YY")
            .replace("%m", "MM")
            .replace("%c", "FMM")
            .replace("%d", "DD")
            .replace("%e", "FMDD")
            .replace("%H", "HH24")
            .replace("%h", "HH12")
            .replace("%I", "HH12")
            .replace("%i", "MI")
            .replace("%s", "SS")
            .replace("%S", "SS")
            .replace("%p", "AM")
            .replace("%W", "Day")
            .replace("%a", "Dy")
            .replace("%M", "Month")
            .replace("%b", "Mon")
    }

    /// SUBSTRING_INDEX / SPLIT_PART — substring by delimiter
    pub fn substring_index(&self, field: &str, delimiter: &str, count: i32) -> String {
        match self {
            DbDialect::Postgres => {
                if count >= 0 {
                    format!("SPLIT_PART({}, '{}', {})", field, delimiter, count)
                } else {
                    let n = -count;
                    let pos_from_right = if n == 1 {
                        format!("POSITION('{}' IN REVERSE({}))", delimiter, field)
                    } else {
                        format!("POSITION('{}' IN REPLACE(REVERSE({}), '{}', '{}', 1)) + {}",
                            delimiter, field, delimiter, delimiter, n - 1)
                    };
                    format!(
                        "SUBSTRING({}, 1, LENGTH({}) - ({}) + 1)",
                        field, field, pos_from_right
                    )
                }
            }
            DbDialect::MySql => format!("SUBSTRING_INDEX({}, '{}', {})", field, delimiter, count),
        }
    }

    /// CONCAT / || — string concatenation
    pub fn concat(&self, fields: &str) -> String {
        match self {
            DbDialect::Postgres => format!("({})", fields.replace(",", " || ")),
            DbDialect::MySql => format!("concat({})", fields),
        }
    }
}

/// 方言设置守卫：离开作用域时自动恢复上一方言
pub struct DialectGuard {
    prev: Option<DbDialect>,
    _no_send_sync: std::marker::PhantomData<fn()>,
}

impl Drop for DialectGuard {
    fn drop(&mut self) {
        CURRENT_DIALECT.with(|d| *d.borrow_mut() = self.prev);
    }
}

/// 统一的列值种类（跨数据库归一化，供行 → JSON 转换使用）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Str,
    SmallInt,
    Int,
    BigInt,
    UInt,
    TinyInt,
    Decimal,
    Float,
    Bool,
    Enum,
    Set,
    Json,
    DateTime,
    Timestamp,
    Date,
    Time,
    Bytes,
    Uuid,
    Other,
}

fn mysql_value_kind(type_name: &str) -> ValueKind {
    match type_name {
        "VARCHAR" | "CHAR" | "TEXT" | "LONGTEXT" | "TINYTEXT" | "MEDIUMTEXT" => {
            ValueKind::Str
        }
        "JSON" => ValueKind::Json,
        "SMALLINT" => ValueKind::SmallInt,
        "INT" | "MEDIUMINT" | "INTEGER" => ValueKind::Int,
        "BIGINT" => ValueKind::BigInt,
        "BIGINT UNSIGNED" | "INT UNSIGNED" => ValueKind::UInt,
        "TINYINT" => ValueKind::TinyInt,
        "DECIMAL" | "NUMERIC" => ValueKind::Decimal,
        "FLOAT" | "DOUBLE" | "REAL" => ValueKind::Float,
        "BOOLEAN" => ValueKind::Bool,
        "ENUM" => ValueKind::Enum,
        "SET" => ValueKind::Set,
        "DATETIME" => ValueKind::DateTime,
        "TIMESTAMP" => ValueKind::Timestamp,
        "DATE" => ValueKind::Date,
        "TIME" => ValueKind::Time,
        "VARBINARY" | "BINARY" | "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" => {
            ValueKind::Bytes
        }
        _ => ValueKind::Other,
    }
}

fn pg_value_kind(type_name: &str) -> ValueKind {
    let type_name = type_name.to_lowercase();
    // PostgreSQL 数组类型名以 [] 结尾（如 sign_symptom_applicable[]），按数组处理
    if type_name.ends_with("[]") {
        return ValueKind::Set;
    }
    match type_name.as_str() {
        "varchar" | "char" | "bpchar" | "text" | "citext" | "name" => ValueKind::Str,
        "json" | "jsonb" => ValueKind::Json,
        "uuid" => ValueKind::Uuid,
        "int2" | "smallint" | "smallserial" => ValueKind::SmallInt,
        "int4" | "integer" | "serial" => ValueKind::Int,
        "int8" | "bigint" | "bigserial" => ValueKind::BigInt,
        "numeric" | "decimal" => ValueKind::Decimal,
        "float4" | "float8" | "real" | "double precision" | "money" => ValueKind::Float,
        "bool" | "boolean" => ValueKind::Bool,
        "timestamp" | "timestamp without time zone" => ValueKind::DateTime,
        "timestamptz" | "timestamp with time zone" => ValueKind::Timestamp,
        "date" => ValueKind::Date,
        "time" => ValueKind::Time,
        "bytea" => ValueKind::Bytes,
        // PostgreSQL 自定义类型（如 CREATE TYPE 的 enum）按字符串处理
        _ => ValueKind::Other,
    }
}

/// MySQL 关键字（原 builder::wrap_field_name 列表，保持向后兼容）
const MYSQL_KEYWORDS: &[&str] = &[
    "order", "group", "groups", "table", "select", "insert", "update", "delete", "where",
    "from", "as", "and", "or", "not", "null", "is", "in", "like", "between",
    "join", "left", "right", "inner", "outer", "on", "by", "having", "distinct",
    "limit", "offset", "set", "values", "into", "create", "alter", "drop",
    "index", "primary", "key", "foreign", "references", "constraint", "default",
    "auto_increment", "desc", "asc", "union", "all", "case", "when", "then",
    "else", "end", "exists", "any", "some", "user", "database", "schema",
    "view", "procedure", "function", "trigger", "event", "temporary", "if",
    "elseif", "begin", "declare", "call", "execute", "show",
    "describe", "explain", "use", "grant", "revoke", "privileges", "flush",
    "lock", "unlock", "commit", "rollback", "savepoint", "transaction",
    "character", "collate", "engine", "row_format", "comment", "partition",
    "check", "cascade", "restrict", "no", "action", "match", "full", "natural",
    "cross", "using", "current_date", "current_time", "current_timestamp",
    "interval", "year", "month", "day", "hour", "minute", "second", "microsecond",
    "date", "time", "timestamp", "datetime", "char", "varchar", "text",
    "tinytext", "mediumtext", "longtext", "blob", "tinyblob", "mediumblob",
    "longblob", "enum", "set", "decimal", "numeric", "float", "double", "real",
    "bit", "bool", "boolean", "serial", "bigserial", "smallserial", "money",
    "uuid", "json", "jsonb", "xml", "array", "range", "multirange", "domain",
];

/// 是否为“普通标识符”：仅由字母、数字、下划线（以及非首位的 `$`）组成。
///
/// 只有普通标识符才需要（也才允许）被引号包裹；
/// `max(price)`、`count(*)`、`a + b` 这类表达式必须原样保留。
fn is_plain_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.chars().enumerate().all(|(index, c)| {
            c == '_'
                || c.is_ascii_alphabetic()
                || (index > 0 && (c.is_ascii_digit() || c == '$'))
        })
}

/// PostgreSQL 关键字全集（保留字 + 可作为函数/类型名的关键字 + 非保留关键字）。
///
/// 非保留关键字在多数位置可以不加引号，但加引号总是安全的，
/// 因此这里一并列入，避免出现遗漏关键字导致的语法错误（如 `user` 表）。
const PG_RESERVED_KEYWORDS: &[&str] = &[
    // ---- 保留字 ----
    "all", "analyse", "analyze", "and", "any", "array", "as", "asc", "asymmetric",
    "authorization", "binary", "both", "case", "cast", "check", "collate", "collation",
    "column", "concurrently", "constraint", "create", "cross", "current_catalog",
    "current_date", "current_role", "current_schema", "current_time", "current_timestamp",
    "current_user", "default", "deferrable", "desc", "distinct", "do", "else", "end",
    "except", "false", "fetch", "for", "foreign", "freeze", "from", "full", "grant",
    "group", "having", "ilike", "in", "initially", "inner", "intersect", "into", "is",
    "isnull", "join", "lateral", "leading", "left", "like", "limit", "localtime",
    "localtimestamp", "natural", "not", "notnull", "null", "offset", "on", "only", "or",
    "order", "outer", "overlaps", "placing", "primary", "references", "returning",
    "right", "select", "session_user", "similar", "some", "symmetric", "table", "then",
    "to", "trailing", "true", "union", "unique", "user", "using", "variadic", "verbose",
    "when", "where", "window", "with",
    // ---- 可作为函数或类型名的关键字 ----
    "between", "bigint", "bit", "boolean", "char", "character", "coalesce", "dec",
    "decimal", "exists", "extract", "float", "greatest", "grouping", "inout", "int",
    "integer", "interval", "least", "national", "nchar", "none", "nullif", "numeric",
    "out", "overlay", "position", "precision", "real", "row", "setof", "smallint",
    "substring", "time", "timestamp", "treat", "trim", "values", "varchar", "xmlattributes",
    "xmlconcat", "xmlelement", "xmlexists", "xmlforest", "xmlnamespaces", "xmlparse",
    "xmlpi", "xmlroot", "xmlserialize", "xmltable",
    // ---- 非保留关键字（加引号同样安全）----
    "abort", "absolute", "access", "action", "add", "admin", "after", "aggregate", "also",
    "alter", "always", "assertion", "assignment", "at", "attach", "attribute", "backward",
    "before", "begin", "by", "cache", "call", "called", "cascade", "cascaded", "chain",
    "characteristics", "checkpoint", "class", "close", "cluster", "comment", "comments",
    "commit", "committed", "compression", "configuration", "conflict", "connection",
    "constraints", "content", "continue", "conversion", "copy", "cost", "csv", "cube",
    "current", "cursor", "cycle", "data", "database", "day", "deallocate", "declare",
    "defaults", "deferred", "definer", "delete", "delimiter", "delimiters", "depends",
    "detach", "dictionary", "disable", "discard", "document", "domain", "double", "drop",
    "each", "enable", "encoding", "encrypted", "enum", "escape", "event", "exclude",
    "excluding", "exclusive", "execute", "explain", "expression", "extension", "external",
    "family", "filter", "finalize", "first", "following", "force", "format", "forward",
    "function", "functions", "generated", "global", "granted", "groups", "handler",
    "header", "hold", "hour", "identity", "if", "immediate", "immutable", "implicit",
    "import", "include", "including", "increment", "index", "indexes", "inherit",
    "inherits", "inline", "input", "insensitive", "insert", "instead", "invoker",
    "isolation", "key", "keys", "label", "language", "large", "last", "leakproof",
    "level", "listen", "load", "local", "location", "lock", "locked", "logged", "mapping",
    "match", "materialized", "maxvalue", "method", "minute", "minvalue", "mode", "month",
    "move", "name", "names", "new", "next", "nfc", "nfd", "nfkc", "nfkd", "no", "nothing",
    "notify", "nowait", "nulls", "object", "of", "off", "oids", "old", "operator",
    "option", "options", "ordinality", "others", "over", "override", "owned", "owner",
    "parallel", "parser", "partial", "partition", "passing", "password", "plans", "policy",
    "preceding", "prepare", "prepared", "preserve", "prior", "privileges", "procedural",
    "procedure", "procedures", "program", "publication", "quote", "range", "read",
    "reassign", "recheck", "recursive", "ref", "referencing", "refresh", "reindex",
    "relative", "release", "rename", "repeatable", "replace", "replica", "reset",
    "restart", "restrict", "return", "returns", "revoke", "role", "rollback", "rollup",
    "routine", "routines", "rows", "rule", "savepoint", "schema", "schemas", "scroll",
    "search", "second", "security", "sequence", "sequences", "serializable", "server",
    "session", "set", "sets", "share", "show", "simple", "skip", "snapshot", "sql",
    "stable", "standalone", "start", "statement", "statistics", "stdin", "stdout",
    "storage", "stored", "strict", "strip", "subscription", "support", "sysid", "system",
    "tables", "tablespace", "temp", "template", "temporary", "text", "ties",
    "transaction", "transform", "trigger", "truncate", "trusted", "type", "types",
    "unbounded", "uncommitted", "unencrypted", "unknown", "unlisten", "unlogged",
    "until", "update", "vacuum", "valid", "validate", "validator", "value", "varying",
    "view", "views", "volatile", "whenever", "whitespace", "work", "wrapper", "write",
    "xml", "year", "yes", "zone",
];