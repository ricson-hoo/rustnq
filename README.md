# rustnq

A type-safe SQL query framework in Rust supporting you to write SQL queries in this way:

    let query = select(vec![&product.id, &product.moq])
    .from(&product.table)
    .where_(product.manufacturer_id.equal("123").and((product.type_id.equal(type.id).or(product.category_id.equal(category.id)))));

You can generate the table mapping and entity(struct) from your main.rs(or build.rs) using:

    rustnq::codegen::entity::generate_entities(&pool, "db_name_here", entityGenConfig).await;
    rustnq::codegen::mapping::generate_mappings(&pool, "db_name_here", mappingGenConfig).await;

refs to examples/generated for generated code.