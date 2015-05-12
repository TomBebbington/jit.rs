use raw::*;
use context::Context;
use function::CompiledFunction;
use util::from_ptr;
use libc::{c_uint, c_char};
use std::ffi::{self, CString};
use std::{fmt, str};
use std::marker::PhantomData;
use std::{mem, ptr};
use std::iter::Iterator;
use std::error::Error;
/// An ELF dependency iterator
pub struct Needed<'a> {
    _reader: jit_readelf_t,
    index: c_uint,
    length: c_uint,
    marker: PhantomData<&'a ()>
}
impl<'a> Needed<'a> {
    #[inline(always)]
    fn new(read:&'a ReadElf) -> Needed<'a> {
        unsafe {
            Needed {
                _reader: read.into(),
                index: 0,
                length: jit_readelf_num_needed(read.into()),
                marker: PhantomData
            }
        }
    }
}
impl<'a> Iterator for Needed<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let index = self.index;
        self.index += 1;
        unsafe {
            if index < self.length {
                let c_name = jit_readelf_get_needed(self._reader, index);
                Some(str::from_utf8(ffi::CStr::from_ptr(c_name).to_bytes()).unwrap())
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
pub struct ReadElf {
    _reader: jit_readelf_t
}
native_ref!(ReadElf, _reader: jit_readelf_t);
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
impl fmt::Debug for ReadElfErrorCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}
impl fmt::Display for ReadElfErrorCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
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
#[derive(Copy)]
/// An error from trying to open the ELF, including the filename
pub struct ReadElfError<'a> {
    filename: &'a str,
    error: ReadElfErrorCode
}
impl<'a> fmt::Display for ReadElfError<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "'{}': {}", self.filename, self.error.description())
    }
}
impl ReadElf {
    /// Open a new ELF binary
    pub fn new(filename:&str) -> Result<ReadElf, ReadElfError> {
        unsafe {
            let mut this = ptr::null_mut();
            let c_name = CString::new(filename.as_bytes()).unwrap();
            let code = jit_readelf_open(&mut this, mem::transmute(c_name.as_bytes().as_ptr()), 0);
            if code == 0 {
                Ok(from_ptr(this))
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
    pub fn get_name(&self) -> &str {
        unsafe {
            let c_name = jit_readelf_get_name(self.into());
            str::from_utf8(ffi::CStr::from_ptr(c_name).to_bytes()).unwrap()
        }
    }
    #[inline]
    pub fn add_to_context(&self, ctx:&Context) {
        unsafe {
            jit_readelf_add_to_context(self.into(), ctx.into())
        }
    }
    #[inline]
    /// Get a symbol in the ELF binary
    pub unsafe fn get_symbol<T>(&self, symbol:&str) -> &mut T {
        let c_sym = CString::new(symbol.as_bytes()).unwrap();
        mem::transmute(jit_readelf_get_symbol(self.into(), c_sym.as_bytes().as_ptr() as *const c_char))
    }
    #[inline]
    /// Iterate over the needed libraries
    pub fn needed(&self) -> Needed {
        Needed::new(self)
    }
}
impl Drop for ReadElf {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            jit_readelf_close(self.into())
        }
    }
}

/// An ELF binary reader
pub struct WriteElf {
    _writer: jit_writeelf_t
}
native_ref!(WriteElf, _writer: jit_writeelf_t);
impl WriteElf {
    #[inline]
    /// Create a new ELF binary reader
    pub fn new(lib_name:&str) -> WriteElf {
        unsafe {
            let c_lib = CString::new(lib_name.as_bytes()).unwrap();
            from_ptr(jit_writeelf_create(c_lib.as_bytes().as_ptr() as *const c_char))
        }
    }
    #[inline]
    /// Write to the filename given (not implemented by LibJIT yet, so there's no point to this yet
    /// but I'm sure GNU will hear the people sing the songs of angry men soon enough)
    pub fn write(&self, filename:&str) -> bool {
        unsafe {
            let c_filename = CString::new(filename.as_bytes()).unwrap();
            jit_writeelf_write(self.into(), c_filename.as_bytes().as_ptr() as *const c_char) != 0
        }
    }
    #[inline]
    /// Add a function to the ELF
    pub fn add_function(&self, func:&CompiledFunction, name:&str) -> bool {
        unsafe {
            let c_name = CString::new(name.as_bytes()).unwrap();
            jit_writeelf_add_function(self.into(), func.into(), c_name.as_bytes().as_ptr() as *const c_char) != 0
        }
    }
    #[inline]
    /// Add a dependency to the ELF
    pub fn add_needed(&self, lib_name:&str) -> bool {
        unsafe {
            let c_lib = CString::new(lib_name.as_bytes()).unwrap();
            jit_writeelf_add_needed(self.into(), c_lib.as_bytes().as_ptr() as *const c_char) != 0
        }
    }
}
impl Drop for WriteElf {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            jit_writeelf_destroy(self.into())
        }
    }
}
