#![crate_name = "jit_macros"]
#![comment = "Macros for LibJIT Bindings"]
#![crate_type = "dylib"]
#![allow(raw_pointer_deriving, dead_code, non_camel_case_types)]
#![deny(unnecessary_parens, unrecognized_lint, unreachable_code, unnecessary_allocation, unnecessary_typecast, unnecessary_allocation, uppercase_variables, unused_must_use)]
#![feature(globs, plugin_registrar, quote)]
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

use rustc::plugin::Registry;
use std::gc::{Gc, GC};
use syntax::ast::*;
use syntax::codemap::*;
use syntax::ext::base::*;
use syntax::ext::build::*;
use syntax::ext::quote::rt::*;
use syntax::parse::token::*;
use syntax::owned_slice::OwnedSlice;

fn gen_compile(cx:&mut ExtCtxt, pos:Span, _:Gc<MetaItem>, item:Gc<Item>, cb:|Gc<Item>|) {
    let fields:Vec<(Option<String>, P<Expr>)> = match item.node {
        ItemStruct(def, _) if def.is_virtual => {
            cx.span_err(pos, "Virtual structs cannot be JIT compiled");
            fail!("...")
        },
        ItemStruct(def, _) => {
            def.fields.iter().map(|field| {
                let name:Option<String> = match field.node.kind {
                    NamedField(id, _) => Some(id.to_source()),
                    UnnamedField(_) => None
                };
                let ty = field.node.ty;
                let ty_expr = quote_expr!(&*cx, &::jit::get_type::<$ty>());
                (name, ty_expr)
            }).collect()
        },
        _ => {
            cx.span_err(pos, "Compile can only be automatically implemented for structs");
            fail!("...")
        }
    };
    let jit_type_t = cx.ty_path(cx.path_global(pos, vec!(cx.ident_of("jit"), cx.ident_of("Type"))), None);
    let create_struct = cx.expr_path(cx.path_global(pos, vec!(cx.ident_of("jit"), cx.ident_of("Type"), cx.ident_of("create_struct"))));
    let jit_type_body = cx.block(pos, vec!(
        cx.stmt_let(pos, false, cx.ident_of("ty"), {
            let mut fields = cx.expr_vec(pos, fields.iter().map(|&(_, ex)| ex).collect());
            fields = cx.expr_method_call(pos, fields, cx.ident_of("as_mut_slice"), vec!());
            cx.expr_call(pos, create_struct, vec!(fields))
        }),
    ), Some(cx.expr_ident(pos, cx.ident_of("ty"))));
    let curr_item = cx.ty_ident(pos, item.ident);
    let jit_type = Method {
        ident: cx.ident_of("jit_type"),
        attrs: vec!(
            cx.attribute(pos, cx.meta_word(pos, InternedString::new("inline")))
        ),
        generics: Generics {
            lifetimes: vec!(),
            ty_params: OwnedSlice::empty()
        },
        explicit_self: Spanned {
            node: SelfStatic,
            span: pos
        },
        fn_style: NormalFn,
        decl: cx.fn_decl(
            vec!(
                cx.arg(pos, cx.ident_of("_"), cx.ty_option(curr_item))
            ),
            jit_type_t
        ),
        body: jit_type_body,
        span: pos,
        id: item.id,
        vis: Inherited
    };
    let lifetime = cx.lifetime(pos, cx.ident_of("'a").name);
    let compile_body = cx.block(pos, vec!(
        cx.stmt_let(pos, false, cx.ident_of("ty"), cx.expr_call(pos, quote_expr!(&*cx, ::jit::get_type::<$curr_item>), vec!())),
        cx.stmt_let(pos, false, cx.ident_of("val"), cx.expr_call(pos, quote_expr!(&*cx, ::jit::Value::new), vec!(
            cx.expr_ident(pos, cx.ident_of("func")),
            cx.expr_addr_of(pos, cx.expr_ident(pos, cx.ident_of("ty")))
        )))
    ), Some(cx.expr_ident(pos, cx.ident_of("val"))));
    let jit_func_t = cx.ty_path(cx.path_all(pos, true, vec!(cx.ident_of("jit"), cx.ident_of("Function")), vec!(lifetime.clone()), vec!()), None);
    let jit_val_t = cx.ty_path(cx.path_all(pos, true, vec!(cx.ident_of("jit"), cx.ident_of("Value")), vec!(lifetime.clone()), vec!()), None);
    let compile = Method {
        ident: cx.ident_of("compile"),
        attrs: vec!(
            cx.attribute(pos, cx.meta_word(pos, InternedString::new("inline")))
        ),
        generics: Generics {
            lifetimes: vec!(
                lifetime.clone()
            ),
            ty_params: OwnedSlice::empty()
        },
        explicit_self: Spanned {
            node: SelfRegion(None, MutImmutable),
            span: pos
        },
        fn_style: NormalFn,
        decl: cx.fn_decl(
            vec!(
                Arg::new_self(pos, MutImmutable),
                cx.arg(pos, cx.ident_of("func"), cx.ty_rptr(pos, jit_func_t, None, MutImmutable))
            ),
            jit_val_t
        ),
        body: compile_body,
        span: pos,
        id: item.id,
        vis: Inherited
    };
    let methods = vec!(
        box(GC) jit_type,
        box(GC) compile
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