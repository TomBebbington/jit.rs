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
#![allow(raw_pointer_derive, non_camel_case_types, non_upper_case_globals)]
#![deny(unused_attributes, dead_code, unused_parens, unknown_lints, unreachable_code, unused_allocation, unused_allocation, unused_must_use)]
#![feature(alloc, plugin, unboxed_closures, optin_builtin_traits, associated_consts, raw, oom)]
#![plugin(rustc_bitflags)]

//! This crate wraps LibJIT in an idiomatic style.
//! For example, here's a quick example which makes a multiply function using LibJIT:
//!
//! ```rust
//! extern crate jit;
//! #[no_link] #[macro_use]
//! extern crate jit_macros;
//! use jit::*;
//! fn main() {
//!     // make a new context to make functions on
//!     let mut ctx = Context::<()>::new();
//!     jit_func!(&mut ctx, func, fn(x: isize, y: isize) -> isize {
//!         func.insn_return(x * y);
//!     }, {
//!         assert_eq!(func(4, 5), 20);
//!         assert_eq!(func(-2, -4), 8);
//!     });
//! }
//! ```
#[no_link] #[macro_use]
extern crate rustc_bitflags;
extern crate alloc;
extern crate libc;
extern crate libjit_sys as raw;
use raw::*;
use libc::c_void;
use std::mem;
pub use compile::Compile;
pub use context::Context;
pub use elf::*;
pub use function::{flags, Abi, UncompiledFunction, Func, CompiledFunction};
pub use function::flags::CallFlags;
pub use label::Label;
pub use insn::{Block, Instruction, InstructionIter};
pub use types::kind::TypeKind;
pub use types::{kind, get, Type, Field, Fields, Params, CowType, StaticType, Ty, TaggedType};
pub use types::consts as typecs;
pub use value::Val;


extern fn free_data<T>(data: *mut c_void) {
    unsafe {
        let actual_data:Box<T> = mem::transmute(data);
        mem::drop(actual_data);
    }
}

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
mod insn;
mod label;
mod types;
mod util;
mod value;
