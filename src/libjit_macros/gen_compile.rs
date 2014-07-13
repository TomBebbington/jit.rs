use context::MacroContext;
use syntax::ast::{
    Arg, Generics, Inherited, Method, MutImmutable, NormalFn, SelfRegion};
use syntax::codemap::Spanned;
use syntax::ext::build::AstBuilder;
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token::InternedString;
pub fn gen_compile_meth(context: MacroContext) -> Method {
    let MacroContext {cx: ref cx, pos: pos, item: item} = context;
    let lifetime = cx.lifetime(pos, cx.ident_of("'a").name);
    let jit_func_t = cx.ty_path(cx.path_all(pos, true, vec!(cx.ident_of("jit"), cx.ident_of("Function")), vec!(lifetime.clone()), vec!()), None);
    let jit_val_t = cx.ty_path(cx.path_all(pos, true, vec!(cx.ident_of("jit"), cx.ident_of("Value")), vec!(lifetime.clone()), vec!()), None);
    let curr_item = context.get_curr();
    let compile_body = cx.block(pos, vec!(
        cx.stmt_let(pos, false, cx.ident_of("ty"), cx.expr_call(pos, quote_expr!(&**cx, ::jit::get_type::<$curr_item>), vec!())),
        cx.stmt_let(pos, false, cx.ident_of("val"), cx.expr_call(pos, quote_expr!(&**cx, ::jit::Value::new), vec!(
            cx.expr_ident(pos, cx.ident_of("func")),
            cx.expr_addr_of(pos, cx.expr_ident(pos, cx.ident_of("ty")))
        )))
    ), Some(cx.expr_ident(pos, cx.ident_of("val"))));
    Method {
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
            node: SelfRegion(None, MutImmutable, cx.ident_of("self")),
            span: pos
        },
        fn_style: NormalFn,
        decl: cx.fn_decl(
            vec!(
                Arg::new_self(pos, MutImmutable, cx.ident_of("func")),
                cx.arg(pos, cx.ident_of("func"), cx.ty_rptr(pos, jit_func_t, None, MutImmutable))
            ),
            jit_val_t
        ),
        body: compile_body,
        span: pos,
        id: item.id,
        vis: Inherited
    }
}