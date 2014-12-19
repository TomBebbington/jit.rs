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
#![crate_name = "jit"]
#![allow(raw_pointer_deriving, dead_code, non_camel_case_types, non_upper_case_globals, unknown_features)]
#![deny(unused_parens, unknown_lints, unreachable_code, unused_allocation, unused_allocation, unused_must_use)]
#![feature(globs, macro_rules, slicing_syntax, unboxed_closures, unsafe_destructor, phase)]
#![stable]
//! This crate wraps LibJIT in an idiomatic style.
//! For example, here's a quick example which makes a multiply function using LibJIT:
//! 
//! ```rust
//! #![feature(phase)]
//! #[phase(link, plugin)]
//! extern crate jit;
//! use jit::{Context, get, Type};
//! fn main() {
//!     // make a new context to make functions on
//!     let mut ctx = Context::new();
//!     jit_func!(ctx, func, fn mul(x: int, y: int) -> int {
//!         let result = func.insn_mul(x, y);
//!         func.insn_return(&result);
//!     }, |mul| {
//!         assert_eq!(mul((4, 5)), 20);
//!     });
//! }
//! ```
extern crate libc;
#[cfg(test)]
extern crate test;
extern crate "libjit-sys" as raw;
use raw::{
    jit_init,
    jit_uses_interpreter,
    jit_supports_threads,
    jit_supports_virtual_memory
};
pub use raw::{
    jit_nint,
    jit_nuint
};
pub use compile::Compile;
pub use context::{Builder, Context};
pub use elf::*;
pub use function::*;
pub use function::flags::CallFlags;
pub use label::Label;
pub use types::kind::TypeKind;
pub use types::*;
pub use util::NativeRef;
pub use value::Value;
use libc::{c_int, c_void};

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

extern fn handle_exception(kind: c_int) -> *mut c_void {
    if kind == 1 {
        return ::std::ptr::null_mut();
    }
    panic!("{}", match kind {
        0 => "The operation resulted in an overflow exception",
        -1 => "The operation resulted in an arithmetic exception",
        -2 => "The operation resulted in a division by zero exception",
        -3 => "An error occurred when attempting to dynamically compile a function",
        -4 => "The system ran out of memory while performing an operation",
        -5 => "An attempt was made to dereference a NULL pointer",
        -6 => "An attempt was made to call a function with a NULL function pointer",
        -7 => "An attempt was made to call a nested function from a non-nested context",
        _ => "Unknown exception"
    });
}
#[macro_escape]
mod macros;
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