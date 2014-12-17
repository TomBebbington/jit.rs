use raw::{
    jit_value_t,
    jit_value_create,
    jit_value_is_addressable,
    jit_value_is_temporary,
    jit_value_get_type,
    jit_value_set_addressable
};
use function::UncompiledFunction;
use std::kinds::marker::ContravariantLifetime;
use types::Type;
use util::NativeRef;
/// Values form the backbone of the storage system in `libjit`.
/// Every value in the system, be it a constant, a local variable, or a
/// temporary result, is represented by an object of type `Value`. The JIT then
/// allocates registers or memory locations to the values as appropriate.
#[deriving(Clone, PartialEq)]
pub struct Value<'a> {
    _value: jit_value_t,
    marker: ContravariantLifetime<'a>
}
impl<'a> NativeRef for Value<'a> {
    #[inline(always)]
    /// Convert to a native pointer
    unsafe fn as_ptr(&self) -> jit_value_t {
        self._value
    }
    #[inline(always)]
    /// Convert from a native pointer
    unsafe fn from_ptr(ptr:jit_value_t) -> Value<'a> {
        Value {
            _value: ptr,
            marker: ContravariantLifetime::<'a>
        }
    }
}
impl<'a> Value<'a> {
    #[inline(always)]
    /// Create a new value in the context of a function's current block.
    /// The value initially starts off as a block-specific temporary. It will be
    /// converted into a function-wide local variable if it is ever referenced
    /// from a different block.
    pub fn new(func:&UncompiledFunction<'a>, value_type:Type) -> Value<'a> {
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
    /// Determine if a value is temporary.  i.e. its scope extends over a single
    /// block within its function.
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
    /// Set a flag on a value to indicate that it is addressable.
    /// This should be used when you want to take the address of a value (e.g. 
    /// `&variable` in Rust/C).  The value is guaranteed to not be stored in a
    /// register across a function call.
    #[inline]
    pub fn set_addressable(&self) -> () {
        unsafe {
            jit_value_set_addressable(self.as_ptr())
        }
    }
}