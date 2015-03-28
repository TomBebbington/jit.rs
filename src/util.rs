use libc::*;
use std::fmt::Error;
use std::{mem, str};

pub fn dump<F>(cb: F) -> Result<String, Error> where F:FnOnce(*mut FILE) {
    unsafe {
        let mut pair = [0, 0];
        if pipe(pair.as_mut_ptr()) == -1 {
            return Err(Error)
        }
        let file = fdopen(pair[1], b"w".as_ptr() as *const c_char);
        if file.is_null() {
            return Err(Error)
        }
        cb(file);
        fclose(file);
        let file = fdopen(pair[0], b"r".as_ptr() as *const c_char);
        if file.is_null() {
            return Err(Error)
        }
        let mut chars:[c_char; 64] = mem::zeroed();
        let mut text = String::new();
        loop {
            let ptr = fgets(chars.as_mut_ptr(), chars.len() as c_int, file);
            let bytes = chars.split(|&c| c == 0).next().unwrap();
            let bytes = mem::transmute(bytes);
            text.push_str(str::from_utf8_unchecked(bytes));
            if ptr.is_null() {
                break
            }
        }
        fclose(file);
        Ok(text)
    }
}
pub fn from_ptr_opt<R>(ptr: *mut c_void) -> Option<R> where R:From<*mut c_void> {
    if ptr.is_null() {
        None
    } else {
        Some(from_ptr(ptr))
    }
}
pub fn from_ptr_oom<R>(ptr: *mut c_void) -> R where R:From<*mut c_void> {
    use alloc::oom;
    if ptr.is_null() {
        oom();
    } else {
        from_ptr(ptr)
    }
}

pub fn from_ptr<R>(ptr: *mut c_void) -> R where R:From<*mut c_void> {
    From::from(ptr)
}
