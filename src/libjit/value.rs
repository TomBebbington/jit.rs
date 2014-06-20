use bindings::{
	jit_value_t,
	jit_value_create,
	jit_value_is_addressable,
	jit_value_is_temporary,
	jit_value_get_context,
	jit_value_get_type,
	jit_value_get_function,
	jit_value_set_addressable
};
use compile::Compile;
use context::{
	Context,
	InContext
};
use function::Function;
use std::kinds::marker::ContravariantLifetime;
use std::ops::*;
use types::Type;
use util::NativeRef;
/**
 * Values form the backbone of the storage system in `libjit`.
 * Every value in the system, be it a constant, a local variable,
 * or a temporary result, is represented by an object of type
 * `Value`. The JIT then allocates registers or memory
 * locations to the values as appropriate.
*/
#[deriving(Clone, PartialEq)]
pub struct Value<'a> {
	_value: jit_value_t,
	marker: ContravariantLifetime<'a>
}
impl<'a> NativeRef for Value<'a> {
	#[inline]
	/// Convert to a native pointer
	unsafe fn as_ptr(&self) -> jit_value_t {
		self._value
	}
	#[inline]
	/// Convert from a native pointer
	unsafe fn from_ptr(ptr:jit_value_t) -> Value {
		Value {
			_value: ptr,
			marker: ContravariantLifetime::<'a>
		}
	}
}
impl<'a> InContext<'a> for Value<'a> {
	/// Get the context which this value belongs to
	fn get_context(&self) -> Context<'a> {
		unsafe {
			NativeRef::from_ptr(jit_value_get_context(self.as_ptr()))
		}
	}
}
impl<'a> Value<'a> {
	/**
	 * Create a new value in the context of a function's current block.
	 * The value initially starts off as a block-specific temporary.
	 * It will be converted into a function-wide local variable if
	 * it is ever referenced from a different block.
	 */
	pub fn new(func:&Function<'a>, value_type:&Type) -> Value<'a> {
		unsafe {
			let value = jit_value_create(func.as_ptr(), value_type.as_ptr());
			NativeRef::from_ptr(value)
		}
	}
	/// Get the type of the value
	pub fn get_type(&self) -> Type {
		unsafe {
			let ty = jit_value_get_type(self.as_ptr());
			NativeRef::from_ptr(ty)
		}
	}
	/// Get the function which owns this value
	#[inline]
	pub fn get_function(&self) -> Function<'a> {
		unsafe {
			NativeRef::from_ptr(jit_value_get_function(self.as_ptr()))
		}
	}
	/**
	 * Determine if a value is temporary.  i.e. its scope extends
	 * over a single block within its function.
	 */
	#[inline]
	pub fn is_temp(&self) -> bool {
		unsafe {
			jit_value_is_temporary(self.as_ptr()) != 0
		}
	}
	/// Determine if a value is addressable.
	#[inline]
	pub fn is_addressable(&self) -> bool {
		unsafe {
			jit_value_is_addressable(self.as_ptr()) != 0
		}
	}
	/* Set a flag on a value to indicate that it is addressable.
	 * This should be used when you want to take the address of a
	 * value (e.g. `&variable` in Rust/C).  The value is guaranteed
	 * to not be stored in a register across a function call.
	 */
	#[inline]
	pub fn set_addressable(&self) -> () {
		unsafe {
			jit_value_set_addressable(self.as_ptr())
		}
	}
}
impl<'a> Add<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn add(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_add(self, other)
	}
}
impl<'a> Sub<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn sub(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_sub(self, other)
	}
}
impl<'a> Mul<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn mul(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_mul(self, other)
	}
}
impl<'a> Div<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn div(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_div(self, other)
	}
}
impl<'a> Rem<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn rem(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_rem(self, other)
	}
}
impl<'a> Shl<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn shl(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_shl(self, other)
	}
}
impl<'a> Shr<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn shr(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_shr(self, other)
	}
}
impl<'a> BitAnd<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn bitand(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_and(self, other)
	}
}
impl<'a> BitOr<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn bitor(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_or(self, other)
	}
}
impl<'a> BitXor<Value<'a>, Value<'a>> for Value<'a> {
	#[inline(always)]
	fn bitxor(&self, other:&Value<'a>) -> Value<'a> {
		self.get_function().insn_xor(self, other)
	}
}
impl<'a> Neg<Value<'a>> for Value<'a> {
	#[inline(always)]
	fn neg(&self) -> Value<'a> {
		self.get_function().insn_neg(self)
	}
}
impl<'a> Not<Value<'a>> for Value<'a> {
	#[inline(always)]
	fn not(&self) -> Value<'a> {
		self.get_function().insn_not(self)
	}
}