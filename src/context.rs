use raw::*;
use std::mem;
use std::kinds::marker::{NoCopy, NoSync, NoSend};
use util::NativeRef;
use {CompiledFunction, Type, UncompiledFunction};
/// Holds all of the functions you have built and compiled. There can be
/// multiple, but normally there is only one.
pub struct Context {
    _context: jit_context_t
}
native_ref!(Context, _context, jit_context_t);

/// A context that is in the build phase while generating IR
pub struct Builder {
    _context: jit_context_t,
    no_sync: NoSync,
    no_send: NoSend,
    no_copy: NoCopy
}
impl NativeRef for Builder {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> jit_context_t {
        self._context
    }
    #[inline(always)]
    unsafe fn from_ptr(ptr:jit_context_t) -> Builder {
        Builder {
            _context: ptr,
            no_sync: NoSync,
            no_send: NoSend,
            no_copy: NoCopy
        }
    }
}

impl Context {
    unsafe fn as_builder(&mut self) -> Builder {
        Builder{
            _context: self._context,
            no_sync: NoSync,
            no_send: NoSend,
            no_copy: NoCopy
        }
    }
    #[inline(always)]
    /// Create a new JIT Context
    pub fn new() -> Context {
        unsafe {
            jit_exception_set_handler(Some(::handle_exception));
            NativeRef::from_ptr(jit_context_create())
        }
    }
    #[inline(always)]
    /// Lock the context so you can safely generate IR
    pub fn build<R, F:FnOnce(&Builder) -> R>(&mut self, cb: F) -> R {
        unsafe {
            jit_context_build_start(self.as_ptr());
            let r = cb(&self.as_builder());
            jit_context_build_end(self.as_ptr());
            r
        }
    }
    #[inline(always)]
    /// Lock the context so you can safely generate IR in a new function on the context which is
    /// compiled for you
    pub fn build_func<'a, F:FnOnce(&UncompiledFunction<'a>)>(&'a mut self, signature: Type, cb: F) -> CompiledFunction<'a> {
        unsafe {
            jit_context_build_start(self.as_ptr());
            let builder = self.as_builder();
            let func = UncompiledFunction::new(mem::copy_lifetime(self, &builder), signature.clone());
            cb(&func);
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