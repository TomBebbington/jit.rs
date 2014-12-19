use raw::*;
use context::Context;
use function::CompiledFunction;
use libc::c_uint;
use util::NativeRef;
use std::fmt::{Show, Formatter};
use std::fmt::Result as FResult;
use std::kinds::marker::ContravariantLifetime;
use std::mem;
use std::iter::Iterator;
use std::c_str::ToCStr;
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
impl<'a> Iterator<String> for Needed<'a> {
    fn next(&mut self) -> Option<String> {
        let index = self.index;
        self.index += 1;
        unsafe {
            if index < self.length {
                let c_name = jit_readelf_get_needed(self._reader, index);
                let name = String::from_raw_buf(mem::transmute(c_name));
                Some(name)
            } else {
                None
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        ((self.length - self.index) as uint, None)
    }
}
/// An ELF binary reader
pub struct ReadElf {
    _reader: jit_readelf_t
}
native_ref!(ReadElf, _reader, jit_readelf_t);
#[deriving(Copy)]
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
impl Show for ReadElfErrorCode {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt, "{}", self.description())
    }
}
impl Error for ReadElfErrorCode {
    fn description(&self) -> &str {
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
pub struct ReadElfError<S> {
    filename: S,
    error: ReadElfErrorCode
}
impl<S:Show> Show for ReadElfError<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt, "'{}': {}", self.filename, self.error.description())
    }
}
impl<S> Error for ReadElfError<S> where S:Send + Show + ToCStr {
    fn description(&self) -> &str {
        self.error.description()
    }
    fn detail(&self) -> Option<String> {
        Some(self.to_string())
    }
}
impl ReadElf {
    /// Open a new ELF binary
    pub fn new<S:ToCStr>(filename:S) -> Result<ReadElf, ReadElfError<S>> {
        unsafe {
            let mut this = RawPtr::null();
            let code = filename.with_c_str(|c_name|
                jit_readelf_open(&mut this, c_name, 0)
            );
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
            String::from_raw_buf(mem::transmute(jit_readelf_get_name(self.as_ptr())))
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
    pub unsafe fn get_symbol<T, S:ToCStr>(&self, symbol:S) -> &mut T {
        symbol.with_c_str(|c_symbol|
            mem::transmute(jit_readelf_get_symbol(self.as_ptr(), c_symbol))
        )
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
pub struct WriteElf {
    _writer: jit_writeelf_t
}
native_ref!(WriteElf, _writer, jit_writeelf_t);
impl WriteElf {
    #[inline]
    /// Create a new ELF binary reader
    pub fn new<S:ToCStr>(lib_name:S) -> WriteElf {
        lib_name.with_c_str(|c_lib_name| unsafe {
            NativeRef::from_ptr(jit_writeelf_create(c_lib_name))
        })
    }
    #[inline]
    /// Write to the filename given (not implemented by LibJIT yet, so there's no point to this yet
    /// but I'm sure GNU will hear the people sing the songs of angry men soon enough)
    pub fn write<S:ToCStr>(&self, filename:S) -> bool {
        filename.with_c_str(|c_filename| unsafe {
            jit_writeelf_write(self.as_ptr(), c_filename) != 0
        })
    }
    #[inline]
    /// Add a function to the ELF
    pub fn add_function<S:ToCStr>(&self, func:&CompiledFunction, name:S) -> bool {
        name.with_c_str(|c_name| unsafe {
            jit_writeelf_add_function(self.as_ptr(), func.as_ptr(), c_name) != 0
        })
    }
    #[inline]
    /// Add a dependency to the ELF
    pub fn add_needed<S:ToCStr>(&self, lib_name:S) -> bool {
        lib_name.with_c_str(|c_lib_name| unsafe {
            jit_writeelf_add_needed(self.as_ptr(), c_lib_name) != 0
        })
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