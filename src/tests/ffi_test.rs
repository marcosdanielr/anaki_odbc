#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use std::ffi::CString;
    use std::os::raw::{c_int, c_void};

    use std::env;

    use crate::ffi::{
        BinaryData, OdbcError, odbc_connect, odbc_create_connection, odbc_free_connection,
    };

    #[derive(Deserialize)]
    struct Header {
        columns: Vec<String>,
    }

    #[test]
    fn test_create_connection() {
        let handle = unsafe { odbc_create_connection() };
        assert!(!handle.is_null(), "Expected a valid connection handle");

        let free_result = unsafe { odbc_free_connection(handle) };
        assert_eq!(
            free_result,
            OdbcError::Success as c_int,
            "Expected successful free operation"
        );
    }

    #[test]
    fn test_connect_success() {
        let handle = unsafe { odbc_create_connection() };
        assert!(!handle.is_null(), "Expected a valid connection handle");

        let conn_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set");

        let conn_url_c = CString::new(conn_url).expect("Failed to create CString");

        let connect_result = unsafe { odbc_connect(handle, conn_url_c.as_ptr()) };

        assert_eq!(
            connect_result,
            OdbcError::Success as c_int,
            "Expected successful connection, got error code: {}",
            connect_result
        );

        unsafe { odbc_free_connection(handle) };
    }

    #[test]
    fn test_null_handle() {
        let conn_url = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "dummy".to_string());
        let conn_url_c = CString::new(conn_url).expect("Failed to create CString");

        let result = unsafe { odbc_connect(std::ptr::null_mut(), conn_url_c.as_ptr()) };
        assert_eq!(
            result,
            OdbcError::NullPointer as c_int,
            "Should return NullPointer error for null handle"
        );
    }

    #[test]
    fn test_null_connection_string() {
        let handle = unsafe { odbc_create_connection() };
        assert!(!handle.is_null(), "Expected a valid connection handle");

        let result = unsafe { odbc_connect(handle, std::ptr::null()) };
        assert_eq!(
            result,
            OdbcError::NullPointer as c_int,
            "Should return NullPointer error for null connection string"
        );

        unsafe {
            odbc_free_connection(handle);
        }
    }

    #[test]
    fn test_integration_workflow() {
        let handle = unsafe { odbc_create_connection() };
        assert!(!handle.is_null(), "Failed to create connection handle");

        if let Ok(conn_url) = env::var("TEST_DATABASE_URL") {
            let conn_url_c = CString::new(conn_url).expect("Failed to create CString");
            let connect_result = unsafe { odbc_connect(handle, conn_url_c.as_ptr()) };

            if connect_result == OdbcError::Success as c_int {
                println!("Connection established successfully");
            } else {
                println!("Connection failed with error code: {}", connect_result);
            }
        }

        let free_result = unsafe { odbc_free_connection(handle) };
        assert_eq!(
            free_result,
            OdbcError::Success as c_int,
            "Failed to free connection"
        );
    }

    #[test]
    fn test_execute_via_ffi() {
        use crate::ffi::{
            BinaryData, OdbcError, odbc_connect, odbc_create_connection, odbc_execute,
            odbc_free_connection,
        };
        use std::env;
        use std::ffi::CString;
        use std::os::raw::c_void;
        use std::sync::{Arc, Mutex};

        let conn_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set");
        let conn_url_c = CString::new(conn_url).unwrap();

        let handle = unsafe { odbc_create_connection() };
        assert!(!handle.is_null());

        let connect_result = unsafe { odbc_connect(handle, conn_url_c.as_ptr()) };
        assert_eq!(connect_result, OdbcError::Success as i32);

        let create_sql =
            CString::new("CREATE TABLE IF NOT EXISTS temp_ffi_test (id INT, name VARCHAR(50))")
                .unwrap();
        let create_result = unsafe {
            odbc_execute(
                handle,
                create_sql.as_ptr(),
                dummy_callback,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(create_result, OdbcError::Success as i32);

        let insert_sql =
            CString::new("INSERT INTO temp_ffi_test (id, name) VALUES (1, 'Test')").unwrap();
        let insert_result = unsafe {
            odbc_execute(
                handle,
                insert_sql.as_ptr(),
                dummy_callback,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(insert_result, OdbcError::Success as i32);

        let binary_data: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let user_data_arc = Arc::clone(&binary_data);

        extern "C" fn collect_callback(binary: *const BinaryData, user_data: *mut c_void) {
            if binary.is_null() || user_data.is_null() {
                return;
            }

            let data =
                unsafe { std::slice::from_raw_parts((*binary).data, (*binary).len) }.to_vec();

            let arc = unsafe { Arc::from_raw(user_data as *const Mutex<Vec<Vec<u8>>>) };
            {
                let mut lock = arc.lock().unwrap();
                lock.push(data);
            }
            std::mem::forget(arc);
        }

        let user_data_ptr = Arc::into_raw(user_data_arc) as *mut c_void;

        let select_sql = CString::new("SELECT id, name FROM temp_ffi_test").unwrap();
        let select_result =
            unsafe { odbc_execute(handle, select_sql.as_ptr(), collect_callback, user_data_ptr) };
        assert_eq!(select_result, OdbcError::Success as i32);

        unsafe {
            Arc::from_raw(user_data_ptr as *const Mutex<Vec<Vec<u8>>>);
        }

        let collected = binary_data.lock().unwrap();

        let header: Header = rmp_serde::from_slice(&collected[0]).unwrap();
        assert_eq!(header.columns, vec!["id", "name"]);

        let row: std::collections::HashMap<String, serde_json::Value> =
            rmp_serde::from_slice(&collected[1]).unwrap();
        assert_eq!(row["id"], 1);
        assert_eq!(row["name"], "Test");

        let drop_sql = CString::new("DROP TABLE temp_ffi_test").unwrap();
        let drop_result = unsafe {
            odbc_execute(
                handle,
                drop_sql.as_ptr(),
                dummy_callback,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(drop_result, OdbcError::Success as i32);

        let free_result = unsafe { odbc_free_connection(handle) };
        assert_eq!(free_result, OdbcError::Success as i32);
    }

    extern "C" fn dummy_callback(_binary: *const BinaryData, _user_data: *mut c_void) {}
}
