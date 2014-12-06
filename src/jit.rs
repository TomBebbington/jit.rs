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
#![feature(globs, macro_rules, slicing_syntax, unsafe_destructor, phase)]
#![stable]
//! This crate wraps LibJIT in an idiomatic style.
//! For example, here's a quick example which makes a multiply function using LibJIT:
//! 
//! ```rust
//! extern crate jit;
//! use jit::{Context, UncompiledFunction, get_type};
//! fn main() {
//!     // make a new context to make functions on
//!     let ref ctx = Context::new();
//!     // get the type of the function
//!     let sig = get_type::<fn(int, int) -> int>();
//!     // make the function
//!     let func = UncompiledFunction::new(ctx, sig);
//!     ctx.build(|| {
//!         let ref x = func[0];
//!         let ref y = func[1];
//!         let ref result = func.insn_mul(x, y);
//!         func.insn_return(result);
//!     });
//!     // compile the IR and get the machine code as a function
//!     func.compile().with_closure2(|mul:extern fn(int, int) -> int| {
//!         assert_eq!(mul(4, 5), 20);
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
pub use context::Context;
pub use function::{
    ABI,
    CompiledFunction,
    UncompiledFunction,
    Function,
    CallFlags
};
pub use label::Label;
pub use types::*;
pub use types::get as get_type;
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