use raw::{
    jit_label_t,
    jit_function_reserve_label
};
use function::UncompiledFunction;
use std::marker::ContravariantLifetime;
use std::fmt;
use std::ops::{Deref, DerefMut};
use util::NativeRef;
#[derive(PartialEq)]
/// A label in the code that can be branched to in instructions
pub struct Label<'a> {
    _label: jit_label_t,
    marker: ContravariantLifetime<'a>
}
impl<'a> fmt::Display for Label<'a> {
    fn fmt(&self, fmt:&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self._label)
    }
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
}
impl<'a> Deref for Label<'a> {
    type Target = u64;
    fn deref(&self) -> &u64 {
        &self._label
    }
}
impl<'a> DerefMut for Label<'a> {
    fn deref_mut(&mut self) -> &mut u64 {
        &mut self._label
    }
}