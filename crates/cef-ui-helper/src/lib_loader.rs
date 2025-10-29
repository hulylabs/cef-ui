use anyhow::Result;
use cef_ui_sys::{
    cef_app_t, cef_main_args_t, cef_process_message_t, cef_string_t, cef_string_userfree_utf16_t,
    cef_string_utf16_t, cef_v8handler_t, cef_v8value_t, char16_t
};
use libloading::{Library, Symbol};
use std::{
    env::current_exe,
    ffi::c_char,
    os::raw::{c_int, c_void},
    path::PathBuf,
    sync::LazyLock
};

type CefExecuteProcessFn = unsafe extern "C" fn(
    args: *const cef_main_args_t,
    app: *mut cef_app_t,
    extra_info: *mut c_void
) -> c_int;
type CefStringUtf8ToUtf16Fn =
    unsafe extern "C" fn(src: *const c_char, src_len: usize, output: *mut cef_string_t) -> c_int;
type CefStringUtf16SetFn = unsafe extern "C" fn(
    src: *const char16_t,
    src_len: usize,
    output: *mut cef_string_utf16_t,
    copy: c_int
) -> c_int;
type CefStringUsefreeUtf16FreeFn = unsafe extern "C" fn(str_: cef_string_userfree_utf16_t);
type CefRegisterExtensionFn = unsafe extern "C" fn(
    name: *const cef_string_t,
    js_code: *const cef_string_t,
    handler: *mut cef_v8handler_t
) -> c_int;
type CefProcessMessageCreateFn =
    unsafe extern "C" fn(name: *const cef_string_t) -> *mut cef_process_message_t;
type CefV8ValueCreateStringFn =
    unsafe extern "C" fn(value: *const cef_string_t) -> *mut cef_v8value_t;
type CefV8ValueCreateFunctionFn = unsafe extern "C" fn(
    name: *const cef_string_t,
    handler: *mut cef_v8handler_t
) -> *mut cef_v8value_t;

pub struct CefLibrary {
    _lib:                               &'static Library,
    pub cef_execute_process:            Symbol<'static, CefExecuteProcessFn>,
    pub cef_string_utf8_to_utf16:       Symbol<'static, CefStringUtf8ToUtf16Fn>,
    pub cef_string_utf16_set:           Symbol<'static, CefStringUtf16SetFn>,
    pub cef_string_userfree_utf16_free: Symbol<'static, CefStringUsefreeUtf16FreeFn>,
    pub cef_register_extension:         Symbol<'static, CefRegisterExtensionFn>,
    pub cef_process_message_create:     Symbol<'static, CefProcessMessageCreateFn>,
    pub cef_v8_value_create_string:     Symbol<'static, CefV8ValueCreateStringFn>,
    pub cef_v8_value_create_function:   Symbol<'static, CefV8ValueCreateFunctionFn>
}
const CEF_PATH: &str = "../../../Chromium Embedded Framework.framework/Chromium Embedded Framework";

pub static CEFLIB: LazyLock<CefLibrary> = LazyLock::new(|| unsafe {
    let path = get_cef_path(CEF_PATH).expect("failed to get CEF path");
    let lib = Library::new(path).expect("failed to load CEF library");
    let lib = Box::leak(Box::new(lib));

    let cef_execute_process: Symbol<CefExecuteProcessFn> = lib
        .get(b"cef_execute_process\0")
        .expect("failed to load cef_execute_process");
    let cef_string_utf8_to_utf16: Symbol<CefStringUtf8ToUtf16Fn> = lib
        .get(b"cef_string_utf8_to_utf16\0")
        .expect("failed to load cef_string_utf8_to_utf16");
    let cef_string_utf16_set: Symbol<CefStringUtf16SetFn> = lib
        .get(b"cef_string_utf16_set\0")
        .expect("failed to load cef_string_utf16_set");
    let cef_string_userfree_utf16_free: Symbol<CefStringUsefreeUtf16FreeFn> = lib
        .get(b"cef_string_userfree_utf16_free\0")
        .expect("failed to load cef_string_userfree_utf16_free");
    let cef_register_extension: Symbol<CefRegisterExtensionFn> = lib
        .get(b"cef_register_extension\0")
        .expect("failed to load cef_register_extension");
    let cef_process_message_create: Symbol<CefProcessMessageCreateFn> = lib
        .get(b"cef_process_message_create\0")
        .expect("failed to load cef_process_message_create");
    let cef_v8_value_create_string: Symbol<CefV8ValueCreateStringFn> = lib
        .get(b"cef_v8value_create_string\0")
        .expect("failed to load cef_v8value_create_string");
    let cef_v8_value_create_function: Symbol<CefV8ValueCreateFunctionFn> = lib
        .get(b"cef_v8value_create_function\0")
        .expect("failed to load cef_v8value_create_function");

    CefLibrary {
        _lib: lib,
        cef_execute_process,
        cef_string_utf8_to_utf16,
        cef_string_utf16_set,
        cef_string_userfree_utf16_free,
        cef_register_extension,
        cef_process_message_create,
        cef_v8_value_create_string,
        cef_v8_value_create_function
    }
});

fn get_cef_path(relative_path: &str) -> Result<PathBuf, std::io::Error> {
    let cef_path = current_exe()?
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not get parent directory"
            )
        })?;

    cef_path
        .join(relative_path)
        .canonicalize()
}
