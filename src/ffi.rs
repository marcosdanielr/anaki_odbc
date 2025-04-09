use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::{panic, ptr};

use odbc_api::Connection;

use crate::odbc::conn::OdbcConnectionManager;
use crate::odbc::exec::execute;

#[repr(C)]
pub struct OdbcConnectionHandle {
    connection: Option<Connection<'static>>,
    manager: Option<OdbcConnectionManager>,
}

#[repr(i32)]
#[derive(Debug)]
pub enum OdbcError {
    Success = 0,
    ConnectionError = 1,
    StringConversionError = 2,
    InvalidHandle = 3,
    NullPointer = 4,
    ExecutionError = 5,
    Panic = 6,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn odbc_create_connection() -> *mut OdbcConnectionHandle {
    match OdbcConnectionManager::new() {
        Ok(manager) => {
            let handle = Box::new(OdbcConnectionHandle {
                manager: Some(manager),
                connection: None,
            });
            Box::into_raw(handle)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn odbc_connect(
    handle: *mut OdbcConnectionHandle,
    conn_str: *const c_char,
) -> c_int {
    if handle.is_null() || conn_str.is_null() {
        return OdbcError::NullPointer as c_int;
    }

    let handle_ref = unsafe { &mut *handle };
    let conn_str_rs = match unsafe { CStr::from_ptr(conn_str) }.to_str() {
        Ok(s) => s,
        Err(_) => return OdbcError::StringConversionError as c_int,
    };

    let Some(manager) = &mut handle_ref.manager else {
        return OdbcError::InvalidHandle as c_int;
    };

    match manager.connect(conn_str_rs) {
        Ok(conn) => {
            handle_ref.connection =
                Some(unsafe { std::mem::transmute::<Connection<'_>, Connection<'static>>(conn) });
            OdbcError::Success as c_int
        }
        Err(_) => OdbcError::ConnectionError as c_int,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn odbc_execute(
    handle: *mut OdbcConnectionHandle,
    sql: *const c_char,
    callback: extern "C" fn(*const c_char, *mut c_void),
    user_data: *mut c_void,
) -> i32 {
    let result = panic::catch_unwind(|| {
        if handle.is_null() || sql.is_null() {
            return Err(OdbcError::NullPointer);
        }

        let handle_ref = unsafe { &mut *handle };

        let Some(conn) = &mut handle_ref.connection else {
            return Err(OdbcError::InvalidHandle);
        };

        let sql_str = unsafe { CStr::from_ptr(sql) }
            .to_str()
            .map_err(|_| OdbcError::StringConversionError)?;

        let on_csv = |line: String| {
            if let Ok(c_line) = CString::new(line) {
                callback(c_line.as_ptr(), user_data);
            }
        };

        execute(conn, sql_str, on_csv).map_err(|_| OdbcError::ExecutionError)
    });

    match result {
        Ok(Ok(_)) => OdbcError::Success as i32,
        Ok(Err(err)) => err as i32,
        Err(_) => OdbcError::Panic as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn odbc_free_connection(handle: *mut OdbcConnectionHandle) -> c_int {
    if handle.is_null() {
        return OdbcError::InvalidHandle as c_int;
    }

    let _ = unsafe { Box::from_raw(handle) };
    OdbcError::Success as c_int
}
