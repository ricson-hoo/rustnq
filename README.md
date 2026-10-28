# rustnq

A jOOQ-style type-safe SQL query builder for Rust. Compile-time SQL validation, zero string concatenation — supports MySQL and PostgreSQL.

```rust
let query = select(vec![&product.id, &product.moq])
    .from(&product.table)
    .where_(product.manufacturer_id.equal("123")
        .and(product.type_id.equal(type.id)
            .or(product.category_id.equal(category.id))));
```

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
rustnq = { path = "../rustnq" }
```

Initialize the connection pool:

```rust
use rustnq::query::pool::init_pools;
use rustnq::query::dialect::DbDialect;

#[derive(Debug, Clone)]
struct AppDbConf { url: String, dialect: DbDialect }
impl DbConf for AppDbConf {
    fn url(&self) -> &str { &self.url }
    fn dialect(&self) -> DbDialect { self.dialect }
}

init_pools(vec![AppDbConf {
    url: "mysql://user:pass@localhost:3306/mydb".to_string(),
    dialect: DbDialect::MySql,
}]).await;
```

Your first query:

```rust
let user = select(vec![
    user_table.id.clone(),
    user_table.user_name.clone(),
])
.from(&user_table)
.where_(user_table.id.equal("123".to_string()))
.fetch_one::<UserDto>().await?;
```

## Core Concepts

### Type Hierarchy

```
Table (trait)
  ├── Custom MapTable struct → implements Table, typed Columns
  └── InnerTable           → returned by as_table(), has fields_map

Column (trait)
  ├── Varchar / Int / Bigint / Tinyint / Smallint
  ├── Date / Datetime / Time / Timestamp
  ├── Boolean / Decimal / Float / Double
  ├── Enum<T> / Set<T> / Array<T>
  ├── Text / Longtext / Tinytext / Char
  └── Json

Condition     → WHERE / ON / HAVING clauses (chainable .and() / .or())
QueryBuilder  → full SQL query constructor
SelectField   → SELECT item (Column, subquery, expression, raw value)
```

### Aliasing

```rust
let col = user_table.id.as_("userId");          // column alias
let u = UserTable::new(Some("u"));              // table alias → FROM user AS u
```

## Table Mapping (MapTable)

### Structure

```rust
use rustnq::mapping::description::{Table, Column as ColTrait, SqlColumn};
use rustnq::mapping::column_types::{Varchar, Int, Date, Enum};

#[derive(Clone, Debug)]
pub struct UserTable {
    pub id: Varchar,
    pub user_name: Varchar,
    pub sex: Enum<UserSex>,
    pub birthday: Date,
    pub community_id: Varchar,
    pub is_disabled: Int,
    _primary_key: Vec<SqlColumn>,
    _alias: Option<String>,
}
```

### Constructor

```rust
impl UserTable {
    pub fn new(alias: Option<&str>) -> Self {
        let table_name = alias.unwrap_or("user");
        UserTable {
            id: Varchar::with_qualified_name(table_name.to_string(), "id".to_string()),
            user_name: Varchar::with_qualified_name(table_name.to_string(), "user_name".to_string()),
            _primary_key: vec![SqlColumn::Varchar(Some(
                Varchar::with_name("id".to_string())
            ))],
            _alias: alias.map(|s| s.to_string()),
        }
    }
}
```

Key constructors: `Varchar::with_qualified_name(table, col)`, `Int::with_qualified_name(table, col)`, `Date::with_qualified_name(table, col)`.

### Table Trait

```rust
impl Table for UserTable {
    fn name(&self) -> String { self._alias.clone().unwrap_or("user".to_string()) }
    fn all_columns(&self) -> Vec<SqlColumn> { vec![
        SqlColumn::Varchar(Some(self.id.clone())),
        SqlColumn::Varchar(Some(self.user_name.clone())),
    ]}
    fn primary_key(&self) -> Vec<SqlColumn> { self._primary_key.clone() }
    fn update_primary_key(&mut self, pk: Vec<SqlColumn>) { self._primary_key = pk; }
}
```

## Conditions

### Basic Comparisons

```rust
use rustnq::query::builder::Condition;

column.equal("value".to_string())               // col = 'value'
column.equal(other_column.clone())              // col = other_col (column-to-column)
column.not_equal("value".to_string())           // col != 'value'
column.ne("value".to_string())                  // same as above
column.ge(60) / column.gt(60)                  // >= / >
column.le(100) / column.lt(100)                // <= / <
column.is_null() / column.is_not_null()         // IS NULL / IS NOT NULL
column.like("%term%".to_string())               // LIKE '%term%'
column.not_like("%test%".to_string())           // NOT LIKE '%test%'

column.in_(vec!["a".to_string(), "b".to_string()])              // IN ('a','b')
column.not_in(vec!["x".to_string()])                            // NOT IN ('x')
column.in_query_result(subquery)                                // IN (SELECT ...)
column.not_in_query_result(subquery)                            // NOT IN (SELECT ...)
```

### Boolean Combinations

```rust
Condition::true_cond()                           // 1=1 — safe starting point
Condition::false_cond()                          // 0>1 — always false

condition.and(other_cond)
condition.or(other_cond)
condition.and_exists(subquery)                   // AND EXISTS (...)
condition.and_not_exists(subquery)               // AND NOT EXISTS (...)
```

> **Do NOT use `Condition::unsafe_raw()`** — it bypasses type safety and introduces SQL injection risk.

## Query Building

### SELECT

```rust
use rustnq::query::select;

let results = select(vec![
    table.id.clone().into(),
    table.name.clone().as_("displayName"),
])
.from(&table)
.where_(table.community_id.equal("org123".to_string()))
.order_by(vec![table.id.desc()])
.limit(10)
.fetch::<MyDto>().await?;
```

### JOIN

```rust
select(fields).from(&main)
    .inner_join(&join_table).on(main.fk.equal(join_table.id.clone()))
    .left_join(&dim_table).on(main.dim_id.equal(dim_table.id.clone()))
    .straight_join(&st_table).on(main.id.equal(st_table.ref_id.clone()))
```

### GROUP BY / ORDER BY

```rust
select(vec![
    table.dept.clone(),
    count(table.id.clone()).as_("cnt"),
])
.from(&table)
.group_by(vec![table.dept.clone()])
.order_by(vec![table.dept.asc()])
```

### Pagination

```rust
select(fields).from(&table).where_(cond)
    .limit_offset(page_size, (page - 1) * page_size)
    .fetch::<Dto>().await?;

// Parallel pagination: auto COUNT + data in one call
let paging: PagingData<Dto> = select(fields).from(&table).where_(cond)
    .limit_offset(page_size, (page - 1) * page_size)
    .fetch_paging_parallel::<Dto>().await?;
// paging.data, paging.current_page, paging.page_size, paging.total_count
```

### Single Row

```rust
let item: Option<Dto> = select(fields).from(&t).where_(cond).fetch_one().await?;
```

### Subquery as Field

```rust
select(vec![
    user.id.clone().into(),
    rustnq::query::select(vec![count(followup.id.clone())])
        .from(&followup)
        .where_(followup.user_id.equal(user.id.clone()))
        .as_("followupCount"),
]);
```

### Force Index

```rust
select(fields).from(&t).force_index("user_id").where_(cond)
```

### Count

```rust
let total: i64 = select(vec![count_all()]).from(&t).where_(cond).fetch_count().await?;
```

## Derived Tables & EXISTS

Three-step pattern: **build subquery → as_table → access via fields_map**.

### Basic Derived Table

```rust
let sub = select(vec![
    hf.user_id.clone(),
    count(hf.id.clone()).as_("followup_count"),  // alias is key!
])
.from(&hf)
.where_(hf.type_.equal(followup_type))
.group_by(vec![hf.user_id.clone()])
.as_table("sub");    // (SELECT ...) AS sub

let followup_col = sub.fields_map["followup_count"].clone();

let result = select(vec![count_all()])
    .from(&sub)
    .where_(followup_col.equal(1))
    .fetch_one::<i64>().await?;
```

### fields_map Key Rules

| SELECT field | Key in fields_map |
|---|---|
| `col.as_("alias")` | `"alias"` |
| `col` (no alias) | `"table.col"` (from `SelectField::to_string()`) |
| `subquery.as_("alias")` | `"alias"` |

### EXISTS Subquery

```rust
let cond = Condition::true_cond().and_exists(
    select(vec![SelectField::from(1)])
        .from(&sub_table)
        .where_(sub_table.id.equal(main_table.id.clone()))
);

let cond = Condition::true_cond().and_not_exists(
    select(vec![SelectField::from(1)])
        .from(&sub_table)
        .where_(condition)
);
```

### EXISTS + Derived Table (HAVING-equivalent)

```rust
let sub = select(vec![
    hf.user_id.clone(),
    count(hf.id.clone()).as_("followup_count"),
])
.from(&hf)
.group_by(vec![hf.user_id.clone()])
.as_table("sub");

let col = sub.fields_map["followup_count"].clone();
condition = condition.and_exists(
    select(vec![SelectField::from(1)])
        .from(&sub)
        .where_(col.equal(2))
);
```

### Derived Table as JOIN Source

```rust
select(fields)
    .from(&main_table)
    .left_join(&sub_table)
    .on(sub_table.fields_map["user_id"].clone()
        .equal(main_table.user_id.clone()))
```

### IN (Subquery)

```rust
let diabetes_users = select(vec![hm.user_id.clone()])
    .from(&hm)
    .where_(hm.type_.equal(HealthManageType::Diabetes));

condition = condition.and(cr.user_id.in_query_result(diabetes_users));
```



## CTE (Common Table Expressions)

Use `with()` to define a CTE, then join it like any `InnerTable`. CTEs are supported on MySQL 8.0+ and PostgreSQL.

```rust
use rustnq::query::with;

let user_stats = with("user_stats", select(vec![
    u.id.clone(),
    count(o.id.clone()).as_("order_count"),
])
.from(&u)
.left_join(&o).on(u.id.equal(o.user_id.clone()))
.group_by(vec![u.id.clone()]));

let col = user_stats.fields_map["order_count"].clone();

select(vec![u.id.clone(), u.name.clone(), col.clone()])
    .from(&u)
    .left_join(&user_stats).on(u.id.equal(user_stats.fields_map["id"].clone()))
    .order_by(vec![col.desc()]);
```

Generated SQL:

```sql
WITH user_stats AS (
    SELECT u.id, COUNT(o.id) AS order_count
    FROM user u LEFT JOIN order o ON u.id = o.user_id
    GROUP BY u.id
)
SELECT u.id, u.name, user_stats.order_count
FROM user u LEFT JOIN user_stats ON u.id = user_stats.id
ORDER BY user_stats.order_count DESC
```

> **Note:** For PostgreSQL-only `FILTER` clause or `ARRAY_AGG`, use `SelectField::UnsafeExprAliased` within the CTE`s SELECT fields.

## Aggregation & Expressions

### Aggregates

```rust
use rustnq::query::{count, count_all, count_distinct, sum, max, min, group_concat};

count_all()                                       // COUNT(*)
count(table.id.clone())                           // COUNT(col)
count_distinct(table.user_id.clone())             // COUNT(DISTINCT col)
sum(table.amount.clone())                         // SUM(col)
max(table.date.clone()) / min(table.date.clone())

count_distinct(table.uid.clone()).as_("cnt")     // with alias
```

### CASE WHEN

```rust
use rustnq::query::case_when;

// Simple
case_when(
    vec![
        (table.status.equal("normal"), Some("Normal")),
        (table.status.equal("abnormal"), Some("Abnormal")),
    ],
    Some("Unknown"),   // ELSE
);

// COUNT + CASE WHEN (conditional aggregation)
count_distinct(case_when(
    vec![(cond1.and(cond2), Some(table.user_id.clone()))],
    None,              // ELSE NULL → not counted
)).as_("q1_count")
// → COUNT(DISTINCT CASE WHEN cond1 AND cond2 THEN user_id ELSE NULL END) AS q1_count
```

### Date Functions

```rust
use rustnq::query::{year, year_int, month, date, date_add, date_sub, timestamp_diff, curdate, year_diff, date_format};
use rustnq::utils::date_sub_unit::DateSubUnit;

year(table.date.clone())                          // YEAR(col) → Varchar
year_int(table.date.clone())                      // YEAR(col) → Int
month(table.date.clone())                         // MONTH(col) → Int
date(table.created_at.clone())                    // DATE(col)
date_add(table.start.clone(), 14, DateSubUnit::DAY)// DATE_ADD(col, INTERVAL 14 DAY)
date_sub(table.end.clone(), 1, DateSubUnit::MONTH) // DATE_SUB(col, INTERVAL 1 MONTH)
timestamp_diff(table.birthday.clone(), DateSubUnit::YEAR) // TIMESTAMPDIFF(YEAR, birthday, NOW())
curdate()                                         // CURDATE()
year_diff(table.birthday.clone(), 2025)           // YEAR diff
date_format(table.date.clone(), "%Y-%m")          // DATE_FORMAT
```

### Math & String

```rust
use rustnq::query::{floor, concat, substring_index, str_replace, coalesce, cast_as, CastType};

floor(table.amount.clone())                       // FLOOR(col)
concat(vec![table.a.clone(), table.b.clone()])    // CONCAT(a, b)
substring_index(table.path.clone(), "/", 2)       // SUBSTRING_INDEX(col, '/', 2)
str_replace(table.content.clone(), "old", "new")  // REPLACE(col, 'old', 'new')
coalesce(table.nick.clone(), table.name.clone())  // COALESCE(nick, name)

// Arithmetic on Varchar
table.end.sub_(table.start.clone())               // col1 - col2
table.price.multiply(2)                           // col * 2
table.price.multiply_field(&table.qty)            // col1 * col2
table.total.div(100)                              // col DIV 100
table.price.round(2)                              // ROUND(col, 2)
```

### Other Expressions

```rust
use rustnq::query::{rand, nullif, unix_timestamp, datediff, quarter_start, quarter_end, month_start, month_end, week_start, week_end, tomorrow};

rand()                                            // RAND()
nullif(table.val.clone(), 0)                      // NULLIF(val, 0)
datediff(table.end.clone(), table.start.clone())  // DATEDIFF(end, start)
quarter_start() / quarter_end()                   // first day of current/next quarter
month_start() / month_end()                       // first day of current/next month
week_start() / week_end()                         // Monday / next Monday
tomorrow()                                        // CURDATE() + 1
```

## Type Casting & NULL

### Two Forms of CAST

**Form 1: Function-style (for SELECT fields)**

`cast_as(expr, type)` returns `SelectField` — use with aggregates and expressions.

```rust
cast_as(sum(case_when(conditions, Some(1))), CastType::Signed)
cast_as(table.price.clone(), CastType::Decimal(10, 2))
cast_as(table.date_str.clone(), CastType::Date)
```

**Form 2: Method chain `.cast_as()` (for WHERE conditions)**

Returns the target column type, chainable with `.ge()`, `.equal()`, etc.

```rust
table.value.cast_as::<Varchar>(CastType::Decimal(10, 2)).ge(60)
table.age.cast_as::<Int>(CastType::Signed).equal(30)
```

### CastType Variants

| CastType | SQL |
|---|---|
| `Signed` | `SIGNED` |
| `Unsigned` | `UNSIGNED` |
| `Decimal(p, s)` | `DECIMAL(10, 2)` |
| `Double` | `DOUBLE` |
| `Date` | `DATE` |
| `DateTime` | `DATETIME` |
| `Char(None)` | `CHAR` |
| `Char(Some(n))` | `CHAR(32)` |

### Common Patterns

```rust
// CAST(SUM(CASE WHEN ... THEN 1 ELSE 0 END) AS SIGNED)
cast_as(sum(case_when(vec![(cond, Some(1))], Some(0))), CastType::Signed)

// COALESCE(CAST(SUM(...) AS SIGNED), 0)
coalesce(cast_as(sum(case_when(vec![(cond, Some(uid))], None)), CastType::Signed), 0)

// CASE WHEN cond THEN col ELSE NULL END
case_when(vec![(cond, Some(col.clone()))], None)
```

## CRUD

### INSERT

**Method 1: from an Entity**

```rust
let table = MyTable::new_with_value(entity, None);
QueryBuilder::insert_into_table_with_value(&table).execute().await?;
```

**Method 2: manual field assignment**

```rust
let mut record = MyTable::new(None);
record.id = "xxx".to_string().into();
record.name = "Alice".to_string().into();
record.status = MyStatus::Active;

QueryBuilder::insert_into_table_with_value(&record).execute().await?;
```

### Upsert

```rust
let result = insert_or_update::<MyTable, MyEntity>(&mut record).await?;
```

### UPDATE

```rust
use rustnq::query::update;

update(&table)
    .set(table.status, "done".to_string())
    .where_(table.id.equal("123".to_string()))
    .execute().await?;

// By primary key
QueryBuilder::update_table_with_value(&record).execute().await?;
```

### DELETE

```rust
use rustnq::query::delete_one_where;

delete_one_where(&table, table.id.equal("123".to_string())).execute().await?;
QueryBuilder::delete_all_where(&table, table.status.equal("expired".to_string())).execute().await?;
```

## Multi-Database Connection Pool

### Design

Use a `Db` enum implementing `DbConf`:

```rust
#[derive(Clone)]
enum Db {
    Main     { url: String, encryptor: Option<Arc<dyn Encryptor>> },
    Readonly { url: String, encryptor: Option<Arc<dyn Encryptor>> },
}
```

- **No `.on_pool()`** → default to first registered pool (Main, read-write).
- **`.on_pool(Db::Readonly)`** → readonly pool.
- When Readonly URL = Main URL, they share the same physical pool.

### Initialization

```rust
let mut pools: Vec<Db> = Vec::new();
pools.push(Db::Main { url: main_url, encryptor: Some(main_encryptor) });
pools.push(Db::Readonly {
    url: db_b_url.unwrap_or(main_url),  // dev: fallback to main
    encryptor: db_b_encryptor,
});
init_pools(pools).await;
```

### Usage

```rust
// Write queries → default Main
insert(&table, &fields).execute().await?;

// Read-heavy queries → readonly pool
let result = select(all_columns())
    .from(&table)
    .condition(condition)
    .on_pool(Db::Readonly)
    .fetch_all().await?;
```

### Write Protection

INSERT / UPDATE / DELETE on Readonly pool → rejected at application layer. Backed by MySQL `SET SESSION transaction_read_only = ON`.

### Dev Single-DB Fallback

When `db_b.url` is not configured, Readonly inherits Main URL. `.on_pool(Db::Readonly)` code runs identically in dev and production.

## Code Generation

Auto-generate Entity structs and MapTable from `INFORMATION_SCHEMA`.

```rust
use rustnq::codegen::entity::{generate_entities, EntityGenerateConfig, NamingConvention};
use rustnq::codegen::mapping::{generate_mappings, MappingGenerateConfig};

let entity_cfg = EntityGenerateConfig {
    output_dir: "target/generated/entity".into(),
    crate_path_of_entity: "shared::entity".into(),
    naming_convention: NamingConvention::CamelCase,
    boolean_columns: {
        let mut m = HashMap::new();
        m.insert("user".into(), vec!["is_qualified".into(), "is_disabled".into()]);
        m
    },
    encrypted_columns: {
        let mut m = HashMap::new();
        m.insert("user".into(), vec!["user_name".into(), "id_card".into()]);
        m
    },
    trait_for_enum_types: {
        let mut m = HashMap::new();
        m.insert("role_permission*".into(), "PermissionType".into());
        m
    },
    ..Default::default()
};

let mapping_cfg = MappingGenerateConfig {
    output_dir: "target/generated/mapping".into(),
    ..Default::default()
};

generate_entities(&pool, "db_name", entity_cfg).await?;
generate_mappings(&pool, "db_name", mapping_cfg).await?;
```

Or via CLI in your project:

```bash
cargo run --bin mapping_gen
```

### Config Options

| Option | Description |
|---|---|
| `boolean_columns` | Map `TINYINT(1)` to `bool` instead of `i8` |
| `encrypted_columns` | Mark columns as encrypted |
| `trait_for_enum_types` | Bind enum traits to columns (supports `table*` wildcard) |
| `naming_convention` | `CamelCase` or `SnakeCase` |
| `partial_eq_tables` | Exclude tables from full `PartialEq` |

## Anti-Patterns

### ❌ Never Do

1. **`Condition::unsafe_raw()`** — bypasses type safety:
   ```rust
   // ❌ Condition::unsafe_raw(format!("{} = {}", col, val))
   // ✅ column.equal(val)
   ```

2. **`SelectField::Untyped()`** for SQL expressions:
   ```rust
   // ❌ SelectField::Untyped("COUNT(DISTINCT user_id)".to_string())
   // ✅ count_distinct(table.user_id.clone())
   ```

3. **Aggregates inside EXISTS** — `COUNT()` always returns 1 row:
   ```rust
   // ❌ exists(select(vec![count(table.id.clone())]).from(&table))   // Always TRUE!
   // ✅ exists(select(vec![SelectField::from(1)]).from(&table))
   ```

4. **Hardcoding database names**:
   ```rust
   // ❌ Condition::new("user_id IN (SELECT id FROM dev_db.users)".into())
   // ✅ table.user_id.in_query_result(subquery)
   ```

### ✅ Best Practices

- Use `.as_()` for aliases, never manual string formatting
- Use `.asc()` / `.desc()` for ordering, never string `"col DESC"`
- Column types are lightweight — clone freely

## Method Reference

### Varchar / Int

| Method | SQL |
|---|---|
| `.equal(v)` | `col = v` |
| `.not_equal(v)` / `.ne(v)` | `col != v` |
| `.ge(n)` / `.gt(n)` / `.le(n)` / `.lt(n)` | `col >= n` etc. |
| `.is_null()` / `.is_not_null()` | `IS NULL` / `IS NOT NULL` |
| `.like(p)` / `.not_like(p)` | `LIKE p` / `NOT LIKE p` |
| `.in_(list)` / `.not_in(list)` | `IN (...)` / `NOT IN (...)` |
| `.in_query_result(q)` / `.not_in_query_result(q)` | `IN (SELECT...)` |
| `.as_("a")` | `col AS a` |
| `.asc()` / `.desc()` | `ASC` / `DESC` |
| `.sub_(other)` | `col - other` |
| `.multiply(n)` / `.multiply_field(c)` | `col * n` / `col * c` |
| `.div(n)` / `.round(n)` | `col DIV n` / `ROUND(col, n)` |
| `.cast_as::<T>(CastType)` | `CAST(col AS type)` |

### Condition

| Method | SQL |
|---|---|
| `Condition::true_cond()` | `1=1` |
| `Condition::false_cond()` | `0>1` |
| `.and(c)` / `.or(c)` | `AND c` / `OR c` |
| `.and_exists(q)` | `AND EXISTS (q)` |
| `.and_not_exists(q)` | `AND NOT EXISTS (q)` |

### QueryBuilder

| Method | Description |
|---|---|
| `.from(&table)` | FROM clause |
| `.inner_join(&t).on(c)` / `.left_join(&t).on(c)` | JOIN |
| `.straight_join(&t).on(c)` | STRAIGHT_JOIN (MySQL) |
| `.where_(c)` | WHERE clause |
| `.group_by(v)` | GROUP BY |
| `.order_by(v)` | ORDER BY |
| `.having(c)` | HAVING |
| `.limit(n)` | LIMIT n |
| `.limit_offset(n, o)` | LIMIT n OFFSET o |
| `.force_index(name)` | FORCE INDEX |
| `.as_("alias")` | aliased subquery |
| `.as_table("name")` | derived table → InnerTable |
| `.on_pool(pool)` | target pool selection |
| `.build()` | generate SQL string |
| `.fetch::<T>()` / `.fetch_one::<T>()` | execute & map rows |
| `.fetch_count()` | COUNT query |
| `.fetch_paging_parallel::<T>()` | auto COUNT + pagination |
