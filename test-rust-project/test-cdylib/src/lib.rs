use std::ffi::CString;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn hello() -> *mut c_char {
    let s = CString::new("Hello from cdylib!").unwrap();
    s.into_raw()
}
