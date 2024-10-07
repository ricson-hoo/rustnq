use std::println;
use std::path::{Path};
use sqlx::{AnyConnection, AnyPool, Pool};
use sqlx_mysql::MySql;
use crate::codegen::utils;

//generate table mappings to db & table definitions
pub async fn generate_mappings(conn: &Pool<MySql>, db_name:&str, output_path:&Path){
    let table_definitions = utils::get_tables(conn/*, db_name*/).await;
    println!("{:#?}",table_definitions);
}