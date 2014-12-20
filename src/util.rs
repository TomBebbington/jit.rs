use libc::{c_void, FILE};
use std::c_str::CString;
use std::mem;
use std::ptr::RawPtr;
/// A structure that wraps a native object
pub trait NativeRef {
    /// Returns the native reference encapsulated by this object
    unsafe fn as_ptr(&self) -> *mut c_void;
    /// Returns a wrapped version of the native reference given, even if the reference is null
    unsafe fn from_ptr(ptr:*mut c_void) -> Self;
    /// Returns a wrapped version of the native reference given
    unsafe fn from_opt_ptr(ptr:*mut c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(NativeRef::from_ptr(ptr))
        }
    }
    #[inline(always)]
    /// Works with the internal pointer in a closure
    unsafe fn with_ptr<F:Fn(*mut c_void) -> R, R>(&self, cb:F) -> R {
        cb(self.as_ptr())
    }
}
impl<T:NativeRef> NativeRef for Option<T> {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> *mut c_void {
        match *self {
            Some(ref v) => v.as_ptr(),
            None => RawPtr::null()
        }
    }
    #[inline(always)]
    unsafe fn from_ptr(ptr:*mut c_void) -> Option<T> {
        if ptr.is_null() {
            None
        } else {
            Some(NativeRef::from_ptr(ptr))
        }
    }
}
impl NativeRef for CString {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> *mut c_void {
        mem::transmute(self.as_ptr())
    }
    #[inline(always)]
    unsafe fn from_ptr(ptr:*mut c_void) -> CString {
        CString::new(mem::transmute(ptr), true)
    }
}

pub fn dump<F:FnOnce(*mut FILE)>(cb: F) -> String {
    use std::io::pipe::PipeStream;
    use std::os;
    use libc::{fdopen, fclose};
    unsafe {
        let pair = os::pipe().unwrap();
        let file = fdopen(pair.writer, b"w".as_ptr() as *const i8);
        cb(file);
        fclose(file);
        PipeStream::open(pair.reader).read_to_string().unwrap()
    }
}