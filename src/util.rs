use libc::c_void;
use std::c_str::{CString, ToCStr};
use std::mem::transmute;
use std::ptr::RawPtr;
use std::string::raw::from_buf;
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
    unsafe fn with_ptr<R>(&self, cb:|*mut c_void| -> R) -> R {
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
impl NativeRef for String {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> *mut c_void {
        transmute(self.to_c_str().unwrap())
    }
    #[inline(always)]
    unsafe fn from_ptr(ptr:*mut c_void) -> String {
        from_buf(transmute(ptr))
    }
    #[inline(always)]
    unsafe fn with_ptr<R>(&self, cb:|*mut c_void| -> R) -> R {
        self.with_c_str(transmute(cb))
    }
}
impl NativeRef for CString {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> *mut c_void {
        transmute(self.clone().unwrap())
    }
    #[inline(always)]
    unsafe fn from_ptr(ptr:*mut c_void) -> CString {
        CString::new(transmute(ptr), true)
    }
}