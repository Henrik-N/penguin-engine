use std::ffi::CStr;

pub fn raw_c_string_to_string(c_string: &[std::os::raw::c_char]) -> String {
    let raw_c_string = unsafe {
        let ptr = c_string.as_ptr();
        CStr::from_ptr(ptr)
    };
    raw_c_string
        .to_str()
        .expect("Couldn't convert c string.")
        .to_owned()
}

// https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
pub unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    std::slice::from_raw_parts((p as *const T) as *const u8, std::mem::size_of::<T>())
}

