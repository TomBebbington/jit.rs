use raw::*;
use std::mem;
use util::NativeRef;
use {CompiledFunction, Type, UncompiledFunction};
/// Holds all of the functions you have built and compiled. There can be
/// multiple, but normally there is only one.
pub struct Context {
    _context: jit_context_t
}
native_ref!(Context, _context, jit_context_t)

/// A context that is in the build phase while generating IR
#[allow(missing_copy_implementations)]
pub struct Builder {
    _context: jit_context_t
}
native_ref!(Builder, _context, jit_context_t)

impl Context {
    #[inline(always)]
    /// Create a new JIT Context
    pub fn new() -> Context {
        unsafe {
            NativeRef::from_ptr(jit_context_create())
        }
    }
    #[inline(always)]
    /// Lock the context so you can safely generate IR
    pub fn build<'a, R, F:FnOnce(&'a Builder) -> R>(&'a self, cb: F) -> R {
        unsafe {
            jit_context_build_start(self.as_ptr());
            let r = cb(mem::transmute(self));
            jit_context_build_end(self.as_ptr());
            r
        }
    }
    #[inline(always)]
    /// Lock the context so you can safely generate IR in a new function on the context which is
    /// compiled for you
    pub fn build_func<'a, F:FnOnce(&UncompiledFunction<'a>)>(&'a self, signature: Type, cb: F) -> CompiledFunction<'a> {
        unsafe {
            jit_context_build_start(self.as_ptr());
            let func = UncompiledFunction::new(mem::transmute(self), signature.clone());
            cb(mem::transmute(self));
            jit_context_build_end(self.as_ptr());
            func.compile()
        }
    }
}

#[unsafe_destructor]
impl Drop for Context {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            jit_context_destroy(self.as_ptr());
        }
    }
}