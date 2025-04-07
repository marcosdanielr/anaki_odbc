#[cfg(test)]
mod tests {
    use crate::odbc::conn::OdbcConnection;
    use crate::odbc::exec::execute;
    use std::env;

    #[test]
    fn test_execute_query() {
        let odbc_connection = OdbcConnection::new().expect("failed to connect to database");

        let conn_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set");
        let mut conn = odbc_connection
            .connect(conn_url.as_str())
            .expect("connection failed");

        let sql = "SELECT 1 AS id, 'test' AS name";

        let mut csv_lines = vec![];

        let result = execute(&mut conn, sql, |line| {
            csv_lines.push(line);
        });

        assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);

        assert_eq!(csv_lines[0], "id,name");
        assert_eq!(csv_lines[1], "1,test");
    }

    #[test]
    fn test_create_table() {
        let odbc_connection = OdbcConnection::new().expect("failed to connect to database");
        let conn_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set");
        let mut conn = odbc_connection
            .connect(&conn_url)
            .expect("connection failed");

        let result = execute(
            &mut conn,
            "CREATE TABLE IF NOT EXISTS temp_create_test (id INT, name VARCHAR(50))",
            |_| {},
        );

        assert!(
            result.is_ok(),
            "Expected CREATE to succeed, got {:?}",
            result
        );

        let drop_result = execute(&mut conn, "DROP TABLE temp_create_test", |_| {});
        assert!(drop_result.is_ok(), "Failed to drop table after test");
    }

    #[test]
    fn test_affected_rows() {
        let odbc_connection = OdbcConnection::new().expect("failed to connect to database");
        let conn_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set");
        let mut conn = odbc_connection
            .connect(&conn_url)
            .expect("connection failed");

        let create_table_result = execute(
            &mut conn,
            "CREATE TABLE IF NOT EXISTS temp_create_test (id INT, name VARCHAR(50))",
            |_| {},
        );

        assert!(
            create_table_result.is_ok(),
            "Expected CREATE to succeed, got {:?}",
            create_table_result
        );

        let insert_all_result = execute(
            &mut conn,
            "
            INSERT INTO temp_create_test (id, name) VALUES 
            (1, 'Marcos'),
            (2, 'Diego'),
            (3, 'Jacob'),
            (4, 'Max')
            ",
            |_| {},
        );

        assert!(
            insert_all_result.is_ok(),
            "Expected INSERT to succeed, got {:?}",
            create_table_result
        );

        let mut lines = vec![];
        let delete_result = execute(
            &mut conn,
            "DELETE FROM temp_create_test WHERE name LIKE 'M%'",
            |line| {
                lines.push(line);
            },
        );
        assert!(delete_result.is_ok());
        assert!(
            lines.contains(&"__META__,affected_rows=2".to_string()),
            "Expected 2 rows affected, but got: {:?}",
            lines
        );

        let drop_result = execute(&mut conn, "DROP TABLE temp_create_test", |_| {});
        assert!(drop_result.is_ok(), "Failed to drop table after test");
    }
}
