/* Copyright (c) 2014, Peter Nelson
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without 
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, 
 *    this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */
#![crate_id = "jit#0.1.2"]
#![comment = "LibJIT Bindings"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]
#![allow(raw_pointer_deriving, dead_code, non_camel_case_types)]
#![deny(unnecessary_parens, unrecognized_lint, unreachable_code, unnecessary_allocation, unnecessary_typecast, unnecessary_allocation, uppercase_variables, unused_must_use)]
#![feature(globs, phase, macro_rules)]
#![stable]

//! This crate wraps LibJIT in an idiomatic style.
//! For example, here's a quick example which makes a multiply function using LibJIT:
//! 
//! ```rust
//! extern crate jit;
//! use jit::{C_DECL, Context, Function, Type, Types, get_type};
//! fn main() {
//!     let cx = Context::new();
//!     cx.build(|| {
//!         // build the IR
//!         let sig = get_type(fn(int, int) -> int);
//!         let func = Function::new(cx, sig);
//!         let x = func.get_param(0);
//!         let y = func.get_param(1);
//!         let result = func.insn_mul(&x, &y);
//!         func.insn_return(&result);
//!         /// run the IR
//!         func.compile();
//!         let rfunc:fn(int, int) -> int = func.closure2();
//!         assert_eq(rfunc(4, 5), 20)
//!     });
//! }
//! ```

extern crate libc;
#[phase(plugin)]
extern crate bindgen;
use bindings::{
    jit_init,
    jit_uses_interpreter,
    jit_supports_threads,
    jit_supports_virtual_memory
};
pub use bindings::{
    jit_nint,
    jit_nuint
};
pub use compile::Compile;
pub use context::{
    Context,
    InContext,
    Functions
};
pub use function::{
    ABI,
    CDECL,
    Function
};
pub use label::Label;
pub use types::*;
pub use get_type = types::get;
pub use util::NativeRef;
pub use value::Value;

/// Initialise the library and prepare for operations
#[inline]
pub fn init() -> () {
    unsafe {
        jit_init()
    }
}
/// Check if the JIT is using a fallback interpreter
#[inline]
pub fn uses_interpreter() -> bool {
    unsafe {
        jit_uses_interpreter() != 0
    }
}
/// Check if the JIT supports theads
#[inline]
pub fn supports_threads() -> bool {
    unsafe {
        jit_supports_threads() != 0
    }
}
/// Check if the JIT supports virtual memory
#[inline]
pub fn supports_virtual_memory() -> bool {
    unsafe {
        jit_supports_virtual_memory() != 0
    }
}
macro_rules! native_ref(
    ($name:ident, $field:ident, $pointer_ty:ty) => (
        #[deriving(PartialEq)]
        pub struct $name {
            $field: $pointer_ty
        }
        impl NativeRef for $name {
            #[inline]
            /// Convert to a native pointer
            unsafe fn as_ptr(&self) -> $pointer_ty {
                self.$field
            }
            #[inline]
            /// Convert from a native pointer
            unsafe fn from_ptr(ptr:$pointer_ty) -> $name {
                $name {
                    $field: ptr
                }
            }
        }
    );
    ($name:ident, $field:ident, $pointer_ty:ty, $lifetime:ident) => (
        #[deriving(PartialEq)]
        pub struct $name<'a> {
            $field: $pointer_ty,
            marker: $lifetime<'a>
        }
        impl<'a> NativeRef for $name<'a> {
            #[inline]
            /// Convert to a native pointer
            unsafe fn as_ptr(&self) -> $pointer_ty {
                self.$field
            }
            #[inline]
            /// Convert from a native pointer
            unsafe fn from_ptr(ptr:$pointer_ty) -> $name {
                $name {
                    $field: ptr,
                    marker: $lifetime::<'a>
                }
            }
        }
    )
)
mod bindings;
mod context;
mod compile;
mod elf;
mod function;
mod label;
#[cfg(test)]
mod tests;
mod types;
mod util;
mod value;