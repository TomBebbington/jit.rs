use raw::*;
use alloc::oom;
use function::{Func, CompiledFunction, UncompiledFunction};
use types::Ty;
use util::{from_ptr, from_ptr_opt};
use std::marker::PhantomData;
use std::{mem, ptr};
use std::ops::{Deref, Index, IndexMut};
use std::iter::IntoIterator;
/// Holds all of the functions you have built and compiled. There can be
/// multiple, but normally there is only one.
pub struct Context {
    _context: LLVMContextRef
}
native_ref!(Context, _context: LLVMContextRef);

/// A context that is in the build phase while generating IR
pub struct Builder {
    _context: LLVMContextRef,
    _builder: LLVMBuilderRef
}
impl Deref for Builder {
    type Target = Context;
    fn deref(&self) -> &Context {
        unsafe {
            mem::transmute(&self._context)
        }
    }
}
impl Drop for Builder {
    fn drop(&mut self) {
        LLVMDisposeBuilder(self._builder)
    }
}
impl From<Builder> for LLVMContextRef {
    fn from(builder: Builder) -> LLVMContextRef {
        builder._context
    }
}
impl From<Builder> for LLVMBuilderRef {
    fn from(builder: Builder) -> LLVMBuilderRef {
        builder._builder
    }
}
native_ref!(Builder, _context: LLVMContextRef);

impl Context {
    #[inline(always)]
    /// Create a new Context
    pub fn new() -> Context {
        unsafe {
            from_ptr(LLVMContextCreate())
        }
    }
    #[inline(always)]
    /// Lock the context so you can safely generate IR
    pub fn build<'a, R, F:FnOnce(Builder) -> R>(&'a mut self, cb: F) -> R {
        unsafe {
            cb(Builder {
                _context: self.into(),
                _builder: LLVMCreateBuilderInContext(self.into())
            })
        }
    }
    #[inline(always)]
    /// Lock the context so you can safely generate IR in a new function on the context which is
    /// compiled for you
    pub fn build_func<'a, F:FnOnce(&UncompiledFunction<'a>)>(&'a mut self, signature: &Ty, cb: F) -> CompiledFunction<'a> {
        unsafe {
            jit_context_build_start(self.into());
            let mut builder = Builder { _context: self.into(), marker: PhantomData::<T>};
            let func = UncompiledFunction::new::<T>(mem::transmute(&mut builder), signature);
            cb(&func);
            jit_context_build_end(self.into());
            func.compile()
        }
    }
    /// Iterate through the functions contained inside this context
    pub fn functions(&self) -> Functions {
        Functions {
            context: self.into(),
            last: ptr::null_mut(),
            lifetime: PhantomData,
        }
    }
}
impl<'a, T> IntoIterator for &'a Context {
    type IntoIter = Functions<'a>;
    type Item = &'a Func;
    fn into_iter(self) -> Functions<'a> {
        self.functions()
    }
}
impl<T> Drop for Context {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            jit_context_destroy(self.into());
        }
    }
}

pub struct Functions<'a> {
    context: LLVMContextRef,
    last: jit_function_t,
    lifetime: PhantomData<&'a ()>
}
impl<'a> Iterator for Functions<'a> {
    type Item = &'a Func;
    fn next(&mut self) -> Option<&'a Func> {
        unsafe {
            self.last = jit_function_next(self.context, self.last);
            from_ptr_opt(self.last)
        }
    }
}
