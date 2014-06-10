use bindings::{
	jit_value_t,
	jit_value_is_addressable,
	jit_value_is_temporary,
	jit_value_get_context,
	jit_value_get_type,
	jit_value_get_function
};
use context::Context;
use function::Function;
use types::Type;
use util::NativeRef;
#[deriving(Clone)]
/// A Value that is being JIT compiled
native_ref!(Value, _value, jit_value_t)
impl Value {
	/// Get the type of the value
	pub fn get_type(&self) -> Type {
		unsafe {
			let ty = jit_value_get_type(self.as_ptr());
			NativeRef::from_ptr(ty)
		}
	}
	/// Get the function which owns this value
	pub fn get_function(&self) -> Function {
		unsafe {
			NativeRef::from_ptr(jit_value_get_function(self.as_ptr()))
		}
	}
	/// Get the context which this value belongs to
	pub fn get_context(&self) -> Context {
		unsafe {
			NativeRef::from_ptr(jit_value_get_context(self.as_ptr()))
		}
	}
	/// Return true if the value is temporary so its scope extends over a single block within its function
	pub fn is_temp(&self) -> bool {
		unsafe {
			jit_value_is_temporary(self.as_ptr()) != 0
		}
	}
	/// Return true if the value can have its address taken from it
	pub fn is_addressable(&self) -> bool {
		unsafe {
			jit_value_is_addressable(self.as_ptr()) != 0
		}
	}
}
