use bindings::*;
use compilable::Compilable;
use context::Context;
use function::Function;
use util::NativeRef;
use std::str::raw::from_c_str;
/// An ELF binary reader
native_ref!(ReadElf, _reader, jit_readelf_t)
impl ReadElf {
	/// Open a new ELF binary
	pub fn new(filename:&str) -> ReadElf {
		unsafe {
			let this:ReadElf = NativeRef::from_ptr(RawPtr::null());
			jit_readelf_open(&mut this.as_ptr(), filename.to_c_str().unwrap(), 0);
			this
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
	/// Get a simple in the ELF binary
	pub unsafe fn get_symbol<T>(&self, symbol:&str) -> *T {
		jit_readelf_get_symbol(self.as_ptr(), symbol.to_c_str().unwrap()) as *T
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
	pub fn new(lib_name:&str) -> WriteElf {
		unsafe {
			NativeRef::from_ptr(jit_writeelf_create(lib_name.to_c_str().unwrap()))
		}
	}
	#[inline]
	/// Write to the filename given
	pub fn write(&self, filename:&str) -> bool {
		unsafe {
			jit_writeelf_write(self.as_ptr(), filename.to_c_str().unwrap()) != 0
		}
	}
	#[inline]
	/// Add a function to the ELF
	pub fn add_function(&self, func:&Function, name:&str) -> bool {
		unsafe {
			jit_writeelf_add_function(self.as_ptr(), func.as_ptr(), name.to_c_str().unwrap()) != 0
		}
	}
	#[inline]
	/// Add a dependency to the ELF
	pub fn add_needed(&self, lib_name:&str) -> bool {
		unsafe  {
			jit_writeelf_add_needed(self.as_ptr(), lib_name.to_c_str().unwrap()) != 0
		}
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