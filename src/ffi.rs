use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;

use crate::odbc::conn::OdbcConnection;

#[repr(C)]
pub struct OdbcConnectionHandle {
    connection: Option<OdbcConnection>,
}

#[repr(C)]
pub enum OdbcError {
    Success = 0,
    ConnectionError = 1,
    StringConversionError = 2,
    InvalidHandle = 3,
    NullPointer = 4,
}

#[unsafe(no_mangle)]
pub extern "C" fn odbc_create_connection() -> *mut OdbcConnectionHandle {
    match OdbcConnection::new() {
        Ok(connection) => {
            let handle = Box::new(OdbcConnectionHandle {
                connection: Some(connection),
            });
            Box::into_raw(handle)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn odbc_connect(
    handle: *mut OdbcConnectionHandle,
    conn_str: *const c_char,
) -> c_int {
    if handle.is_null() {
        return OdbcError::InvalidHandle as c_int;
    }

    if conn_str.is_null() {
        return OdbcError::NullPointer as c_int;
    }

    let conn_str_rs = unsafe {
        match CStr::from_ptr(conn_str).to_str() {
            Ok(s) => s,
            Err(_) => return OdbcError::StringConversionError as c_int,
        }
    };

    let handle_ref = unsafe { &mut *handle };

    if let Some(connection) = &handle_ref.connection {
        match connection.connect(conn_str_rs) {
            Ok(_) => OdbcError::Success as c_int,
            Err(_) => OdbcError::ConnectionError as c_int,
        }
    } else {
        OdbcError::InvalidHandle as c_int
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn odbc_free_connection(handle: *mut OdbcConnectionHandle) -> c_int {
    if handle.is_null() {
        return OdbcError::InvalidHandle as c_int;
    }

    unsafe {
        let _ = Box::from_raw(handle);
    }

    OdbcError::Success as c_int
}
