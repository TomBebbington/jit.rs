use bindings::{
    jit_context_t,
    jit_context_create,
    jit_context_destroy,
    jit_context_build_start,
    jit_context_build_end
};
use std::kinds::marker::ContravariantLifetime;
use util::NativeRef;
/// Holds all of the functions you have built and compiled. There can be
/// multiple, but normally there is only one.
native_ref!(Context, _context, jit_context_t, ContravariantLifetime)
impl<'a> Context<'a> {
    #[inline(always)]
    /// Create a new JIT Context
    pub fn new() -> Context<'a> {
        unsafe {
            NativeRef::from_ptr(jit_context_create())
        }
    }
    #[inline(always)]
    /// Run a closure that can generate IR
    pub fn build<R>(&'a self, cb: proc() -> R) -> R {
        unsafe {
            jit_context_build_start(self.as_ptr());
            let value = cb();
            jit_context_build_end(self.as_ptr());
            value
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for Context<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            jit_context_destroy(self.as_ptr());
        }
    }
}