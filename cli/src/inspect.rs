pub fn database(connection: &elephantry::Connection) -> crate::Result {
    let mut table = term_table::Table::new();
    table.style = term_table::TableStyle::rounded();

    table.add_row(term_table::row::Row::new(vec![
        term_table::table_cell::TableCell::new("name"),
        term_table::table_cell::TableCell::new("oid"),
        term_table::table_cell::TableCell::new("relations"),
        term_table::table_cell::TableCell::new("comment"),
    ]));

    for schema in elephantry::inspect::database(connection)?.iter() {
        table.add_row(term_table::row::Row::new(vec![
            term_table::table_cell::TableCell::new(&schema.name),
            term_table::table_cell::TableCell::new(&schema.oid),
            term_table::table_cell::TableCell::new(&schema.relations),
            term_table::table_cell::TableCell::new(&schema.comment),
        ]));
    }

    println!("{}", table.render());

    Ok(())
}

pub fn schema(connection: &elephantry::Connection, schema: &str) -> crate::Result {
    let relations = elephantry::inspect::schema(connection, schema)?;
    let mut table = term_table::Table::new();
    table.style = term_table::TableStyle::rounded();

    table.add_row(term_table::row::Row::new(vec![
        term_table::table_cell::TableCell::new("name"),
        term_table::table_cell::TableCell::new("type"),
        term_table::table_cell::TableCell::new("oid"),
        term_table::table_cell::TableCell::new("comment"),
    ]));

    for relation in relations.iter() {
        table.add_row(term_table::row::Row::new(vec![
            term_table::table_cell::TableCell::new(&relation.name),
            term_table::table_cell::TableCell::new(&relation.ty),
            term_table::table_cell::TableCell::new(&relation.oid),
            term_table::table_cell::TableCell::new(relation.comment.clone().unwrap_or_default()),
        ]));
    }

    println!(
        "\nFound {} relation(s) in schema '{}'.",
        relations.len(),
        schema
    );
    println!("{}", table.render());

    Ok(())
}

pub fn relation(
    connection: &elephantry::Connection,
    schema: &str,
    relation: &str,
) -> crate::Result {
    let mut table = term_table::Table::new();
    table.style = term_table::TableStyle::rounded();

    table.add_row(term_table::row::Row::new(vec![
        term_table::table_cell::TableCell::new("pk"),
        term_table::table_cell::TableCell::new("name"),
        term_table::table_cell::TableCell::new("type"),
        term_table::table_cell::TableCell::new("default"),
        term_table::table_cell::TableCell::new("notnull"),
        term_table::table_cell::TableCell::new("comment"),
    ]));

    for column in elephantry::inspect::relation(connection, schema, relation)? {
        let primary = if column.is_primary {
            "*".to_string()
        } else {
            String::new()
        };

        let not_null = if column.is_notnull {
            "yes".to_string()
        } else {
            "no".to_string()
        };

        table.add_row(term_table::row::Row::new(vec![
            term_table::table_cell::TableCell::new(primary),
            term_table::table_cell::TableCell::new(&column.name),
            term_table::table_cell::TableCell::new(&column_type(&column)),
            term_table::table_cell::TableCell::new(column.default.clone().unwrap_or_default()),
            term_table::table_cell::TableCell::new(not_null),
            term_table::table_cell::TableCell::new(column.comment.clone().unwrap_or_default()),
        ]));
    }

    println!("\nRelation {}.{}", schema, relation);
    println!("{}", table.render());

    Ok(())
}

fn column_type(column: &elephantry::inspect::Column) -> String {
    use std::convert::TryFrom;

    let ty = if let Ok(ty) = elephantry::pq::types::Type::try_from(column.oid) {
        match ty.kind {
            elephantry::pq::types::Kind::Array(_) => {
                format!("{}[]", ty.name.trim_start_matches('_'))
            }
            _ => ty.name.to_string(),
        }
    } else {
        column.ty.clone()
    };

    ty
}

pub fn enums(connection: &elephantry::Connection, schema: &str) -> crate::Result {
    let enumerations = elephantry::inspect::enums(connection, schema)?;

    let mut table = term_table::Table::new();
    table.style = term_table::TableStyle::rounded();

    table.add_row(term_table::row::Row::new(vec![
        term_table::table_cell::TableCell::new("name"),
        term_table::table_cell::TableCell::new("elements"),
        term_table::table_cell::TableCell::new("description"),
    ]));

    for enumeration in &enumerations {
        table.add_row(term_table::row::Row::new(vec![
            term_table::table_cell::TableCell::new(&enumeration.name),
            term_table::table_cell::TableCell::new(&format!("{:?}", enumeration.elements)),
            term_table::table_cell::TableCell::new(
                &enumeration.description.clone().unwrap_or_default(),
            ),
        ]));
    }

    println!(
        "\nFound {} enum(s) in schema '{}'.",
        enumerations.len(),
        schema
    );
    println!("{}", table.render());

    Ok(())
}

pub fn domains(connection: &elephantry::Connection, schema: &str) -> crate::Result {
    let domains = elephantry::inspect::domains(connection, schema)?;

    let mut table = term_table::Table::new();
    table.style = term_table::TableStyle::rounded();

    table.add_row(term_table::row::Row::new(vec![
        term_table::table_cell::TableCell::new("name"),
        term_table::table_cell::TableCell::new("description"),
    ]));

    for domain in &domains {
        table.add_row(term_table::row::Row::new(vec![
            term_table::table_cell::TableCell::new(&domain.name),
            term_table::table_cell::TableCell::new(&domain.description.clone().unwrap_or_default()),
        ]));
    }

    println!(
        "\nFound {} domain(s) in schema '{}'.",
        domains.len(),
        schema
    );
    println!("{}", table.render());

    Ok(())
}

pub fn composites(connection: &elephantry::Connection, schema: &str) -> crate::Result {
    let composites = elephantry::inspect::composites(connection, schema)?;

    let mut table = term_table::Table::new();
    table.style = term_table::TableStyle::rounded();

    table.add_row(term_table::row::Row::new(vec![
        term_table::table_cell::TableCell::new("name"),
        term_table::table_cell::TableCell::new("fields"),
        term_table::table_cell::TableCell::new("description"),
    ]));

    for composite in &composites {
        let fields = composite
            .fields
            .iter()
            .map(|(n, t)| format!("{}::{}", n, t.name))
            .collect::<Vec<_>>()
            .join(", ");

        table.add_row(term_table::row::Row::new(vec![
            term_table::table_cell::TableCell::new(&composite.name),
            term_table::table_cell::TableCell::new(&fields),
            term_table::table_cell::TableCell::new(
                &composite.description.clone().unwrap_or_default(),
            ),
        ]));
    }

    println!(
        "\nFound {} composite type(s) in schema '{}'.",
        composites.len(),
        schema
    );
    println!("{}", table.render());

    Ok(())
}
