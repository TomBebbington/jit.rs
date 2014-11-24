use bindings::{
    jit_label_t,
    jit_function_reserve_label
};
use function::UncompiledFunction;
use std::kinds::marker::ContravariantLifetime;
use util::NativeRef;
#[deriving(PartialEq)]
/// A label in the code that can be branched to in instructions
pub struct Label<'a> {
    _label: jit_label_t,
    marker: ContravariantLifetime<'a>
}
impl<'a> Label<'a> {
    #[inline(always)]
    /// Create a new label
    pub fn new(func:&UncompiledFunction<'a>) -> Label<'a> {
        unsafe {
            Label {
                _label: jit_function_reserve_label(func.as_ptr()),
                marker: ContravariantLifetime::<'a>
            }
        }
    }
    /// Get the value of this label
    #[inline(always)]
    pub fn get_value(&self) -> uint {
        self._label as uint
    }
}