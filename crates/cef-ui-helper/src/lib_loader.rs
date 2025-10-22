use anyhow::Result;
use cef_ui_sys::{
    cef_app_t, cef_browser_t, cef_frame_t, cef_list_value_t, cef_main_args_t, cef_process_id_t,
    cef_process_message_t, cef_string_t, cef_string_userfree_t, cef_string_userfree_utf16_t,
    cef_string_utf16_t, cef_v8context_t, cef_v8handler_t, cef_v8value_t, char16_t
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
type CefV8ValueCreateFunctionFn = unsafe extern "C" fn(
    name: *const cef_string_t,
    handler: *mut cef_v8handler_t
) -> *mut cef_v8value_t;
type CefBrowserGetMainFrameFn =
    unsafe extern "C" fn(browser: *mut cef_browser_t) -> *mut cef_frame_t;
type CefBrowserIsValidFn = unsafe extern "C" fn(browser: *mut cef_browser_t) -> c_int;
type CefFrameIsMainFn = unsafe extern "C" fn(frame: *mut cef_frame_t) -> c_int;
type CefFrameSendProcessMessageFn = unsafe extern "C" fn(
    frame: *mut cef_frame_t,
    target_process: cef_process_id_t,
    message: *mut cef_process_message_t
) -> c_int;
type CefV8ContextGetGlobalFn =
    unsafe extern "C" fn(context: *mut cef_v8context_t) -> *mut cef_v8value_t;
type CefV8ValueSetValueByKeyFn = unsafe extern "C" fn(
    object: *mut cef_v8value_t,
    key: *const cef_string_t,
    value: *mut cef_v8value_t,
    attributes: c_int
) -> c_int;
type CefV8ValueGetValueByKeyFn = unsafe extern "C" fn(
    object: *mut cef_v8value_t,
    key: *const cef_string_t
) -> *mut cef_v8value_t;
type CefV8ValueGetStringValueFn =
    unsafe extern "C" fn(value: *mut cef_v8value_t) -> cef_string_userfree_t;
type CefProcessMessageGetArgumentListFn =
    unsafe extern "C" fn(message: *mut cef_process_message_t) -> *mut cef_list_value_t;
type CefListValueSetStringFn = unsafe extern "C" fn(
    list: *mut cef_list_value_t,
    index: usize,
    value: *const cef_string_t
) -> c_int;

pub struct CefLibrary {
    _lib: &'static Library,
    pub cef_execute_process: Symbol<'static, CefExecuteProcessFn>,
    pub cef_string_utf8_to_utf16: Symbol<'static, CefStringUtf8ToUtf16Fn>,
    pub cef_string_utf16_set: Symbol<'static, CefStringUtf16SetFn>,
    pub cef_string_userfree_utf16_free: Symbol<'static, CefStringUsefreeUtf16FreeFn>,
    pub cef_register_extension: Symbol<'static, CefRegisterExtensionFn>,
    pub cef_process_message_create: Symbol<'static, CefProcessMessageCreateFn>,
    pub cef_v8value_create_function: Symbol<'static, CefV8ValueCreateFunctionFn>,
    pub cef_browser_get_main_frame: Symbol<'static, CefBrowserGetMainFrameFn>,
    pub cef_browser_is_valid: Symbol<'static, CefBrowserIsValidFn>,
    pub cef_frame_is_main: Symbol<'static, CefFrameIsMainFn>,
    pub cef_frame_send_process_message: Symbol<'static, CefFrameSendProcessMessageFn>,
    pub cef_v8context_get_global: Symbol<'static, CefV8ContextGetGlobalFn>,
    pub cef_v8value_set_value_by_key: Symbol<'static, CefV8ValueSetValueByKeyFn>,
    pub cef_v8value_get_value_by_key: Symbol<'static, CefV8ValueGetValueByKeyFn>,
    pub cef_v8value_get_string_value: Symbol<'static, CefV8ValueGetStringValueFn>,
    pub cef_process_message_get_argument_list: Symbol<'static, CefProcessMessageGetArgumentListFn>,
    pub cef_list_value_set_string: Symbol<'static, CefListValueSetStringFn>
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
    let cef_v8value_create_function: Symbol<CefV8ValueCreateFunctionFn> = lib
        .get(b"cef_v8value_create_function\0")
        .expect("failed to load cef_v8value_create_function");
    let cef_browser_get_main_frame: Symbol<CefBrowserGetMainFrameFn> = lib
        .get(b"cef_browser_get_main_frame\0")
        .expect("failed to load cef_browser_get_main_frame");
    let cef_browser_is_valid: Symbol<CefBrowserIsValidFn> = lib
        .get(b"cef_browser_is_valid\0")
        .expect("failed to load cef_browser_is_valid");
    let cef_frame_is_main: Symbol<CefFrameIsMainFn> = lib
        .get(b"cef_frame_is_main\0")
        .expect("failed to load cef_frame_is_main");
    let cef_frame_send_process_message: Symbol<CefFrameSendProcessMessageFn> = lib
        .get(b"cef_frame_send_process_message\0")
        .expect("failed to load cef_frame_send_process_message");
    let cef_v8context_get_global: Symbol<CefV8ContextGetGlobalFn> = lib
        .get(b"cef_v8context_get_global\0")
        .expect("failed to load cef_v8context_get_global");
    let cef_v8value_set_value_by_key: Symbol<CefV8ValueSetValueByKeyFn> = lib
        .get(b"cef_v8value_set_value_by_key\0")
        .expect("failed to load cef_v8value_set_value_by_key");
    let cef_v8value_get_value_by_key: Symbol<CefV8ValueGetValueByKeyFn> = lib
        .get(b"cef_v8value_get_value_by_key\0")
        .expect("failed to load cef_v8value_get_value_by_key");
    let cef_v8value_get_string_value: Symbol<CefV8ValueGetStringValueFn> = lib
        .get(b"cef_v8value_get_string_value\0")
        .expect("failed to load cef_v8value_get_string_value");
    let cef_process_message_get_argument_list: Symbol<CefProcessMessageGetArgumentListFn> = lib
        .get(b"cef_process_message_get_argument_list\0")
        .expect("failed to load cef_process_message_get_argument_list");
    let cef_list_value_set_string: Symbol<CefListValueSetStringFn> = lib
        .get(b"cef_list_value_set_string\0")
        .expect("failed to load cef_list_value_set_string");

    CefLibrary {
        _lib: lib,
        cef_execute_process,
        cef_string_utf8_to_utf16,
        cef_string_utf16_set,
        cef_string_userfree_utf16_free,
        cef_register_extension,
        cef_process_message_create,
        cef_v8value_create_function,
        cef_browser_get_main_frame,
        cef_browser_is_valid,
        cef_frame_is_main,
        cef_frame_send_process_message,
        cef_v8context_get_global,
        cef_v8value_set_value_by_key,
        cef_v8value_get_value_by_key,
        cef_v8value_get_string_value,
        cef_process_message_get_argument_list,
        cef_list_value_set_string
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
