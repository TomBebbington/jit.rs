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
pub struct Context<T = ()> {
    _context: jit_context_t,
    marker: PhantomData<T>
}
native_ref!(Context<T>, _context: jit_context_t, marker = PhantomData);

/// A context that is in the build phase while generating IR
pub struct Builder<T> {
    _context: jit_context_t,
    marker: PhantomData<T>
}
impl<T> Deref for Builder<T> {
    type Target = Context<T>;
    fn deref(&self) -> &Context<T> {
        unsafe {
            mem::transmute(self)
        }
    }
}
native_ref!(Builder<T>, _context: jit_context_t, marker = PhantomData);

impl<T = ()> Index<i32> for Context<T> {
    type Output = T;
    fn index(&self, index: i32) -> &T {
        unsafe {
            let meta = jit_context_get_meta(self.into(), index);
            if meta.is_null() {
                panic!("No such index {} on Context", index)
            }
            mem::transmute(meta)
        }
    }
}
impl<T = ()> IndexMut<i32> for Context<T> {
    fn index_mut(&mut self, index: i32) -> &mut T {
        unsafe {
            let meta = jit_context_get_meta(self.into(), index);
            if meta.is_null() {
                let boxed = Box::new(mem::uninitialized::<T>());
                if jit_context_set_meta(self.into(), index, mem::transmute(boxed), Some(::free_data::<T>)) == 0 {
                    oom()
                } else {
                    mem::transmute(jit_context_get_meta(self.into(), index))
                }
            } else {
                mem::transmute(meta)
            }
        }
    }
}
impl<T = ()> Context<T> {
    #[inline(always)]
    /// Create a new JIT Context
    pub fn new() -> Context<T> {
        unsafe {
            from_ptr(jit_context_create())
        }
    }
    #[inline(always)]
    /// Lock the context so you can safely generate IR
    pub fn build<'a, R, F:FnOnce(Builder<T>) -> R>(&'a mut self, cb: F) -> R {
        unsafe {
            jit_context_build_start(self.into());
            let r = cb(Builder { _context: self.into(), marker: PhantomData});
            jit_context_build_end(self.into());
            r
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
impl<'a, T> IntoIterator for &'a Context<T> {
    type IntoIter = Functions<'a>;
    type Item = &'a Func;
    fn into_iter(self) -> Functions<'a> {
        self.functions()
    }
}
#[unsafe_destructor]
impl<T> Drop for Context<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            jit_context_destroy(self.into());
        }
    }
}

pub struct Functions<'a> {
    context: jit_context_t,
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
