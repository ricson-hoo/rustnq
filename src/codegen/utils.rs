use std::fs;
use std::collections::HashMap;

use sqlx::{Column, Row};
use sqlx::pool::Pool;
use sqlx_mysql::{MySql, MySqlRow};
use sqlx_postgres::{PgRow, Postgres};
use crate::codegen::entity::NamingConvention;
use crate::utils::stringUtils;

#[derive(Debug)]
pub struct TableRow {
    pub name: String,
}

/// SHOW INDEX 返回的每一行，代表"一个索引的一列"
#[derive(Debug)]
pub struct TableIndexColumnRow {
    pub index_name: String,
    pub column_name: String,
    pub non_unique: bool,
}

/// 分组后的表索引信息
#[derive(Debug)]
pub struct TableIndexRow {
    pub index_name: String,
    pub columns: Vec<String>,
    pub non_unique: bool,
}

#[derive(Debug)]
pub struct TableFieldRow {
    pub(crate) name: String,
    pub(crate) data_type: String,//varchar(32),enum('a','b'),set('a','b'),tinyint(1)
    pub(crate) nullable:bool,
    pub(crate) is_primary_key:bool
}

#[derive(Debug)]
pub struct TableFullFieldRow {
    pub(crate) name: String,
    pub(crate) comment: String,
    pub(crate) data_type: String,//varchar(32),enum('a','b'),set('a','b'),tinyint(1)
    pub(crate) nullable:bool,
    pub(crate) is_primary_key:bool
}

/// MySQL: `SHOW TABLES` 返回的列名是动态的（Tables_in_<db>）；PostgreSQL: information_schema 侧统一别名成 Tables_in_*
fn parse_table_row_mysql(row: &MySqlRow) -> TableRow {
    let mut str = "".to_string();
    for column in row.columns() {
        let column_name = column.name();
        if column_name.starts_with("Tables_in_") {
            // MySQL 下该列可能是 VARBINARY(BLOB)，先按 String 读，失败再按字节读
            if let Ok(value) = row.try_get::<String, _>(column_name) {
                str = value;
            } else if let Ok(value) = row.try_get::<Vec<u8>, _>(column_name) {
                str = String::from_utf8_lossy(&value).to_string();
            } else {
                str = format!("Error retrieving value for column '{}'", column_name);
            }
        }
    }
    TableRow {
        name: str.to_string()
    }
}

fn parse_index_column_row_mysql(row: &MySqlRow) -> TableIndexColumnRow {
    let index_name = row.try_get::<String,_>("Key_name").unwrap_or_else(|error|{
        panic!("failed to get Key_name: {}",error);
    });
    let column_name = row.try_get::<String,_>("Column_name").unwrap_or_default();
    // Non_unique: 0 表示唯一索引，1 表示非唯一
    let non_unique = row.try_get::<i64,_>("Non_unique").unwrap_or(1) != 0;
    TableIndexColumnRow {
        index_name,
        column_name,
        non_unique,
    }
}

/// MySQL 的 varchar 列在特定字符集下可能以 BLOB 返回，先按 String 读，失败再按字节读
fn get_str_field_mysql(row: &MySqlRow, column_name: &str) -> String {
    match row.try_get::<String, _>(column_name) {
        Ok(value) => value,
        Err(_) => {
            let blob_value: Vec<u8> = row.try_get(column_name).unwrap_or_default();
            String::from_utf8_lossy(&blob_value).to_string()
        }
    }
}

fn parse_field_row_mysql(row: &MySqlRow) -> TableFieldRow {
    let name_value = row.try_get::<String,_>("Field").unwrap_or_else(|error|{
        panic!("failed to get name: {}",error);
    });
    let type_value = get_str_field_mysql(row, "Type");
    let nullable_value = get_str_field_mysql(row, "Null");
    let primary_value = get_str_field_mysql(row, "Key");

    TableFieldRow {
        name: name_value,
        data_type: type_value,
        nullable: "Yes" == nullable_value || "YES" == nullable_value,
        is_primary_key: "Pri" == primary_value || "PRI" == primary_value,
    }
}

fn parse_full_field_row_mysql(row: &MySqlRow) -> TableFullFieldRow {
    let field_row = parse_field_row_mysql(row);
    TableFullFieldRow {
        name: field_row.name,
        comment: get_str_field_mysql(row, "Comment"),
        data_type: field_row.data_type,
        nullable: field_row.nullable,
        is_primary_key: field_row.is_primary_key,
    }
}

/// PostgreSQL 侧全部按 String 解码（information_schema 返回文本类型），无需 BLOB fallback
fn parse_table_row_pg(row: &PgRow) -> TableRow {
    let name = row.try_get::<String, _>("Tables_in_postgres").unwrap_or_default();
    TableRow { name }
}

fn parse_index_column_row_pg(row: &PgRow) -> TableIndexColumnRow {
    let index_name = row.try_get::<String,_>("Key_name").unwrap_or_else(|error|{
        panic!("failed to get Key_name: {}",error);
    });
    let column_name = row.try_get::<String,_>("Column_name").unwrap_or_default();
    // Non_unique: 0 表示唯一索引，1 表示非唯一（查询侧已用 CASE 对齐该语义）
    let non_unique = row.try_get::<i64,_>("Non_unique").unwrap_or(1) != 0;
    TableIndexColumnRow {
        index_name,
        column_name,
        non_unique,
    }
}

fn get_str_field_pg(row: &PgRow, column_name: &str) -> String {
    row.try_get::<String, _>(column_name).unwrap_or_default()
}

fn parse_field_row_pg(row: &PgRow) -> TableFieldRow {
    let name_value = row.try_get::<String,_>("Field").unwrap_or_else(|error|{
        panic!("failed to get name: {}",error);
    });
    let nullable_value = get_str_field_pg(row, "Null");
    let primary_value = get_str_field_pg(row, "Key");

    TableFieldRow {
        name: name_value,
        data_type: get_str_field_pg(row, "Type"),
        nullable: "Yes" == nullable_value || "YES" == nullable_value,
        is_primary_key: "Pri" == primary_value || "PRI" == primary_value,
    }
}

fn parse_full_field_row_pg(row: &PgRow) -> TableFullFieldRow {
    let field_row = parse_field_row_pg(row);
    TableFullFieldRow {
        name: field_row.name,
        comment: get_str_field_pg(row, "Comment"),
        data_type: field_row.data_type,
        nullable: field_row.nullable,
        is_primary_key: field_row.is_primary_key,
    }
}

/// PostgreSQL: 表列表。列名对齐 MySQL `SHOW TABLES` 的 "Tables_in_*" 动态列名，复用同一解析。
/// 注意：表按连接当前的 search_path（current_schema()）读取。
const PG_TABLES_SQL: &str = r#"
SELECT table_name AS "Tables_in_postgres"
FROM information_schema.tables
WHERE table_schema = current_schema()
  AND table_type = 'BASE TABLE'
ORDER BY table_name
"#;

/// PostgreSQL: 表字段。列名与取值语义对齐 MySQL `SHOW FULL COLUMNS`（Field/Type/Null/Key/Default/Comment）。
/// 其中 Type 被归一化成现有解析器认识的 MySQL 风格字符串：
///   - 自定义枚举列   -> enum('a','b')        （实体层生成 Enum<T>）
///   - 自定义枚举数组 -> set('a','b')         （与 MySQL SET 行为一致，实体层生成 Set<T> 并自动生成枚举）
///   - 标量数组       -> int[]/text[]/...     （实体层生成 Option<Vec<T>>）
///   - 标量列         -> varchar(32)/int/bigint/boolean/...
const PG_TABLE_FIELDS_SQL: &str = r#"
SELECT
    a.attname AS "Field",
    CASE
        WHEN t.typtype = 'e' THEN
            'enum(' || (SELECT string_agg(quote_literal(e.enumlabel), ',' ORDER BY e.enumsortorder)
                        FROM pg_enum e WHERE e.enumtypid = t.oid) || ')'
        WHEN et.typtype = 'e' THEN
            'set(' || (SELECT string_agg(quote_literal(e.enumlabel), ',' ORDER BY e.enumsortorder)
                       FROM pg_enum e WHERE e.enumtypid = et.oid) || ')'
        WHEN t.typelem <> 0 THEN
            CASE et.typname
                WHEN 'int4' THEN 'int[]'
                WHEN 'int8' THEN 'bigint[]'
                WHEN 'int2' THEN 'smallint[]'
                WHEN 'bool' THEN 'boolean[]'
                WHEN 'text' THEN 'text[]'
                WHEN 'varchar' THEN 'varchar[]'
                WHEN 'bpchar' THEN 'char[]'
                WHEN 'numeric' THEN 'numeric[]'
                WHEN 'float8' THEN 'double[]'
                WHEN 'float4' THEN 'float[]'
                WHEN 'date' THEN 'date[]'
                WHEN 'time' THEN 'time[]'
                WHEN 'timestamp' THEN 'datetime[]'
                WHEN 'timestamptz' THEN 'timestamp[]'
                WHEN 'bytea' THEN 'blob[]'
                WHEN 'json' THEN 'json[]'
                WHEN 'jsonb' THEN 'json[]'
                WHEN 'uuid' THEN 'varchar[]'
                ELSE 'varchar[]'
            END
        ELSE
            CASE t.typname
                WHEN 'varchar' THEN CASE WHEN a.atttypmod > 4 THEN 'varchar(' || (a.atttypmod - 4) || ')' ELSE 'varchar' END
                WHEN 'bpchar' THEN CASE WHEN a.atttypmod > 4 THEN 'char(' || (a.atttypmod - 4) || ')' ELSE 'char' END
                WHEN 'int4' THEN 'int'
                WHEN 'int8' THEN 'bigint'
                WHEN 'int2' THEN 'smallint'
                WHEN 'bool' THEN 'boolean'
                WHEN 'numeric' THEN CASE WHEN a.atttypmod > 4
                    THEN 'numeric(' || (((a.atttypmod - 4) >> 16) & 65535) || ',' || ((a.atttypmod - 4) & 65535) || ')'
                    ELSE 'numeric' END
                WHEN 'float8' THEN 'double'
                WHEN 'float4' THEN 'float'
                WHEN 'text' THEN 'text'
                WHEN 'bytea' THEN 'blob'
                WHEN 'json' THEN 'json'
                WHEN 'jsonb' THEN 'json'
                WHEN 'uuid' THEN 'varchar(36)'
                WHEN 'date' THEN 'date'
                WHEN 'time' THEN 'time'
                WHEN 'timestamp' THEN 'datetime'
                WHEN 'timestamptz' THEN 'timestamp'
                ELSE 'varchar'
            END
    END AS "Type",
    CASE WHEN a.attnotnull THEN 'NO' ELSE 'YES' END AS "Null",
    CASE WHEN EXISTS (
        SELECT 1 FROM pg_index ix
        WHERE ix.indrelid = c.oid AND ix.indisprimary AND a.attnum = ANY(ix.indkey)
    ) THEN 'PRI' ELSE '' END AS "Key",
    COALESCE(pg_get_expr(ad.adbin, ad.adrelid), '') AS "Default",
    COALESCE(col_description(c.oid, a.attnum), '') AS "Comment"
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
JOIN pg_attribute a ON a.attrelid = c.oid
LEFT JOIN pg_type t ON t.oid = a.atttypid
LEFT JOIN pg_type et ON et.oid = t.typelem
LEFT JOIN pg_attrdef ad ON ad.adrelid = c.oid AND ad.adnum = a.attnum
WHERE n.nspname = current_schema()
  AND c.relname = $1
  AND c.relkind = 'r'
  AND a.attnum > 0
  AND NOT a.attisdropped
ORDER BY a.attnum
"#;

/// PostgreSQL: 表索引。列名与语义对齐 MySQL `SHOW INDEX`（Key_name/Column_name/Non_unique）：
///   - 主键索引用 "PRIMARY" 命名，与 MySQL 一致，下游跳过逻辑可直接复用
///   - Non_unique: 0 唯一 / 1 非唯一，与 MySQL 一致
///   - unnest(ix.indkey::int2[]) WITH ORDINALITY 保证多列索引的列顺序
const PG_TABLE_INDEXES_SQL: &str = r#"
SELECT
    CASE WHEN ix.indisprimary THEN 'PRIMARY' ELSE i.relname END AS "Key_name",
    a.attname AS "Column_name",
    CASE WHEN ix.indisunique THEN 0 ELSE 1 END AS "Non_unique"
FROM pg_index ix
JOIN pg_class i ON i.oid = ix.indexrelid
JOIN pg_class t ON t.oid = ix.indrelid
JOIN pg_namespace n ON n.oid = t.relnamespace
JOIN LATERAL unnest(ix.indkey::int2[]) WITH ORDINALITY AS k(attnum, ord) ON TRUE
JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = k.attnum
WHERE n.nspname = current_schema()
  AND t.relname = $1
  AND t.relkind = 'r'
ORDER BY i.relname, ord
"#;

/// 把表名嵌入 PostgreSQL 查询（SQL 中固定使用 $1 占位），并转义单引号
fn pg_table_literal(table_name: &str) -> String {
    format!("'{}'", table_name.replace('\'', "''"))
}

/// codegen 阶段读取数据库表结构的抽象。
///
/// 为 `Pool<MySql>` 与 `Pool<Postgres>` 分别实现：PostgreSQL 的 introspection 结果会被
/// 归一化成与 MySQL 相同的中间结构（TableRow/TableFieldRow/...），
/// 因此下游实体/映射生成逻辑完全不需要关心具体方言。
#[doc(hidden)]
pub trait TableIntrospector {
    async fn get_tables(&self) -> Result<Vec<TableRow>, sqlx::Error>;
    async fn get_table_fields(&self, table_name: &str) -> Result<Vec<TableFieldRow>, sqlx::Error>;
    async fn get_table_full_fields(&self, table_name: &str) -> Result<Vec<TableFullFieldRow>, sqlx::Error>;
    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<TableIndexRow>, sqlx::Error>;
}

impl TableIntrospector for Pool<MySql> {
    async fn get_tables(&self) -> Result<Vec<TableRow>, sqlx::Error> {
        let rows = sqlx::query("SHOW TABLES").fetch_all(self).await?;
        Ok(rows.iter().map(parse_table_row_mysql).collect())
    }

    async fn get_table_fields(&self, table_name: &str) -> Result<Vec<TableFieldRow>, sqlx::Error> {
        let query = format!("DESCRIBE `{}`;", table_name);
        let rows = sqlx::query(&query).fetch_all(self).await?;
        Ok(rows.iter().map(parse_field_row_mysql).collect())
    }

    async fn get_table_full_fields(&self, table_name: &str) -> Result<Vec<TableFullFieldRow>, sqlx::Error> {
        let query = format!("SHOW FULL COLUMNS FROM `{}`;", table_name);
        let rows = sqlx::query(&query).fetch_all(self).await?;
        Ok(rows.iter().map(parse_full_field_row_mysql).collect())
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<TableIndexRow>, sqlx::Error> {
        let query = format!("SHOW INDEX FROM `{}`;", table_name);
        let rows = sqlx::query(&query).fetch_all(self).await?;
        let index_columns: Vec<TableIndexColumnRow> = rows.iter().map(parse_index_column_row_mysql).collect();
        Ok(collect_indexes(index_columns))
    }
}

impl TableIntrospector for Pool<Postgres> {
    async fn get_tables(&self) -> Result<Vec<TableRow>, sqlx::Error> {
        let rows = sqlx::query(PG_TABLES_SQL).fetch_all(self).await?;
        Ok(rows.iter().map(parse_table_row_pg).collect())
    }

    async fn get_table_fields(&self, table_name: &str) -> Result<Vec<TableFieldRow>, sqlx::Error> {
        let query = PG_TABLE_FIELDS_SQL.replace("$1", &pg_table_literal(table_name));
        let rows = sqlx::query(&query).fetch_all(self).await?;
        Ok(rows.iter().map(parse_field_row_pg).collect())
    }

    async fn get_table_full_fields(&self, table_name: &str) -> Result<Vec<TableFullFieldRow>, sqlx::Error> {
        let query = PG_TABLE_FIELDS_SQL.replace("$1", &pg_table_literal(table_name));
        let rows = sqlx::query(&query).fetch_all(self).await?;
        Ok(rows.iter().map(parse_full_field_row_pg).collect())
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<TableIndexRow>, sqlx::Error> {
        let query = PG_TABLE_INDEXES_SQL.replace("$1", &pg_table_literal(table_name));
        let rows = sqlx::query(&query).fetch_all(self).await?;
        let index_columns: Vec<TableIndexColumnRow> = rows.iter().map(parse_index_column_row_pg).collect();
        Ok(collect_indexes(index_columns))
    }
}

/// 按索引名分组。MySQL 中唯一索引的所有列 Non_unique 均为 0，非唯一索引的所有列均为 1；
/// PostgreSQL 侧查询已用 CASE 对齐该语义，因此两侧共用同一分组逻辑。
fn collect_indexes(index_columns: Vec<TableIndexColumnRow>) -> Vec<TableIndexRow> {
    let mut index_map: HashMap<String, (Vec<String>, bool)> = HashMap::new();
    for index_column in index_columns {
        let entry = index_map.entry(index_column.index_name.clone()).or_insert((vec![], !index_column.non_unique));
        entry.0.push(index_column.column_name);
        if index_column.non_unique {
            entry.1 = true;
        }
    }
    let mut indexes: Vec<TableIndexRow> = index_map.into_iter().map(|(index_name, (columns, non_unique))| {
        TableIndexRow {
            index_name,
            columns,
            non_unique,
        }
    }).collect();
    indexes.sort_by(|a, b| a.index_name.cmp(&b.index_name));
    indexes
}

// 索引名 -> 常量名: 仅改大写，保留下划线
pub(crate) fn to_screaming_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut first = true;
    for c in name.chars() {
        if c.is_uppercase() && !first {
            result.push('_');
        }
        if c.is_alphanumeric() || c == '_' {
            result.push(c.to_uppercase().next().unwrap());
        }
        first = false;
    }
    result
}

pub(crate) fn reserved_field_names() -> Vec<String> {
    vec![
        "as", "async", "await", "break", "const", "continue", "crate", "dyn",
        "else", "enum", "extern", "false", "fn", "for", "if", "impl",
        "in", "let", "loop", "match", "mod", "move", "mut", "pub",
        "ref", "return", "self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while"
    ].iter().map(|s| s.to_string()).collect()
}

pub(crate) fn get_simple_name(qualified_name: &str) -> String {
    if qualified_name.is_empty(){
        return "".to_string();
    } 
    let last_sep_index = qualified_name.rfind("::").unwrap_or(0);
    qualified_name[(last_sep_index + 1)..].to_string()
}

// Create parent directories if they do not exist yet.
pub fn prepare_directory(path:& std::path::Path){
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect(&format!("Failed to create directory {}", parent.display()));
        }
    }
    if path.is_dir() && !path.exists() {
        fs::create_dir_all(path).expect(&format!("Failed to create directory {}", path.display()));
    }
}

pub fn format_name(name:&str, convention: NamingConvention) -> String{
    match convention {
       NamingConvention::CamelCase => {
           stringUtils::to_camel_case(name)
       }
       NamingConvention::SnakeCase => {
            let mut result = String::new();
            let mut first = true;
            for c in name.chars() {
                if c.is_uppercase() && !first {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
                first = false;
            }
            result
        }
        NamingConvention::PascalCase => {
            let mut result = String::new();
            let mut capitalize_next = true;
            for c in name.chars() {
                if c.is_alphanumeric() {
                    if capitalize_next {
                        result.push(c.to_uppercase().next().unwrap());
                        capitalize_next = false;
                    } else {
                        result.push(c.to_lowercase().next().unwrap());
                    }
                } else {
                    capitalize_next = true;
                }
            }
            result
        }
    }
}