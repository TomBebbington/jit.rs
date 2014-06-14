use bindings::{
	jit_label_t,
	jit_function_reserve_label
};
use function::Function;
use std::kinds::marker::ContravariantLifetime;
use util::NativeRef;
#[deriving(PartialEq)]
/// A label in the code that can be branched to in instructions
pub struct Label<'a> {
	_label: jit_label_t,
	marker: ContravariantLifetime<'a>
}
impl<'a> Label<'a> {
	/// Create a new label
	pub fn new(func:&'a Function) -> Label<'a> {
		unsafe {
			Label {
				_label: jit_function_reserve_label(func.as_ptr()),
				marker: ContravariantLifetime::<'a>
			}
		}
	}
	/// Get the value of this label
	#[inline]
	pub fn get_value(&self) -> jit_label_t {
		self._label
	}
}