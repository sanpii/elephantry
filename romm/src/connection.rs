pub struct Connection
{
    connection: postgres::Connection,
}

impl Connection
{
    pub fn new(url: &str) -> postgres::Result<Self>
    {
        Ok(Self {
            connection: postgres::Connection::connect(url, postgres::TlsMode::None)?,
        })
    }

    pub fn find_all<M>(&self)
        -> postgres::Result<Vec<M::Entity>> where M: crate::Model
    {
        use crate::RowStructure;

        let query = format!(
            "SELECT {} FROM {};",
            M::create_projection(),
            M::RowStructure::relation(),
        );

        let results = self.connection.query(&query, &[])?;

        Ok(results.iter()
            .map(|row| M::create_entity(row))
            .collect()
        )
    }
}
