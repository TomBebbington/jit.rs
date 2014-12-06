use raw::*;
use context::Context;
use function::CompiledFunction;
use libc::c_uint;
use util::NativeRef;
use std::fmt::Show;
use std::kinds::marker::ContravariantLifetime;
use std::mem::transmute;
use std::iter::Iterator;
use std::c_str::ToCStr;
/// An ELF dependency iterator
pub struct Needed<'a> {
    _reader: jit_readelf_t,
    index: c_uint,
    length: c_uint,
    marker: ContravariantLifetime<'a>
}
impl<'a> Needed<'a> {
    #[inline(always)]
    fn new(read:&ReadElf<'a>) -> Needed<'a> {
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
                let name = String::from_raw_buf(transmute(c_name));
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
native_ref!(ReadElf, _reader, jit_readelf_t, ContravariantLifetime)
impl<'a> ReadElf<'a> {
    /// Open a new ELF binary
    pub fn new<S:ToCStr+Show>(filename:S) -> ReadElf<'a> {
        unsafe {
            let mut this = RawPtr::null();
            let code = filename.with_c_str(|c_name|
                jit_readelf_open(&mut this, c_name, 0)
            );
            if this.is_null() {
                panic!("'{}' couldn't be opened due to {}", filename, code);
            } else {
                NativeRef::from_ptr(this)
            }
        }
    }
    #[inline]
    /// Get the name of this ELF binary
    pub fn get_name(&self) -> String {
        unsafe {
            String::from_raw_buf(transmute(jit_readelf_get_name(self.as_ptr())))
        }
    }
    #[inline]
    pub fn add_to_context(&self, ctx:&Context<'a>) {
        unsafe {
            jit_readelf_add_to_context(self.as_ptr(), ctx.as_ptr())
        }
    }
    #[inline]
    /// Get a symbol in the ELF binary
    pub unsafe fn get_symbol<T, S:ToCStr>(&self, symbol:S) -> &'a mut T {
        symbol.with_c_str(|c_symbol|
            transmute(jit_readelf_get_symbol(self.as_ptr(), c_symbol))
        )
    }
    #[inline]
    /// Iterate over the needed libraries
    pub fn needed<'a>(&'a self) -> Needed<'a> {
        Needed::new(self)
    }
}
#[unsafe_destructor]
impl<'a> Drop for ReadElf<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            jit_readelf_close(self.as_ptr())
        }
    }
}

/// An ELF binary reader
native_ref!(WriteElf, _writer, jit_writeelf_t, ContravariantLifetime)
impl<'a> WriteElf<'a> {
    #[inline]
    /// Create a new ELF binary reader
    pub fn new<S:ToCStr>(lib_name:S) -> WriteElf<'a> {
        lib_name.with_c_str(|c_lib_name| unsafe {
            NativeRef::from_ptr(jit_writeelf_create(c_lib_name))
        })
    }
    #[inline]
    /// Write to the filename given (not implemented by LibJIT yet)
    pub fn write<S:ToCStr>(&self, filename:S) -> bool {
        filename.with_c_str(|c_filename| unsafe {
            jit_writeelf_write(self.as_ptr(), c_filename) != 0
        })
    }
    #[inline]
    /// Add a function to the ELF
    pub fn add_function<S:ToCStr>(&self, func:&'a CompiledFunction<'a>, name:S) -> bool {
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
impl<'a> Drop for WriteElf<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            jit_writeelf_destroy(self.as_ptr())
        }
    }
}