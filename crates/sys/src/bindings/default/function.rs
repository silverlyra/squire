use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

use super::{
    connection::sqlite3,
    sqlite3_destructor_type,
    types::{sqlite3_int64, sqlite3_uint64},
    value::sqlite3_value,
};

/// The [evaluation context][] for a user-defined SQL function.
///
/// [evaluation context]: https://sqlite.org/c3ref/context.html
#[repr(C)]
pub struct sqlite3_context {
    _unused: [u8; 0],
}

unsafe extern "C" {
    pub fn sqlite3_create_function_v2(
        db: *mut sqlite3,
        zFunctionName: *const c_char,
        nArg: c_int,
        eTextRep: c_int,
        pApp: *mut c_void,
        xFunc: Option<
            unsafe extern "C" fn(
                context: *mut sqlite3_context,
                nArg: c_int,
                args: *mut *mut sqlite3_value,
            ),
        >,
        xStep: Option<
            unsafe extern "C" fn(
                context: *mut sqlite3_context,
                nArg: c_int,
                args: *mut *mut sqlite3_value,
            ),
        >,
        xFinal: Option<unsafe extern "C" fn(context: *mut sqlite3_context)>,
        xDestroy: Option<unsafe extern "C" fn(pApp: *mut c_void)>,
    ) -> c_int;

    pub fn sqlite3_create_window_function(
        db: *mut sqlite3,
        zFunctionName: *const c_char,
        nArg: c_int,
        eTextRep: c_int,
        pApp: *mut c_void,
        xStep: Option<
            unsafe extern "C" fn(
                context: *mut sqlite3_context,
                nArg: c_int,
                args: *mut *mut sqlite3_value,
            ),
        >,
        xFinal: Option<unsafe extern "C" fn(context: *mut sqlite3_context)>,
        xValue: Option<unsafe extern "C" fn(context: *mut sqlite3_context)>,
        xInverse: Option<
            unsafe extern "C" fn(
                conte: *mut sqlite3_context,
                arg2: c_int,
                arg3: *mut *mut sqlite3_value,
            ),
        >,
        xDestroy: Option<unsafe extern "C" fn(pApp: *mut c_void)>,
    ) -> c_int;

    pub fn sqlite3_context_db_handle(pCtx: *mut sqlite3_context) -> *mut sqlite3;

    pub fn sqlite3_result_blob(
        context: *mut sqlite3_context,
        value: *const c_void,
        len: c_int,
        destructor: sqlite3_destructor_type,
    );
    pub fn sqlite3_result_blob64(
        context: *mut sqlite3_context,
        value: *const c_void,
        len: sqlite3_uint64,
        destructor: sqlite3_destructor_type,
    );
    pub fn sqlite3_result_double(context: *mut sqlite3_context, value: f64);
    pub fn sqlite3_result_error(context: *mut sqlite3_context, message: *const c_char, len: c_int);
    pub fn sqlite3_result_error_toobig(context: *mut sqlite3_context);
    pub fn sqlite3_result_error_nomem(context: *mut sqlite3_context);
    pub fn sqlite3_result_error_code(context: *mut sqlite3_context, value: c_int);
    pub fn sqlite3_result_int(context: *mut sqlite3_context, value: c_int);
    pub fn sqlite3_result_int64(context: *mut sqlite3_context, value: sqlite3_int64);
    pub fn sqlite3_result_null(context: *mut sqlite3_context);
    pub fn sqlite3_result_text(
        context: *mut sqlite3_context,
        value: *const c_char,
        len: c_int,
        destructor: sqlite3_destructor_type,
    );
    pub fn sqlite3_result_text64(
        context: *mut sqlite3_context,
        value: *const c_char,
        len: sqlite3_uint64,
        destructor: sqlite3_destructor_type,
        encoding: c_uchar,
    );
    pub fn sqlite3_result_value(context: *mut sqlite3_context, value: *mut sqlite3_value);
    pub fn sqlite3_result_pointer(
        context: *mut sqlite3_context,
        value: *mut c_void,
        type_name: *const c_char,
        destructor: sqlite3_destructor_type,
    );
    pub fn sqlite3_result_zeroblob(context: *mut sqlite3_context, bytes: c_int);
    pub fn sqlite3_result_zeroblob64(context: *mut sqlite3_context, bytes: sqlite3_uint64)
    -> c_int;
    pub fn sqlite3_result_subtype(context: *mut sqlite3_context, value: c_uint);

    pub fn sqlite3_get_auxdata(context: *mut sqlite3_context, iArg: c_int) -> *mut c_void;
    pub fn sqlite3_set_auxdata(
        context: *mut sqlite3_context,
        iArg: c_int,
        pData: *mut c_void,
        destructor: Option<unsafe extern "C" fn(arg1: *mut c_void)>,
    );

    pub fn sqlite3_aggregate_context(context: *mut sqlite3_context, nBytes: c_int) -> *mut c_void;

    pub fn sqlite3_user_data(context: *mut sqlite3_context) -> *mut c_void;
}

pub const SQLITE_DETERMINISTIC: i32 = 1 << 11;
pub const SQLITE_DIRECTONLY: i32 = 1 << 19;
pub const SQLITE_SUBTYPE: i32 = 1 << 20;
pub const SQLITE_INNOCUOUS: i32 = 1 << 21;
pub const SQLITE_RESULT_SUBTYPE: i32 = 1 << 24;
pub const SQLITE_SELFORDER1: i32 = 1 << 25;
