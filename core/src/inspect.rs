#![allow(unused_variables)]

#[derive(Debug, elephantry_derive::Entity)]
#[entity(internal)]
pub struct Schema {
    pub name: String,
    pub oid: crate::pq::Oid,
    pub relations: String,
    pub comment: String,
}

/**
 * Retreive schemas of the connected database.
 */
pub fn database(connection: &crate::Connection) -> crate::Result<Vec<crate::inspect::Schema>> {
    connection
        .query(
            r#"
select
    n.nspname     as "name",
    n.oid         as "oid",
    count(c)      as "relations",
    d.description as "comment"
from pg_catalog.pg_namespace n
    left join pg_catalog.pg_description d on n.oid = d.objoid
    left join pg_catalog.pg_class c on
        c.relnamespace = n.oid and c.relkind in ('r', 'v')
where n.nspname !~ '^pg' and n.nspname <> 'information_schema'
group by 1, 2, 4
order by 1;
"#,
            &[],
        )
        .map(|x| x.collect())
}

#[derive(Debug, elephantry_derive::Entity)]
#[entity(internal)]
pub struct Relation {
    pub name: String,
    pub ty: String,
    pub oid: crate::pq::Oid,
    pub comment: Option<String>,
}

/**
 * Retreive relations (ie: tables, views, …) of `schema`.
 */
pub fn schema(
    connection: &crate::Connection,
    schema: &str,
) -> crate::Result<Vec<crate::inspect::Relation>> {
    let oid = crate::inspect::schema_oid(connection, &schema)?;

    connection
        .query(
            r#"
select
    cl.relname      as "name",
    case
        when cl.relkind = 'r' then 'table'
        when cl.relkind = 'v' then 'view'
        when cl.relkind = 'm' then 'materialized view'
        when cl.relkind = 'f' then 'foreign table'
        else 'other'
    end             as "ty",
    cl.oid          as "oid",
    des.description as "comment"
from
    pg_catalog.pg_class cl
        left join pg_catalog.pg_description des on
            cl.oid = des.objoid and des.objsubid = 0
where relkind in ('r', 'v', 'm', 'f')
and cl.relnamespace = $*
order by name asc;
"#,
            &[&oid],
        )
        .map(|x| x.collect())
}

#[derive(Debug, elephantry_derive::Entity)]
#[entity(internal)]
pub struct Column {
    #[elephantry(default)]
    pub is_primary: bool,
    pub name: String,
    pub oid: crate::pq::Oid,
    pub ty: String,
    pub default: Option<String>,
    pub is_notnull: bool,
    pub comment: Option<String>,
}

/**
 * Retreive columns of the `schema.relation` relation.
 */
pub fn relation(
    connection: &crate::Connection,
    schema: &str,
    relation: &str,
) -> crate::Result<Vec<crate::inspect::Column>> {
    let oid = connection
        .query_one::<i32>(
            r#"
select c.oid as oid
    from
        pg_catalog.pg_class c
            left join pg_catalog.pg_namespace n on n.oid = c.relnamespace
    where n.nspname = $1
        and c.relname = $2
    "#,
            &[&schema, &relation],
        )
        .map_err(|_| {
            return crate::Error::Inspect(format!("Unknow relation {}.{}", schema, relation));
        })?;

    connection
        .query(
            r#"
select
    att.attnum = any(ind.indkey) as "is_primary",
    att.attname as "name",
    typ.oid as "oid",
    case
        when name.nspname = 'pg_catalog' then typ.typname
        else format('%s.%s', name.nspname, typ.typname)
    end as "ty",
    pg_catalog.pg_get_expr(def.adbin, def.adrelid) as "default",
    att.attnotnull as "is_notnull",
    dsc.description as "comment"
from
  pg_catalog.pg_attribute att
    join pg_catalog.pg_type  typ  on att.atttypid = typ.oid
    join pg_catalog.pg_class cla  on att.attrelid = cla.oid
    join pg_catalog.pg_namespace clns on cla.relnamespace = clns.oid
    left join pg_catalog.pg_description dsc on cla.oid = dsc.objoid and att.attnum = dsc.objsubid
    left join pg_catalog.pg_attrdef def     on att.attrelid = def.adrelid and att.attnum = def.adnum
    left join pg_catalog.pg_index ind       on cla.oid = ind.indrelid and ind.indisprimary
    left join pg_catalog.pg_namespace name  on typ.typnamespace = name.oid
where
    att.attnum > 0
    and not att.attisdropped
    and att.attrelid = $*
order by
    att.attnum
"#,
            &[&oid],
        )
        .map(|x| x.collect())
}

#[derive(Debug, elephantry_derive::Entity)]
#[entity(internal)]
pub struct Enum {
    pub name: String,
    pub elements: Vec<String>,
    pub description: Option<String>,
}

/**
 * Retreive enumeration for `schema`.
 */
pub fn enums(
    connection: &crate::Connection,
    schema: &str,
) -> crate::Result<Vec<crate::inspect::Enum>> {
    crate::inspect::types(connection, schema, 'e').map(|x| x.collect())
}

#[derive(Debug, elephantry_derive::Entity)]
#[entity(internal)]
pub struct Domain {
    pub name: String,
    pub description: Option<String>,
}

/**
 * Retreive domain for `schema`.
 */
pub fn domains(
    connection: &crate::Connection,
    schema: &str,
) -> crate::Result<Vec<crate::inspect::Domain>> {
    crate::inspect::types(connection, schema, 'd').map(|x| x.collect())
}

#[derive(Debug, elephantry_derive::Entity)]
#[entity(internal)]
pub struct Composite {
    pub name: String,
    #[elephantry(default)]
    pub fields: Vec<(String, crate::pq::Type)>,
    pub description: Option<String>,
}

/**
 * Retreive composite type for `schema`.
 */
pub fn composites(
    connection: &crate::Connection,
    schema: &str,
) -> crate::Result<Vec<crate::inspect::Composite>> {
    let mut composites =
        crate::inspect::types(connection, schema, 'c')?.collect::<Vec<crate::inspect::Composite>>();

    for composite in &mut composites {
        composite.fields = crate::inspect::composite_fields(connection, &composite.name)?;
    }

    Ok(composites)
}

pub(crate) fn composite_fields(
    connection: &crate::Connection,
    composite: &str,
) -> crate::Result<Vec<(String, crate::pq::Type)>> {
    connection
        .query(
            r#"
select row(a.attname, pg_catalog.format_type(a.atttypid, a.atttypmod))
    from pg_catalog.pg_attribute a
    join pg_catalog.pg_class c
        on a.attrelid = c.oid
            and c.relname = $*
    where a.attnum > 0 and not a.attisdropped
    order by a.attnum;
        "#,
            &[&composite],
        )
        .map(|x| x.collect())
}

pub(crate) fn types<E: crate::Entity>(
    connection: &crate::Connection,
    schema: &str,
    typtype: char,
) -> crate::Result<crate::Rows<E>> {
    schema_oid(connection, schema)?;

    connection
        .query(
            r#"
select pg_catalog.format_type(t.oid, null) as "name",
    array(
        select e.enumlabel
        from pg_catalog.pg_enum e
        where e.enumtypid = t.oid
        order by e.enumsortorder
    ) as "elements",
    pg_catalog.obj_description(t.oid, 'pg_type') as "description"
from pg_catalog.pg_type t
    left join pg_catalog.pg_namespace n on n.oid = t.typnamespace
where t.typtype = $*
    and n.nspname = $*
    and (t.typrelid = 0 or (select c.relkind = 'c' from pg_catalog.pg_class c where c.oid = t.typrelid))
    and not exists(select 1 from pg_catalog.pg_type el where el.oid = t.typelem and el.typarray = t.oid)
    and n.nspname <> 'pg_catalog'
    and n.nspname <> 'information_schema'
    and pg_catalog.pg_type_is_visible(t.oid)
order by 1;
    "#,
            &[&typtype, &schema],
        )
}

pub(crate) fn schema_oid(connection: &crate::Connection, name: &str) -> crate::Result<i32> {
    connection
        .query_one::<i32>(
            r#"
select
    s.oid as oid
from
    pg_catalog.pg_namespace s
where s.nspname = $*
    "#,
            &[&name],
        )
        .map_err(|_| {
            return crate::Error::Inspect(format!("Unknow schema {}", name));
        })
}
