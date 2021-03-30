use case::CaseExt;
use std::io::Write;

pub fn schema(
    connection: &elephantry::Connection,
    prefix_dir: &str,
    schema: &str,
) -> crate::Result {
    let relations = elephantry::inspect::schema(connection, schema)?;

    add_mod(&format!("{}/model", prefix_dir), &schema)?;

    for r in relations {
        relation(connection, prefix_dir, schema, &r.name)?;
    }

    Ok(())
}

pub fn relation(
    connection: &elephantry::Connection,
    prefix_dir: &str,
    schema: &str,
    relation: &str,
) -> crate::Result {
    let dir = format!("{}/model/{}", prefix_dir, schema);
    add_mod(&dir, &relation)?;

    let filename = format!("{}/{}.rs", dir, relation);
    let mut file = std::io::BufWriter::new(std::fs::File::create(filename)?);

    write_entity(&mut file, connection, schema, relation)?;

    let mut pk = Vec::new();
    let mut columns = Vec::new();

    for column in &elephantry::inspect::relation(connection, schema, relation)? {
        let name = column.name.to_snake();

        if column.is_primary {
            pk.push(format!("\"{}\"", name));
        }

        columns.push(format!("            \"{name}\",", name = name,));
    }

    write!(
        file,
        r"
pub struct Model<'a> {{
    connection: &'a elephantry::Connection,
}}

impl<'a> elephantry::Model<'a> for Model<'a> {{
    type Entity = Entity;
    type Structure = Structure;

    fn new(connection: &'a elephantry::Connection) -> Self {{
        Self {{ connection }}
    }}
}}

"
    )?;

    write!(
        file,
        r#"pub struct Structure;

impl elephantry::Structure for Structure {{
    fn relation() -> &'static str {{
        "{schema}.{relation}"
    }}

    fn primary_key() -> &'static [&'static str] {{
        &[{pk}]
    }}

    fn columns() -> &'static [&'static str] {{
        &[
{columns}
        ]
    }}
}}
"#,
        pk = pk.join(","),
        schema = schema,
        relation = relation,
        columns = columns.join("\n")
    )?;

    Ok(())
}

pub fn entity(
    connection: &elephantry::Connection,
    prefix_dir: &str,
    schema: &str,
    relation: &str,
) -> crate::Result {
    let dir = format!("{}/model/{}", prefix_dir, schema);
    add_mod(&dir, &relation)?;

    let filename = format!("{}/{}.rs", dir, relation);
    let mut file = std::io::BufWriter::new(std::fs::File::create(filename)?);

    write_entity(&mut file, connection, schema, relation)?;

    Ok(())
}

fn write_entity<W>(
    file: &mut std::io::BufWriter<W>,
    connection: &elephantry::Connection,
    schema: &str,
    relation: &str,
) -> crate::Result
where
    W: std::io::Write,
{
    let columns = elephantry::inspect::relation(connection, schema, relation)?;

    let mut fields = Vec::new();

    for column in &columns {
        let name = name_to_rust(&column);
        let ty = ty_to_rust(&column)?;

        fields.push(format!("    pub {}: {},", name, ty));
    }

    write!(
        file,
        r"#[derive(elephantry::Entity)]
pub struct Entity {{
{fields}
}}
",
        fields = fields.join("\n")
    )?;

    Ok(())
}

pub fn enums(connection: &elephantry::Connection, prefix_dir: &str, schema: &str) -> crate::Result {
    let dir = format!("{}/enums", prefix_dir);
    std::fs::create_dir_all(&dir)?;

    let filename = format!("{}/{}.rs", dir, schema);
    let mut file = std::io::BufWriter::new(std::fs::File::create(filename)?);

    for enumeration in &elephantry::inspect::enums(connection, schema)? {
        write_enum(&mut file, &enumeration)?;
    }

    Ok(())
}

fn write_enum<W>(
    file: &mut std::io::BufWriter<W>,
    enumeration: &elephantry::inspect::Enum,
) -> crate::Result
where
    W: std::io::Write,
{
    let elements = enumeration
        .elements
        .iter()
        .map(|x| format!("    {},", x))
        .collect::<Vec<_>>();

    write!(
        file,
        r"#[derive(elephantry::Enum)]
pub struct {name} {{
{elements}
}}
",
        name = enumeration.name,
        elements = elements.join("\n"),
    )?;

    Ok(())
}

pub fn composites(
    connection: &elephantry::Connection,
    prefix_dir: &str,
    schema: &str,
) -> crate::Result {
    let dir = format!("{}/composites", prefix_dir);
    std::fs::create_dir_all(&dir)?;

    let filename = format!("{}/{}.rs", dir, schema);
    let mut file = std::io::BufWriter::new(std::fs::File::create(filename)?);

    for composite in &elephantry::inspect::composites(connection, schema)? {
        write_composite(&mut file, &composite)?;
    }

    Ok(())
}

fn write_composite<W>(
    file: &mut std::io::BufWriter<W>,
    composite: &elephantry::inspect::Composite,
) -> crate::Result
where
    W: std::io::Write,
{
    let fields = composite
        .fields
        .iter()
        .map(|(name, ty)| format!("    {}: {},", name, elephantry::pq::sql_to_rust(ty)))
        .collect::<Vec<_>>();

    write!(
        file,
        r"#[derive(elephantry::Composite)]
pub struct {name} {{
{fields}
}}
",
        name = composite.name.to_camel(),
        fields = fields.join("\n")
    )?;

    Ok(())
}

fn ty_to_rust(column: &elephantry::inspect::Column) -> crate::Result<String> {
    use std::convert::TryFrom;

    let ty = elephantry::pq::Type::try_from(column.oid).map_err(crate::Error::Libpq)?;
    let mut rty = elephantry::pq::sql_to_rust(&ty);

    if !column.is_notnull {
        rty = format!("Option<{}>", rty);
    }

    Ok(rty)
}

fn name_to_rust(column: &elephantry::inspect::Column) -> String {
    let mut name = column.name.to_snake();

    if is_keyword(&name) {
        name = format!("r#{}", name);
    }

    name
}

fn is_keyword(name: &str) -> bool {
    static KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do",
        "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
    ];

    KEYWORDS.contains(&name)
}

fn add_mod(dir: &str, name: &str) -> crate::Result {
    std::fs::create_dir_all(&dir)?;

    let mod_filename = format!("{}/mod.rs", dir);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&mod_filename)?;
    file.write_all(format!("mod {};\n", name).as_bytes())?;

    Ok(())
}
