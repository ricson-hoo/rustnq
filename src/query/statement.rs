use std::collections::HashMap;
use uuid::uuid;
use crate::mapping::description::{Column, SqlColumn};
use crate::mapping::description::Table;
use crate::mapping::column_types::{Bigint, Date, Int, Varchar};
use crate::query::builder::{Condition, InnerTable, QueryBuilder, SelectField, TargetTable};
use serde::Serialize;
use sqlx::Error;
use sqlx::Row;
use tokio::sync::RwLock;
use crate::configuration::{get_processors, PROCESSORS};
use crate::query::builder::construct_upsert_primary_key_value;
pub use crate::utils::date_sub_unit::DateSubUnit;
use crate::query::dialect::DbDialect;
use crate::query::db::DbRow;
pub fn select<T: Into<SelectField>>(fields: Vec<T>) -> QueryBuilder{
    let fields = fields.into_iter().map(|field| field.into()).collect();
    QueryBuilder::init_with_select_fields(fields)
}


pub fn with(name: &str, builder: QueryBuilder) -> InnerTable {
    builder.cte(name)
}

pub fn select_distinct<T: Into<SelectField>>(fields: Vec<T>) -> QueryBuilder{
    let fields = fields.into_iter().map(|field| field.into()).collect();
    QueryBuilder::init_with_select_distinct_fields(fields)
}

pub fn sum<T: Into<SelectField>>(field:T) -> Bigint{
   Bigint::with_name(format!("sum({})", field.into().to_string()))
}

pub fn count<T: Into<SelectField>>(field:T) -> Bigint{
    Bigint::with_name(format!("count({})", field.into().to_string()))
}

pub fn count_all() -> Bigint{
    Bigint::with_name("count(*)".to_string())
}

pub fn count_distinct<T: Into<SelectField>>(field:T) -> Bigint{
    Bigint::with_name(format!("count(distinct {})", field.into().to_string()))
}

pub fn case_when<T: Into<SelectField>>(cases: Vec<(Condition, Option<T>)>, else_value: Option<T>) -> SelectField {
    let mut case = "CASE".to_string();

    // Add WHEN conditions
    for (condition, value) in cases {
        if let Some(val) = value {
            case.push_str(&format!(" WHEN {} THEN {}", condition, val.into().to_string()));
        } else {
            case.push_str(&format!(" WHEN {} THEN null", condition));
        }
    }

    // Add ELSE clause if provided
    if let Some(else_val) = else_value {
        case.push_str(&format!(" ELSE {}", else_val.into().to_string()));
    } else {
        case.push_str(" ELSE null");
    }

    case.push_str(" END");

    SelectField::UnsafeExpr(case)
}

pub fn union_all(sql_list: Vec<QueryBuilder>) -> QueryBuilder{
    let mut list: Vec<String> = vec![];
    for sql in sql_list {
        let sql_result = sql.build();
        if let Ok(sql_string) = sql_result {
            println!("sql string {}", sql_string);
            list.push(sql_string);
        }
    }
    let table = InnerTable{
        table_name: format!("({}) as my_table", list.join(" union all ")),
        fields_map: Default::default(),
        cte_query: None,
    };
    QueryBuilder::init_with_select_all_fields(&table)
}

pub fn exists(sql:QueryBuilder) -> Condition{
    Condition::new(format!("exists ({})", sql.build().unwrap_or_default()))
}

pub fn not_exists(sql:QueryBuilder) -> Condition{
    Condition::new(format!("not exists ({})", sql.build().unwrap_or_default()))
}

pub fn max<T: Into<SelectField>>(field:T) -> Varchar{
    Varchar::with_name(format!("max({})", field.into().to_string()))
}
pub fn min<T: Into<SelectField>>(field:T) -> Varchar{
    Varchar::with_name(format!("min({})", field.into().to_string()))
}
pub fn timestamp_diff<T: Into<SelectField>>(date: T, unit: DateSubUnit) -> Int{
    Int::with_name(DbDialect::current().timestamp_diff(&unit.to_string(), &date.into().to_string()))
}

pub fn curdate() -> Date{
    Date::with_name(DbDialect::current().curdate().to_string())
}

pub fn year<T: Into<SelectField>>(field:T) -> Varchar{
    Varchar::with_name(DbDialect::current().year(&field.into().to_string()))
}
pub fn year_int<T: Into<SelectField>>(field:T) -> Int{
    Int::with_name(DbDialect::current().year(&field.into().to_string()))
}


pub fn extract_year<T: Into<SelectField>>(field:T) -> Varchar{
    Varchar::with_name(format!("Extract(YEAR from {})", field.into().to_string()))
}

pub fn year_diff<T: Into<SelectField>>(field:T, another_year:i32) -> Int{
    Int::with_name(DbDialect::current().year_diff(another_year, &field.into().to_string()))
}

pub fn date<T: Into<SelectField>>(field:T) -> Varchar{
    Varchar::with_name(DbDialect::current().date(&field.into().to_string()))
}

pub fn month<T: Into<SelectField>>(field:T) -> Int{
    Int::with_name(DbDialect::current().month(&field.into().to_string()))
}

/// DATE_FORMAT(date, format) — date formatting (MySQL: DATE_FORMAT, PG: TO_CHAR)
pub fn date_format<T: Into<SelectField>>(field: T, fmt: &str) -> Varchar {
    Varchar::with_name(DbDialect::current().date_format(&field.into().to_string(), fmt))
}

/// SUBSTRING_INDEX(str, delim, count) — substring by delimiter (MySQL: SUBSTRING_INDEX, PG: SPLIT_PART)
pub fn substring_index<T: Into<SelectField>>(field: T, delimiter: &str, count: i32) -> Varchar {
    Varchar::with_name(DbDialect::current().substring_index(&field.into().to_string(), delimiter, count))
}

/// REPLACE(str, from_str, to_str) — string replace (with SQL injection protection)
pub fn str_replace<T: Into<SelectField>>(field: T, from_str: &str, to_str: &str) -> Varchar {
    Varchar::with_name(format!("REPLACE({}, '{}', '{}')", field.into().to_string(),
        from_str.replace("'", "''"), to_str.replace("'", "''")))
}

///DATE_SUB(date, INTERVAL value unit)
pub fn date_sub<D: Into<SelectField>>(date: D, value: i32, unit: DateSubUnit) -> Date{
    Date::with_name(DbDialect::current().date_sub(&date.into().to_string(), value, &unit.to_string()))
}

/// 本季度第一天
pub fn quarter_start() -> Varchar {
    Varchar::with_name(DbDialect::current().quarter_start())
}

/// 下季度第一天
pub fn quarter_end() -> Varchar {
    Varchar::with_name(DbDialect::current().quarter_end())
}

/// 本月第一天
pub fn month_start() -> Varchar {
    Varchar::with_name(DbDialect::current().month_start())
}

/// 下月第一天
pub fn month_end() -> Varchar {
    Varchar::with_name(DbDialect::current().month_end())
}

///DATE_ADD(date, INTERVAL value unit)
pub fn date_add<D: Into<SelectField>>(date: D, value: i32, unit: DateSubUnit) -> Date{
    Date::with_name(DbDialect::current().date_add(&date.into().to_string(), value, &unit.to_string()))
}

#[derive(Clone, Debug)]
pub enum CastType {
    Signed,
    Unsigned,
    Char(Option<u32>),
    Decimal(u32, u32),
    Double,
    DateTime,
    Date,
}

impl CastType {
    pub(crate) fn to_sql(&self) -> String {
        let is_pg = crate::query::dialect::DbDialect::current().is_postgres();
        match self {
            CastType::Signed => if is_pg { "INTEGER".to_string() } else { "SIGNED".to_string() },
            CastType::Unsigned => if is_pg { "INTEGER".to_string() } else { "UNSIGNED".to_string() },
            CastType::Char(None) => "CHAR".to_string(),
            CastType::Char(Some(n)) => format!("CHAR({})", n),
            CastType::Decimal(p, s) => format!("DECIMAL({},{})", p, s),
            CastType::Double => "DOUBLE".to_string(),
            CastType::DateTime => "DATETIME".to_string(),
            CastType::Date => "DATE".to_string(),
        }
    }
}

/// CAST(expr AS type) — type-safe cast
pub fn cast_as<T: Into<SelectField>>(expr: T, cast_type: CastType) -> SelectField {
    SelectField::UnsafeExpr(format!("CAST({} AS {})", expr.into().to_string(), cast_type.to_sql()))
}

/// COALESCE(expr, default) — returns first non-null argument
pub fn coalesce<T: Into<SelectField>, U: Into<SelectField>>(expr: T, default: U) -> SelectField {
    SelectField::UnsafeExpr(format!("COALESCE({}, {})", expr.into().to_string(), default.into().to_string()))
}

/// CAST(expr AS target_type) — legacy string-based cast
pub fn cast<T: Into<SelectField>>(expr: T, target_type: &str) -> SelectField {
    SelectField::UnsafeExpr(format!("CAST({} AS {})", expr.into().to_string(), target_type))
}

/// UNIX_TIMESTAMP / EXTRACT(EPOCH FROM ...) — epoch seconds
pub fn unix_timestamp<T: Into<SelectField>>(field: T) -> Int {
    Int::with_name(DbDialect::current().unix_timestamp(&field.into().to_string()))
}

/// epoch milliseconds (MySQL: UNIX_TIMESTAMP*1000, PG: EXTRACT(EPOCH)*1000)
pub fn unix_timestamp_ms<T: Into<SelectField>>(field: T) -> Bigint {
    Bigint::with_name(DbDialect::current().unix_timestamp_ms(&field.into().to_string()))
}

/// RAND() / RANDOM() — random value
pub fn rand() -> SelectField {
    SelectField::RawValue(DbDialect::current().rand().to_string())
}

/// DATEDIFF / date subtraction — date difference in days
pub fn datediff<T: Into<SelectField>, U: Into<SelectField>>(date1: T, date2: U) -> Int {
    Int::with_name(DbDialect::current().datediff(&date1.into().to_string(), &date2.into().to_string()))
}

/// FLOOR(expr)
pub fn floor<T: Into<SelectField>>(expr: T) -> SelectField {
    SelectField::UnsafeExpr(format!("FLOOR({})", expr.into().to_string()))
}

/// REPLACE(field, from_str, to_str)
pub fn replace<T: Into<SelectField>>(field: T, from_str: &str, to_str: &str) -> SelectField {
    SelectField::UnsafeExpr(format!("REPLACE({}, '{}', '{}')", field.into().to_string(), 
        from_str.replace("'", "''"), to_str.replace("'", "''")))
}

/// NULLIF(expr1, expr2)
pub fn nullif<T: Into<SelectField>, U: Into<SelectField>>(expr1: T, expr2: U) -> SelectField {
    SelectField::UnsafeExpr(format!("NULLIF({}, {})", expr1.into().to_string(), expr2.into().to_string()))
}

/// 本周第一天
pub fn week_start() -> Varchar {
    Varchar::with_name(DbDialect::current().week_start())
}

/// 下周日
pub fn week_end() -> Varchar {
    Varchar::with_name(DbDialect::current().week_end())
}

/// 明日
pub fn tomorrow() -> Date {
    Date::with_name(DbDialect::current().tomorrow())
}

pub fn group_concat<T: Into<SelectField>>(fields: Vec<T>) -> Varchar{
    let fields_str = fields.into_iter().map(|field| field.into().to_string()).collect::<Vec<String>>().join(",");
    Varchar::with_name(DbDialect::current().group_concat(&fields_str))
}

pub fn concat<T: Into<SelectField>>(fields: Vec<T>) -> Varchar{
    let fields_str = fields.into_iter().map(|field| field.into().to_string()).collect::<Vec<String>>().join(",");
    Varchar::with_name(DbDialect::current().concat(&fields_str))
}

pub async fn insert_or_update<A,T: Serialize + for<'de> serde::Deserialize<'de>>(table_with_value: &mut A) -> Result<T,Error> where A : Table{
    let target_table:TargetTable = TargetTable::new(table_with_value);
    //let mut multiple_typed_primary_key = MultiTypedPrimaryKey{uuid_key:None,i32_key:None,i64_key:None,u64_key:None};
    let primary_key_vec = table_with_value.primary_key();
    if primary_key_vec.len()<1{
        Err(sqlx::Error::ColumnNotFound("no primary key column is found".to_string()))?
    }
    let mut text_primary_key_value:Option<String> = None;
    let mut text_primary_key_name:Option<String> = None;
    if primary_key_vec.len() == 1 {
        let primary_key = primary_key_vec.get(0).unwrap().clone();
        match primary_key {
            SqlColumn::Varchar(optional_column_info) => { //uuid
                if let Some(column_info) = optional_column_info {
                    text_primary_key_name = Some(column_info.name());
                    let primary_column_value = column_info.value();
                    if let None = primary_column_value {
                        let generated_uuid_key = Some(uuid::Uuid::new_v4().to_string().replace("-", "").to_string());
                        table_with_value.update_primary_key(vec![SqlColumn::Varchar(Some(Varchar::with_name_value(column_info.name(),generated_uuid_key.clone())))]);
                        text_primary_key_value = generated_uuid_key;
                    } else {
                        text_primary_key_value = primary_column_value;
                    }

                }
            }
            _ => {
                //Err(sqlx::Error::Encode("unsupported primary key".into()))?
            }
        }
    }

    //
    let mut condition= Condition::new("1 = 1".to_string());
    if table_with_value.primary_key().len()>1 || text_primary_key_value.is_some(){
        // 复合主键 / 字符串(uuid)主键：upsert 后按主键条件回查
        let upsert_result = QueryBuilder::upsert_table_with_value(table_with_value).execute().await;
        match upsert_result {
            Ok(query_result) => {
                if query_result.rows_affected() > 0 {
                    if text_primary_key_value.is_some(){
                        condition = condition.and(Condition::new(format!("{} = '{}'",&text_primary_key_name.unwrap(),&text_primary_key_value.unwrap_or_default())))
                    }else{
                        let mut primary_key_as_conditions = vec![];
                        construct_upsert_primary_key_value(&table_with_value.primary_key(), &mut vec![], &mut vec![], &mut primary_key_as_conditions);
                        for cond in primary_key_as_conditions {
                            condition = condition.and(Condition::new(cond))
                        }
                    }
                } else {
                    return Err(sqlx::Error::RowNotFound)?;
                }
            }
            Err(error) => {
                return Err(error)
            }
        }
    } else {
        // 数字主键：PostgreSQL 用 RETURNING 取回；MySQL 用 LAST_INSERT_ID() + SELECT 回查
        let primary_key = primary_key_vec.get(0).unwrap().clone();
        let primary_key_name = primary_key.get_col_name();
        match QueryBuilder::upsert_table_with_value(table_with_value).execute_returning(vec![primary_key_name.clone()]).await {
            Ok(Some(row)) => {
                let id = match &row {
                    DbRow::Mysql(r) => r.try_get::<Option<i64>, _>(0),
                    DbRow::Postgres(r) => r.try_get::<Option<i64>, _>(0),
                };
                if let Ok(Some(id)) = id {
                    if id > 0 {
                        condition = condition.and(Condition::new(format!("{} = {}", primary_key_name, id)));
                    }else{
                        // RETURNING 未取到有效主键：按主键条件回查
                        let mut primary_key_as_conditions = vec![];
                        construct_upsert_primary_key_value(&table_with_value.primary_key(), &mut vec![], &mut vec![], &mut primary_key_as_conditions);
                        for cond in primary_key_as_conditions {
                            condition = condition.and(Condition::new(cond))
                        }
                    }
                }else{
                    let mut primary_key_as_conditions = vec![];
                    construct_upsert_primary_key_value(&table_with_value.primary_key(), &mut vec![], &mut vec![], &mut primary_key_as_conditions);
                    for cond in primary_key_as_conditions {
                        condition = condition.and(Condition::new(cond))
                    }
                }
            }
            Ok(None) => {
                return Err(sqlx::Error::RowNotFound)?;
            }
            Err(error) => {
                return Err(error)
            }
        }
    }
    let result = QueryBuilder::select_all_fields().from(table_with_value).where_(condition).fetch_one().await;
    match result {
        Ok(row) => {
            match row {
                Some(row) => {
                    return Ok(row)
                }
                None => {
                    return Err(sqlx::Error::RowNotFound)
                }
            }
        },
        Err(error) => {
            Err(error)
        }
    }
}

pub fn update<A>(table:& A) -> QueryBuilder where A : Table{
    QueryBuilder::update(table)
}

pub fn delete_one_from<A>(table:& A) -> QueryBuilder where A : Table{
    QueryBuilder::delete_one_from(table)
}

pub fn delete_one_where<A>(table:& A, condition: Condition) -> QueryBuilder where A : Table{
    QueryBuilder::delete_one_where(table, condition)
}

pub fn delete_all_where<A>(table:& A, condition: Condition) -> QueryBuilder where A : Table{
    QueryBuilder::delete_all_where(table, condition)
}

/*pub fn insert_into<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}

pub fn update<'a,A>(table:&'a A) -> QueryBuilder<'a> where A : Table{

}*/