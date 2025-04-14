#[cfg(test)]
mod tests {
    use crate::odbc::conn::OdbcConnectionManager;
    use crate::odbc::exec::execute;
    use serde::Deserialize;
    use std::env;

    #[derive(Deserialize)]
    struct Header {
        columns: Vec<String>,
    }

    #[derive(Deserialize)]
    struct Meta {
        affected_rows: String,
    }

    #[test]
    fn test_execute_query() {
        let conn_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set");

        let environment = OdbcConnectionManager::new().expect("failed to create environment");

        let mut conn = environment.connect(&conn_url).expect("connection failed");

        let sql = "SELECT 1 AS id, 'test' AS name";

        let mut binary_data = vec![];

        let result = execute(&mut conn, sql, |data| {
            binary_data.push(data);
        });

        assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);

        let header: Header = rmp_serde::from_slice(&binary_data[0]).unwrap();
        assert_eq!(header.columns, vec!["id", "name"]);

        let row: std::collections::HashMap<String, serde_json::Value> =
            rmp_serde::from_slice(&binary_data[1]).unwrap();
        assert_eq!(row["id"], 1);
        assert_eq!(row["name"], "test");
    }

    #[test]
    fn test_create_table() {
        let manager = OdbcConnectionManager::new().expect("failed to create env");
        let conn_url = env::var("TEST_DATABASE_URL").expect("env var missing");
        let mut conn = manager.connect(&conn_url).expect("connect failed");

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
        let manager = OdbcConnectionManager::new().expect("failed to create env");
        let conn_url = env::var("TEST_DATABASE_URL").expect("env var missing");
        let mut conn = manager.connect(&conn_url).expect("connect failed");

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
            insert_all_result
        );

        let mut binary_data = vec![];
        let delete_result = execute(
            &mut conn,
            "DELETE FROM temp_create_test WHERE name LIKE 'M%'",
            |data| {
                binary_data.push(data);
            },
        );

        assert!(delete_result.is_ok(), "DELETE failed: {:?}", delete_result);

        let meta: Meta = rmp_serde::from_slice(&binary_data[0]).unwrap();
        assert_eq!(meta.affected_rows, "2");

        let drop_result = execute(&mut conn, "DROP TABLE temp_create_test", |_| {});
        assert!(drop_result.is_ok(), "Failed to drop table after test");
    }
}
