use odbc_api::{Connection, ConnectionOptions, Environment, Error};
use std::sync::Arc;

#[derive(Clone)]
pub struct OdbcConnectionManager {
    environment: Arc<Environment>,
}

impl OdbcConnectionManager {
    pub fn new() -> Result<Self, Error> {
        let environment = Environment::new()?;
        Ok(Self {
            environment: Arc::new(environment),
        })
    }

    pub fn connect(&self, conn_str: &str) -> Result<Connection<'_>, Error> {
        self.environment
            .connect_with_connection_string(conn_str, ConnectionOptions::default())
    }
}
