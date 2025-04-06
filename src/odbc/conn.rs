use odbc_api::{Connection, ConnectionOptions, Environment, Error};

pub struct OdbcConnection {
    pub environment: Environment,
}

impl OdbcConnection {
    pub fn new() -> Result<Self, Error> {
        let environment = Environment::new()?;

        Ok(Self { environment })
    }

    pub fn connect(&self, conn_str: &str) -> Result<Connection<'_>, Error> {
        self.environment
            .connect_with_connection_string(conn_str, ConnectionOptions::default())
    }
}
