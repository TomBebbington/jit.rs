use raw::*;
use alloc::oom;
use std::marker::PhantomData;
use std::any::TypeId;
use std::{hash, mem, ptr};
use std::iter::IntoIterator;
use util::{from_ptr, NativeRef};
use {AnyFunction, CompiledFunction, TypeRef, UncompiledFunction};
/// Holds all of the functions you have built and compiled. There can be
/// multiple, but normally there is only one.
native_ref!(Context {
    _context: jit_context_t
});

/// A context that is in the build phase while generating IR
pub struct Builder {
    _context: jit_context_t,
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
        }
    }
}

impl Context {
    unsafe fn as_builder(&mut self) -> Builder {
        from_ptr(self.as_ptr())
    }
    #[inline(always)]
    /// Create a new JIT Context
    pub fn new() -> Context {
        unsafe {
            from_ptr(jit_context_create())
        }
    }
    /// Get the tagged metadata of an object
    pub fn get_meta<T>(&self) -> Option<&T> where T:'static {
        unsafe {
            let id = hash::hash::<TypeId, hash::SipHasher>(&TypeId::of::<T>());
            mem::transmute(jit_context_get_meta(self.as_ptr(), id as i32))
        }
    }

    /// Tag the context with some metadata
    pub fn set_meta<T>(&self, data: Box<T>) where T:'static {
        unsafe {
            let id = hash::hash::<TypeId, hash::SipHasher>(&TypeId::of::<T>());
            if jit_context_set_meta(self.as_ptr(), id as i32, mem::transmute(data), Some(::free_data::<T>)) == 0 {
                oom()
            }
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
    pub fn build_func<'a, F:FnOnce(&UncompiledFunction<'a>)>(&'a mut self, signature: TypeRef, cb: F) -> CompiledFunction<'a> {
        unsafe {
            jit_context_build_start(self.as_ptr());
            let builder = self.as_builder();
            let func = UncompiledFunction::new(mem::copy_lifetime(self, &builder), signature);
            cb(&func);
            jit_context_build_end(self.as_ptr());
            func.compile()
        }
    }
    /// Iterate through the functions contained inside this context
    pub fn functions<'a>(&'a self) -> Functions<'a> {
        unsafe {
            Functions {
                context: self.as_ptr(),
                last: ptr::null_mut(),
                lifetime: PhantomData,
            }
        }
    }
}
impl<'a> IntoIterator for &'a Context {
    type IntoIter = Functions<'a>;
    type Item = AnyFunction<'a>;
    fn into_iter(self) -> Functions<'a> {
        self.functions()
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

pub struct Functions<'a> {
    context: jit_context_t,
    last: jit_function_t,
    lifetime: PhantomData<&'a ()>
}
impl<'a> Iterator for Functions<'a> {
    type Item = AnyFunction<'a>;
    fn next(&mut self) -> Option<AnyFunction<'a>> {
        unsafe {
            self.last = jit_function_next(self.context, self.last);
            from_ptr(self.last)
        }
    }
}
