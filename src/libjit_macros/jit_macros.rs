#![crate_name = "jit_macros"]
#![comment = "Macros for LibJIT Bindings"]
#![crate_type = "dylib"]
#![allow(raw_pointer_deriving, dead_code, non_camel_case_types)]
#![deny(unnecessary_parens, unrecognized_lint, unreachable_code, unnecessary_allocation, unnecessary_typecast, unnecessary_allocation, uppercase_variables, unused_must_use)]
#![feature(plugin_registrar, quote)]
#![experimental]
//! This crate provides some macros for better LibJIT interop
//! For example, here's a quick example of automatic struct compilation to a LibJIT value:
//! 
//! ```rust
//! extern crate jit;
//! #[phase(syntax)]
//! extern crate jit_macros;
//! use jit::{C_DECL, Context, Function, Type, Types, get_type};
//! 
//! #[jit_compile]
//! struct Pos {
//!     x: f64,
//!     y: f64,
//!     z: f64
//! }
//! fn main() {
//!     let cx = Context::new();
//!     cx.build(|| {
//!         // build the IR
//!         let sig = get_type(fn() -> Pos);
//!         let func = Function::new(cx, sig);
//!         let result = Pos {
//!             x: -10,
//!             y: 0,
//!             z: 10
//!         }.compile();
//!         func.insn_return(&result);
//!         /// run the IR
//!         func.compile();
//!         let rfunc:fn() -> Pos = func.closure0();
//!         assert_eq(rfunc().x, -10)
//!     });
//! }
//! ```
extern crate rustc;
extern crate syntax;

use context::MacroContext;
use gen_compile::gen_compile_meth;
use gen_type::gen_type_meth;
use rustc::plugin::Registry;
use std::gc::GC;
use syntax::ast::{Generics, Item, ItemImpl, P, MetaItem};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, ItemDecorator};
use syntax::ext::build::AstBuilder;
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token::intern;
pub mod context;
pub mod gen_compile;
pub mod gen_type;
fn gen_compile(cx:&mut ExtCtxt, pos:Span, _:P<MetaItem>, item:P<Item>, cb:|P<Item>|) {
    let context = MacroContext::new(cx, pos, item);
    let ref cx = context.cx;
    let methods = vec!(
        box(GC) gen_type_meth(context),
        box(GC) gen_compile_meth(context)
    );
    let node = ItemImpl(
        Generics {
            lifetimes: vec!(),
            ty_params: OwnedSlice::empty()
        },
        Some(cx.trait_ref(cx.path_global(pos, vec!(cx.ident_of("jit"), cx.ident_of("Compile"))))),
        cx.ty_path(cx.path_ident(pos, item.ident), None),
        methods
    );
    cb(cx.item(pos, cx.ident_of("Compile"), vec!(), node));
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("jit_compile"), ItemDecorator(gen_compile));
}