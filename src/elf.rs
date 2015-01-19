use raw::*;
use context::Context;
use function::CompiledFunction;
use libc::c_uint;
use util::NativeRef;
use std::ffi::{self, CString};
use std::fmt::{Show, Formatter};
use std::fmt::Result as FResult;
use std::marker::ContravariantLifetime;
use std::{mem, ptr};
use std::iter::Iterator;
use std::error::Error;
/// An ELF dependency iterator
pub struct Needed<'a> {
    _reader: jit_readelf_t,
    index: c_uint,
    length: c_uint,
    marker: ContravariantLifetime<'a>
}
impl<'a> Needed<'a> {
    #[inline(always)]
    fn new(read:&'a ReadElf) -> Needed<'a> {
        unsafe {
            Needed {
                _reader: read.as_ptr(),
                index: 0 as c_uint,
                length: jit_readelf_num_needed(read.as_ptr()),
                marker: ContravariantLifetime::<'a>
            }
        }
    }
}
impl<'a> Iterator for Needed<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        let index = self.index;
        self.index += 1;
        unsafe {
            if index < self.length {
                let c_name = jit_readelf_get_needed(self._reader, index);
                let bytes = ffi::c_str_to_bytes(&c_name);
                Some(String::from_utf8_lossy(bytes).into_owned())
            } else {
                None
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        ((self.length - self.index) as usize, None)
    }
}
/// An ELF binary reader
native_ref!(ReadElf {
    _reader: jit_readelf_t
});
#[repr(i32)]
/// An error from trying to open the ELF
pub enum ReadElfErrorCode {
    /// The file couldn't be opened
    CannotOpen,
    /// The file isn't an ELF
    NotElf,
    /// The ELF is for a different architecture
    WrongArch,
    /// The ELF is badly formatted
    BadFormat,
    /// The ELF is too big to be loaded
    Memory
}
impl Copy for ReadElfErrorCode {}
impl Show for ReadElfErrorCode {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt, "{}", self.description())
    }
}
impl Error for ReadElfErrorCode {
    fn description(&self) -> &'static str {
        match *self {
            ReadElfErrorCode::CannotOpen => "Could not open the file",
            ReadElfErrorCode::NotElf => "Not an ELF-format binary",
            ReadElfErrorCode::WrongArch => "Wrong architecture for local system",
            ReadElfErrorCode::BadFormat => "ELF file, but badly formatted",
            ReadElfErrorCode::Memory => "Insufficient memory to load the file"
        }
    }
}
#[deriving(Copy)]
/// An error from trying to open the ELF, including the filename
pub struct ReadElfError<'a> {
    filename: &'a str,
    error: ReadElfErrorCode
}
impl<'a> Show for ReadElfError<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt, "'{}': {}", self.filename, self.error.description())
    }
}
impl ReadElf {
    /// Open a new ELF binary
    pub fn new(filename:&str) -> Result<ReadElf, ReadElfError> {
        unsafe {
            let c_name = CString::from_slice(filename.as_bytes());
            let mut this = ptr::null_mut();
            let code = jit_readelf_open(&mut this, mem::transmute(c_name.as_ptr()), 0);
            if code == 0 {
                Ok(NativeRef::from_ptr(this))
            } else {
                Err(ReadElfError {
                    filename: filename,
                    error: mem::transmute(code)
                })
            }
        }
    }
    #[inline]
    /// Get the name of this ELF binary
    pub fn get_name(&self) -> String {
        unsafe {
            let c_name = jit_readelf_get_name(self.as_ptr());
            let bytes = ffi::c_str_to_bytes(&c_name);
            String::from_utf8_lossy(bytes).into_owned()
        }
    }
    #[inline]
    pub fn add_to_context(&self, ctx:&Context) {
        unsafe {
            jit_readelf_add_to_context(self.as_ptr(), ctx.as_ptr())
        }
    }
    #[inline]
    /// Get a symbol in the ELF binary
    pub unsafe fn get_symbol<T>(&self, symbol:&str) -> &mut T {
        let c_sym = CString::from_slice(symbol.as_bytes());
        mem::transmute(jit_readelf_get_symbol(self.as_ptr(), mem::transmute(c_sym.as_ptr())))
    }
    #[inline]
    /// Iterate over the needed libraries
    pub fn needed(&self) -> Needed {
        Needed::new(self)
    }
}
#[unsafe_destructor]
impl Drop for ReadElf {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            jit_readelf_close(self.as_ptr())
        }
    }
}

/// An ELF binary reader
native_ref!(WriteElf {
    _writer: jit_writeelf_t
});
impl WriteElf {
    #[inline]
    /// Create a new ELF binary reader
    pub fn new(lib_name:&str) -> WriteElf {
        unsafe {
            let c_lib = CString::from_slice(lib_name.as_bytes());
            NativeRef::from_ptr(jit_writeelf_create(mem::transmute(c_lib.as_ptr())))
        }
    }
    #[inline]
    /// Write to the filename given (not implemented by LibJIT yet, so there's no point to this yet
    /// but I'm sure GNU will hear the people sing the songs of angry men soon enough)
    pub fn write(&self, filename:&str) -> bool {
        unsafe {
            let c_filename = CString::from_slice(filename.as_bytes());
            jit_writeelf_write(self.as_ptr(), mem::transmute(c_filename.as_ptr())) != 0
        }
    }
    #[inline]
    /// Add a function to the ELF
    pub fn add_function(&self, func:&CompiledFunction, name:&str) -> bool {
        unsafe {
            let c_name = CString::from_slice(name.as_bytes());
            jit_writeelf_add_function(self.as_ptr(), func.as_ptr(), mem::transmute(c_name.as_ptr())) != 0
        }
    }
    #[inline]
    /// Add a dependency to the ELF
    pub fn add_needed(&self, lib_name:&str) -> bool {
        unsafe {
            let c_lib = CString::from_slice(lib_name.as_bytes());
            jit_writeelf_add_needed(self.as_ptr(), mem::transmute(c_lib.as_ptr())) != 0
        }
    }
}
#[unsafe_destructor]
impl Drop for WriteElf {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            jit_writeelf_destroy(self.as_ptr())
        }
    }
}