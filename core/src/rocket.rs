use dep_rocket as rocket;

#[cfg_attr(docsrs, doc(cfg(feature = "rocket")))]
impl rocket_sync_db_pools::Poolable for crate::Connection {
    type Error = r2d2::Error;
    type Manager = crate::r2d2::ConnectionManager;

    fn pool(
        db_name: &str,
        rocket: &rocket::Rocket<rocket::Build>,
    ) -> rocket_sync_db_pools::PoolResult<Self> {
        let config = rocket_sync_db_pools::Config::from(db_name, rocket)?;
        let manager = Self::Manager::new(&config.url);

        let connection = r2d2::Pool::builder()
            .max_size(config.pool_size)
            .build(manager)?;

        Ok(connection)
    }
}
