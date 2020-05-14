#[cfg(feature = "derive")]
include!("entity_derive.rs");

#[cfg(not(feature = "derive"))]
include!("entity.rs");

fn main() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/elephantry".to_string());
    let elephantry = elephantry::Pool::default()
        .add_default("elephantry", &database_url)
        .unwrap();
    let connection = elephantry.get_default().unwrap();

    let count = connection
        .count_where::<EventModel>("name = $1", &[&"pageview"])
        .unwrap();
    println!("Count events: {}", count);
    assert_eq!(count, 7);
    println!();

    println!("Find one event:\n");
    find_by_pk::<EventModel>(
        connection,
        "f186b680-237d-449d-ad66-ad91c4e53d3d",
    );
    println!();

    println!("Find all events:\n");
    find_all::<EventModel>(connection);
    println!();

    println!("Find all extra events:\n");
    find_all::<EventExtraModel>(connection);
    println!();

    println!("Insert one row:\n");
    let new_event = Event {
        uuid: None,
        name: "purchase".to_string(),
        visitor_id: 15,
        #[cfg(feature = "json")]
        properties: serde_json::json!({ "amount": 200 }),
        #[cfg(not(feature = "json"))]
        properties: "{\"amount\": 200}".to_string(),
        #[cfg(feature = "json")]
        browser: serde_json::json!({ "name": "Firefox", "resolution": { "x": 1280, "y": 800 } }),
        #[cfg(not(feature = "json"))]
        browser: "{ \"name\": \"Firefox\", \"resolution\": { \"x\": 1280, \"y\": 800 } }"
            .to_string(),
    };
    let mut entity = insert_one::<EventModel>(connection, &new_event);
    println!();

    println!("Update one row:\n");
    entity.name = "pageview".to_string();
    let entity = update_one::<EventModel>(
        connection,
        &elephantry::pk!(uuid => entity.uuid),
        &entity,
    );
    assert_eq!(&entity.name, "pageview");
    println!();

    println!("Delete one row\n");
    connection.delete_one::<EventModel>(&entity).unwrap();
    let uuid = entity.uuid.unwrap();
    assert!(connection
        .find_by_pk::<EventModel>(&elephantry::pk! {uuid => uuid})
        .unwrap()
        .is_none());
    assert_eq!(
        connection
            .exist_where::<EventModel>("uuid = $1", &[&uuid])
            .unwrap(),
        false
    );

    let count = connection
        .model::<EventModel>()
        .count_uniq_visitor()
        .unwrap();
    assert_eq!(count, 4);
    println!("Count uniq visitor: {}", count);
}

fn find_by_pk<'a, M>(connection: &elephantry::Connection, uuid: &str)
where
    M: elephantry::Model<'a>,
    M::Entity: std::fmt::Debug,
{
    #[cfg(feature = "json")]
    let uuid = uuid::Uuid::parse_str(uuid).unwrap();
    let event = connection
        .find_by_pk::<EventModel>(&elephantry::pk!(uuid))
        .unwrap();

    match event {
        Some(event) => println!("{:?}", event),
        None => println!("Event '{}' not found", uuid),
    };
}

fn find_all<'a, M>(connection: &elephantry::Connection)
where
    M: elephantry::Model<'a>,
    M::Entity: std::fmt::Debug,
{
    let events = connection.find_all::<M>(None).unwrap();

    if events.is_empty() {
        println!("No events in database.");
    }
    else {
        for event in events {
            println!("{:?}", event);
        }
    }
}

fn insert_one<'a, M>(
    connection: &elephantry::Connection,
    entity: &M::Entity,
) -> M::Entity
where
    M: elephantry::Model<'a>,
    M::Entity: std::fmt::Debug,
{
    let new_entity = connection.insert_one::<M>(&entity).unwrap();

    println!("{:?}", new_entity);

    new_entity
}

fn update_one<'a, M>(
    connection: &elephantry::Connection,
    pk: &std::collections::HashMap<&str, &dyn elephantry::ToSql>,
    entity: &M::Entity,
) -> M::Entity
where
    M: elephantry::Model<'a>,
    M::Entity: std::fmt::Debug,
{
    let new_entity = connection.update_one::<M>(pk, entity).unwrap();

    println!("{:?}", new_entity);

    new_entity
}