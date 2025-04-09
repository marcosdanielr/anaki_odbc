#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::CString;
    use std::os::raw::c_int;

    use crate::ffi::{OdbcError, odbc_connect, odbc_create_connection, odbc_free_connection};

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
}
