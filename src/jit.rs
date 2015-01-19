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
#![allow(raw_pointer_derive, dead_code, non_camel_case_types, non_upper_case_globals, unused_attributes, unstable)]
#![deny(unused_parens, unknown_lints, unreachable_code, unused_allocation, unused_allocation, unused_must_use)]
#![feature(plugin, slicing_syntax, unboxed_closures, unsafe_destructor)]
#![stable]
//! This crate wraps LibJIT in an idiomatic style.
//! For example, here's a quick example which makes a multiply function using LibJIT:
//! 
//! ```rust
//! #![feature(plugin)]
//! extern crate jit;
//! #[no_link] #[plugin] #[macro_use]
//! extern crate jit_macros;
//! use jit::{Context, get, Type};
//! fn main() {
//!     // make a new context to make functions on
//!     let mut ctx = Context::new();
//!     jit_func!(ctx, func, mul(x: isize, y: isize) -> isize, {
//!         let result = x * y;
//!         func.insn_return(result);
//!     }, |mul| {
//!         assert_eq!(mul((4, 5)), 20);
//!     });
//! }
//! ```
#[no_link] #[plugin] #[macro_use]
extern crate rustc_bitflags;
extern crate libc;
extern crate "libjit-sys" as raw;
use raw::*;
pub use compile::Compile;
pub use context::{Builder, Context};
pub use elf::*;
pub use function::{flags, Abi, AnyFunction, UncompiledFunction, Function, CompiledFunction};
pub use function::flags::CallFlags;
pub use label::Label;
pub use types::kind::TypeKind;
pub use types::{kind, get, Type, Field, Fields, Params, StaticType};
pub use types::consts as typecs;
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
#[macro_use]
mod macros;
mod context;
mod compile;
mod elf;
mod function;
mod label;
mod types;
mod util;
mod value;