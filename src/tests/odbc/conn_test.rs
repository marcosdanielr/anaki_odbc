#[cfg(test)]
mod tests {
    use crate::odbc::conn::OdbcConnectionManager;
    use std::env;

    #[test]
    fn test_connect_success() {
        let odbc_connection = OdbcConnectionManager::new().expect("failed to connect do database");

        let conn_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set");

        let result = odbc_connection.connect(conn_url.as_str());

        assert!(result.is_ok(), "Expected Ok(Connection), got {:?}", result);
    }
}
