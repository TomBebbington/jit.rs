use context::MacroContext;
use syntax::ast::{
    Generics, Expr, Inherited, ItemStruct, Method,
    NamedField, NormalFn, P, SelfStatic, UnnamedField};
use syntax::codemap::Spanned;
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToSource;
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token::InternedString;
pub fn gen_type_meth(context: MacroContext) -> Method {
    let MacroContext {cx: ref cx, pos: pos, item: item} = context;
    let jit_type_t = cx.ty_path(cx.path_global(pos, vec!(cx.ident_of("jit"), cx.ident_of("Type"))), None);
    let create_struct = cx.expr_path(cx.path_global(pos, vec!(cx.ident_of("jit"), cx.ident_of("Type"), cx.ident_of("create_struct"))));
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
                let ty_expr = quote_expr!(&**cx, &::jit::get_type::<$ty>());
                (name, ty_expr)
            }).collect()
        },
        _ => {
            cx.span_err(pos, "Compile can only be automatically implemented for structs");
            fail!("...")
        }
    };
    let jit_type_body = cx.block(pos, vec!(
        cx.stmt_let(pos, false, cx.ident_of("ty"), {
            let mut fields = cx.expr_vec(pos, fields.iter().map(|&(_, ex)| ex).collect());
            fields = cx.expr_method_call(pos, fields, cx.ident_of("as_mut_slice"), vec!());
            cx.expr_call(pos, create_struct, vec!(fields))
        }),
    ), Some(cx.expr_ident(pos, cx.ident_of("ty"))));
    Method {
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
                cx.arg(pos, cx.ident_of("_"), cx.ty_option(context.get_curr()))
            ),
            jit_type_t
        ),
        body: jit_type_body,
        span: pos,
        id: item.id,
        vis: Inherited
    }
}