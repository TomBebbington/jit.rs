#![allow(deprecated)] // old_io, os

use libc::{c_void, FILE};
use std::ptr;
use std::fmt::Error;
/// A structure that wraps a native object
pub trait NativeRef {
    /// Returns the native reference encapsulated by this object
    unsafe fn as_ptr(&self) -> *mut c_void;
    /// Returns a wrapped version of the native reference given, even if the reference is null
    unsafe fn from_ptr(ptr:*mut c_void) -> Self;
}
#[inline(always)]
pub unsafe fn from_ptr<T>(ptr: *mut c_void) -> T where T:NativeRef {
    NativeRef::from_ptr(ptr)
}
impl<T> NativeRef for Option<T> where T:NativeRef {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> *mut c_void {
        match *self {
            Some(ref v) => v.as_ptr(),
            None => ptr::null_mut()
        }
    }
    #[inline(always)]
    unsafe fn from_ptr(ptr:*mut c_void) -> Option<T> {
        if ptr.is_null() {
            None
        } else {
            Some(from_ptr(ptr))
        }
    }
}

pub fn dump<F>(cb: F) -> Result<String, Error> where F:FnOnce(*mut FILE) {
    use std::old_io::pipe::PipeStream;
    use std::old_io::Reader;
    use std::os;
    use libc::{fdopen, fclose};
    unsafe {
        let pair = os::pipe().unwrap();
        let file = fdopen(pair.writer, b"w".as_ptr() as *const i8);
        cb(file);
        fclose(file);
        match PipeStream::open(pair.reader).read_to_end() {
            Ok(v) => Ok(String::from_utf8_unchecked(v)),
            Err(_) => Err(Error)
        }
    }
}
