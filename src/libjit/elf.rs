use bindings::*;
use context::Context;
use function::Function;
use util::NativeRef;
use std::fmt::Show;
use std::str::raw::from_c_str;
use std::c_str::ToCStr;
/// An ELF binary reader
native_ref!(ReadElf, _reader, jit_readelf_t)
impl ReadElf {
	/// Open a new ELF binary
	pub fn new<S:ToCStr+Show>(filename:S) -> ReadElf {
		unsafe {
			let mut this = RawPtr::null();
			let code = filename.with_c_str(|c_name|
				jit_readelf_open(&mut this, c_name, 0)
			);
			if this.is_null() {
				fail!("'{}' couldn't be opened due to {}", filename, code);
			} else {
				NativeRef::from_ptr(this)
			}
		}
	}
	#[inline]
	/// Get the name of this ELF binary
	pub fn get_name(&self) -> String {
		unsafe {
			from_c_str(jit_readelf_get_name(self.as_ptr()))
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
	pub unsafe fn get_symbol<T, S:ToCStr>(&self, symbol:S) -> *T {
		symbol.with_c_str(|c_symbol|
			jit_readelf_get_symbol(self.as_ptr(), c_symbol) as *T
		)
	}
}
impl Drop for ReadElf {
	#[inline]
	fn drop(&mut self) {
		unsafe {
			jit_readelf_close(self.as_ptr())
		}
	}
}

/// An ELF binary reader
native_ref!(WriteElf, _writer, jit_writeelf_t)
impl WriteElf {
	#[inline]
	/// Create a new ELF binary reader
	pub fn new<S:ToCStr>(lib_name:S) -> WriteElf {
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
	pub fn add_function<S:ToCStr>(&self, func:&Function, name:S) -> bool {
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
impl Drop for WriteElf {
	#[inline]
	fn drop(&mut self) {
		unsafe {
			jit_writeelf_destroy(self.as_ptr())
		}
	}
}
#[test]
fn test_elf() {
	let elf = ReadElf::new("/usr/lib/libjit.so.0");
	assert_eq!(elf.get_name().as_slice(), "libjit.so.0");
}