#![feature(rustc_private, plugin_registrar, quote, plugin)]
#![plugin(matches)]
extern crate syntax;
extern crate rustc;
#[no_link]
#[macro_use]
extern crate matches;

use syntax::codemap::*;
use syntax::parse::*;
use syntax::parse::token::{Token, IdentStyle};
use syntax::abi::Abi;
use syntax::ast_util::empty_generics;
use syntax::ast::*;
use syntax::ext::base::*;
use syntax::ext::build::*;
use syntax::ext::quote::rt::ToTokens;
use syntax::ext::source_util::*;
use syntax::ptr::P;
use syntax::owned_slice::OwnedSlice;
use rustc::plugin::Registry;

static BAD_EXPR:&'static str = "bad jit expression";
static BAD_STRUCT:&'static str = "jit-compatible structs must be packed, mark with #[repr(packed)] to fix";
static BAD_ITEM:&'static str = "only structs can be compatible with LibJIT";

fn simple_type(cx: &mut ExtCtxt, name: &'static str, as_cow:bool) -> Option<P<Expr>> {
    let new_name = format!("get_{}", name);
    let name = cx.ident_of(&new_name);
    let mut expr = quote_expr!(cx, jit::typecs::$name());
    if as_cow {
        expr = quote_expr!(cx, $expr.into())
    }
    Some(expr)
}
fn type_expr(cx: &mut ExtCtxt, sp: Span, ty: P<Ty>, as_cow: bool) -> Option<P<Expr>> {
    match ty.node {
        Ty_::TyParen(ref ty) => type_expr(cx, sp, ty.clone(), as_cow),
        Ty_::TyPtr(_) | Ty_::TyRptr(_, _) => simple_type(cx, "VOID_PTR", as_cow),
        Ty_::TyPath(ref self_, ref path) => {
            if self_.is_none() && path.segments.len() == 1 {
                match path.segments[0].identifier.as_str() {
                    "i8" => return simple_type(cx, "sbyte", as_cow),
                    "u8" => return simple_type(cx, "ubyte", as_cow),
                    "i16" => return simple_type(cx, "short", as_cow),
                    "u16" => return simple_type(cx, "ushort", as_cow),
                    "i32" => return simple_type(cx, "int", as_cow),
                    "u32" => return simple_type(cx, "uint", as_cow),
                    "i64" => return simple_type(cx, "long", as_cow),
                    "u64" => return simple_type(cx, "ulong", as_cow),
                    "isize" => return simple_type(cx, "nint", as_cow),
                    "usize" => return simple_type(cx, "nuint", as_cow),
                    "f32" => return simple_type(cx, "float32", as_cow),
                    "f64" => return simple_type(cx, "float64", as_cow),
                    "bool" => return simple_type(cx, "sys_bool", as_cow),
                    "char" => return simple_type(cx, "sys_char", as_cow),
                    _ => {/* fall through */}
                }
            }

            let get_type =
                cx.path(sp,
                        vec![cx.ident_of("jit"),
                             cx.ident_of("Compile"),
                             cx.ident_of("get_type")]);
            let qpath = cx.expr(sp, Expr_::ExprPath(
                Some(QSelf {
                    ty: ty.clone(),
                    // trait's path includes everything except
                    // the function at the end.
                    position: get_type.segments.len() - 1
                }),
                get_type));
            Some(if as_cow {
                qpath
            } else {
                quote_expr!(cx, &$qpath())
            })
        },
        _ => {
            let error = format!("could not resolve type {:?}", ty.to_tokens(cx));
            cx.span_err(sp, &*error);
            None
        }
    }
}

fn expand_jit(cx: &mut ExtCtxt, sp: Span, _meta: &MetaItem, item: &Item, mut push: &mut FnMut(P<Item>)) {
    let name = item.ident;
    let jit = cx.ident_of("jit");
    let jit_life = cx.lifetime(sp, token::intern("'a"));
    let jit_compile = cx.path_all(sp, false, vec![jit, cx.ident_of("Compile")], vec![jit_life], vec![], vec![]);
    let jit_cow_type = cx.path_all(sp, false, vec![jit, cx.ident_of("CowType")], vec![cx.lifetime(sp, token::intern("'static"))], vec![], vec![]);
    let jit_func = cx.path_all(sp, false, vec![jit, cx.ident_of("UncompiledFunction")], vec![jit_life], vec![], vec![]);
    let jit_val = cx.path(sp, vec![jit, cx.ident_of("Val")]);
    let jit_val_new = cx.path(sp, vec![jit, cx.ident_of("Val"), cx.ident_of("new")]);
    let jit_value = cx.ty_rptr(sp, cx.ty_path(jit_val), Some(jit_life), Mutability::MutImmutable);
    let new_struct = cx.path(sp, vec![jit, cx.ident_of("Type"), cx.ident_of("new_struct")]);
    let func = cx.ident_of("func");
    let value = cx.ident_of("value");
    let offset = cx.ident_of("offset");
    let mut repr = None;
    for attr in item.attrs.iter() {
        if let MetaItem_::MetaList(ref name, ref items) = attr.node.value.node {
            if &**name == "repr" && items.len() == 1 {
                if let MetaItem_::MetaWord(ref text) = items[0].node {
                    repr = Some(&**text)
                }
            }
        }
    }
    match item.node {
        Item_::ItemEnum(ref def, _) => {
            if let Some(kind) = repr {
                let inner_ty = cx.ty_ident(sp, cx.ident_of(kind));
                let type_expr = type_expr(cx, sp, inner_ty.clone(), true).unwrap();
                let expr = quote_expr!(cx, (self as $inner_ty).compile(&func));
                let item = cx.item(sp, name, vec![], Item_::ItemImpl(
                    Unsafety::Normal,
                    ImplPolarity::Positive,
                    Generics {
                        lifetimes: vec![ LifetimeDef {lifetime: jit_life, bounds: vec![]}],
                        ty_params: OwnedSlice::empty(),
                        where_clause: WhereClause {
                            id: DUMMY_NODE_ID,
                            predicates: vec![]
                        }
                    },
                    Some(cx.trait_ref(jit_compile)),
                    cx.ty_ident(sp, name),
                    vec![
                        P(ImplItem {
                            attrs: vec![],
                            id: DUMMY_NODE_ID,
                            span: sp,
                            ident: cx.ident_of("get_type"),
                            vis: Visibility::Inherited,
                            node: ImplItem_::MethodImplItem(
                                MethodSig {
                                    unsafety: Unsafety::Normal,
                                    abi: Abi::Rust,
                                    explicit_self: respan(sp, ExplicitSelf_::SelfStatic),
                                    decl: cx.fn_decl(vec![], cx.ty_path(jit_cow_type)),
                                    generics: empty_generics(),
                                },
                                cx.block_expr(type_expr))
                        }),
                        P(ImplItem {
                            attrs: vec![],
                            id: DUMMY_NODE_ID,
                            span: sp,
                            ident: cx.ident_of("compile"),
                            vis: Visibility::Inherited,
                            node: ImplItem_::MethodImplItem(
                                MethodSig {
                                    unsafety: Unsafety::Normal,
                                    abi: Abi::Rust,
                                    explicit_self: respan(
                                        sp,
                                        ExplicitSelf_::SelfValue(cx.ident_of("b"))),
                                    decl: cx.fn_decl(
                                        vec![
                                            Arg::new_self(sp, Mutability::MutImmutable,
                                                          cx.ident_of("self")),
                                            cx.arg(sp, func, cx.ty_rptr(sp, cx.ty_path(jit_func),
                                                                        None, Mutability::MutImmutable))],
                                        jit_value),
                                    generics: empty_generics(),
                                },
                                cx.block_expr(expr)
                            )
                        })
                    ]
                ));
                push(item);
            } else {
                cx.span_err(sp, BAD_ITEM)
            }
        },
        Item_::ItemStruct(ref def, ref gen) => {
            if !matches!(repr, Some("packed") | Some("c") | Some("C")) {
                cx.span_err(sp, BAD_STRUCT);
                return;
            }
            let mut fields = Vec::with_capacity(def.fields.len());
            let mut names = Some(Vec::with_capacity(fields.len()));
            let mut compiler = Vec::with_capacity(def.fields.len() + 1);
            let types = gen.ty_params.iter().map(|param| cx.ty_ident(sp, param.ident)).collect();
            let self_ty = cx.ty_path(cx.path_all(sp, false, vec![name], vec![], types, vec![]));
            let self_type = if let Some(expr) = type_expr(cx, sp, self_ty.clone(), false) {
                expr
            } else {
                return
            };
            compiler.push(cx.stmt_let(sp, false, value, cx.expr_call(
                sp,
                cx.expr_path(jit_val_new),
                vec![
                    cx.expr_ident(sp, func),
                    self_type
                ]
            )));
            let lit_usize = LitIntType::UnsignedIntLit(UintTy::TyUs);
            if def.fields.len() > 1 {
                compiler.push(cx.stmt_let(sp, true, offset, cx.expr_lit(sp, Lit_::LitInt(0, lit_usize))));
            }
            for (index, field) in def.fields.iter().enumerate() {
                if let Some(expr) = type_expr(cx, sp, field.node.ty.clone(), false) {
                    fields.push(expr);
                    let has_name = field.node.ident().is_some();
                    if has_name && names.is_some() {
                        let ident = field.node.ident().unwrap();
                        let expr = expand_stringify(cx, sp, &[TokenTree::TtToken(sp, Token::Ident(ident, IdentStyle::Plain))]);
                        names.as_mut().unwrap().push(expr.make_expr().unwrap());
                    } else {
                        names = None
                    }
                    let current_offset = if index == 0 {
                        cx.expr_lit(sp, Lit_::LitInt(0, lit_usize))
                    } else {
                        cx.expr_ident(sp, offset)
                    };
                    let name = field.node.ident().unwrap();
                    compiler.push(quote_stmt!(cx, func.insn_store_relative(value, $current_offset, self.$name.compile(func))).unwrap());
                    let size_of = cx.expr_path(cx.path_all(sp, false, vec![cx.ident_of("std"), cx.ident_of("mem"), cx.ident_of("size_of")], vec![], vec![field.node.ty.clone()], vec![]));
                    if def.fields.len() > 1 && index < def.fields.len() - 1 {
                        compiler.push(quote_stmt!(cx, offset += $size_of()).unwrap());
                    }
                } else {
                    return;
                }
            }
            let fields = cx.expr_mut_addr_of(sp, cx.expr_vec(sp, fields));
            let mut type_expr = cx.expr_call(sp, cx.expr_path(new_struct), vec![fields]);
            if let Some(names) = names {
                let names = cx.expr_vec(sp, names);
                type_expr = quote_expr!(cx, {
                    let mut ty: Type = $type_expr;
                    ty.set_names(&$names);
                    ty.into()
                })
            }
            let item = cx.item(sp, name, vec![], Item_::ItemImpl(
                Unsafety::Normal,
                ImplPolarity::Positive,
                Generics {
                    lifetimes: vec![ LifetimeDef {lifetime: jit_life, bounds: vec![]}],
                    ty_params: gen.ty_params.clone(),
                    where_clause: WhereClause {
                        id: DUMMY_NODE_ID,
                        predicates: gen.ty_params.iter()
                            .map(|param| WherePredicate::BoundPredicate(
                                WhereBoundPredicate {
                                    span: sp,
                                    bound_lifetimes: vec![],
                                    bounded_ty: cx.ty_ident(sp, param.ident),
                                    bounds: OwnedSlice::from_vec(vec![
                                        TyParamBound::TraitTyParamBound(
                                            cx.poly_trait_ref(sp, jit_compile.clone()),
                                            TraitBoundModifier::None
                                        ),
                                    ])
                                }
                            ))
                            .collect()
                    }
                },
                Some(cx.trait_ref(jit_compile)),
                self_ty,
                vec![
                    P(ImplItem {
                        attrs: vec![],
                        id: DUMMY_NODE_ID,
                        span: sp,
                        ident: cx.ident_of("get_type"),
                        vis: Visibility::Inherited,
                        node: ImplItem_::MethodImplItem(
                            MethodSig {
                                unsafety: Unsafety::Normal,
                                abi: Abi::Rust,
                                explicit_self: respan(sp, ExplicitSelf_::SelfStatic),
                                decl: cx.fn_decl(vec![], cx.ty_path(jit_cow_type)),
                                generics: empty_generics(),
                            },
                            cx.block_expr(type_expr)
                        )
                    }),
                    P(ImplItem {
                        attrs: vec![],
                        id: DUMMY_NODE_ID,
                        span: sp,
                        ident: cx.ident_of("compile"),
                        vis: Visibility::Inherited,
                        node: ImplItem_::MethodImplItem(
                            MethodSig {
                                unsafety: Unsafety::Normal,
                                abi: Abi::Rust,
                                explicit_self: respan(
                                    sp,
                                    ExplicitSelf_::SelfValue(cx.ident_of("b"))),
                                decl: cx.fn_decl(
                                    vec![
                                        Arg::new_self(sp, Mutability::MutImmutable,
                                                      cx.ident_of("self")),
                                        cx.arg(sp, func, cx.ty_rptr(sp, cx.ty_path(jit_func),
                                                                    None, Mutability::MutImmutable))],
                                    jit_value),
                                generics: empty_generics(),
                            },
                            cx.block(sp, compiler, Some(cx.expr_ident(sp, value))))
                    })
                ]
            ));
            push(item);
        },
        _ => {
            cx.span_err(sp, BAD_ITEM);
            return;
        }
    }
}
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern("jit"), SyntaxExtension::Decorator(Box::new(expand_jit)));
}

#[macro_export]
/// Construct a JIT struct with the fields given
macro_rules! jit_struct(
    ($($name:ident: $ty:ty),*) => ({
        let mut ty = Type::new_struct(&mut [
            $(&get::<$ty>()),*
        ]);
        ty.set_names(&[$(stringify!($name)),*]);
        ty
    });
    ($($ty:ty),+ ) => (
        Type::new_struct(&mut [
            $(&get::<$ty>()),+
        ])
    )
);

#[macro_export]
/// Construct a JIT union with the fields given
macro_rules! jit_union(
    ($($name:ident: $ty:ty),*) => ({
        let union = Type::new_union(&mut [
            $(&get::<$ty>()),*
        ]);
        union.set_names(&[$(stringify!($name)),*]);
        union
    });
    ($($ty:ty),+ ) => (
        Type::new_union(&mut [
            $(&get::<$ty>()),*
        ])
    )
);
#[macro_export]
/// Construct a JIT function signature with the arguments and return type given
macro_rules! jit_fn(
    ($($arg:ty),* -> $ret:ty) => ({
        use std::default::Default;
        Type::new_signature(Default::default(), &get::<$ret>(), &mut [
            $(&get::<$arg>()),*
        ])
    });
    (raw $($arg:expr),* -> $ret:expr) => ({
        use std::default::Default;
        Type::new_signature(Default::default(), &$ret, &mut [
            $(&$arg),*
        ])
    });
);

#[macro_export]
macro_rules! jit(
    ($func:ident, return) => (
        $func.insn_default_return()
    );
    ($func:ident, return $($t:tt)+) => (
        $func.insn_return(jit!($func, $($t)+))
    );
    ($func:ident, $var:ident += $($t:tt)+) => (
        $func.insn_store($var, &$func.insn_add($var, jit!($func, $($t)+)));
    );
    ($func:ident, $var:ident -= $($t:tt)+) => (
        $func.insn_store($var, &$func.insn_sub($var, jit!($func, $($t)+)));
    );
    ($func:ident, $var:ident *= $($t:tt)+) => (
        $func.insn_store($var, &$func.insn_mul($var, jit!($func, $($t)+)));
    );
    ($func:ident, $var:ident /= $($t:tt)+) => (
        $func.insn_store($var, &$func.insn_div($var, jit!($func, $($t)+)));
    );
    ($func:ident, $($a:tt)+ + $($b:tt)+) => (
        $func.insn_add(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, $($a:tt)+ - $($b:tt)+) => (
        $func.insn_sub(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, $($a:tt)+ * $($b:tt)+) => (
        $func.insn_mul(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, $($a:tt)+ / $($b:tt)+) => (
        $func.insn_div(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, $($a:tt)+ % $($b:tt)+) => (
        $func.insn_rem(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, ($($t:tt)+).sqrt()) => (
        $func.insn_sqrt(&jit!($func, $($t)+))
    );
    ($func:ident, $var:ident = $($t:tt)+) => (
        $func.insn_store($var, jit!($func, $val));
    );
    ($func:ident, *$var:ident) => (
        $func.insn_load($var)
    );
    ($func:ident, call($call:expr,
        $($arg:expr),+
    )) => (
        $func.insn_call(None::<String>, $call, None, [$($arg),+].as_mut_slice())
    );
    ($func:ident, jump_table($value:expr,
        $($label:ident),+
    )) => (
    let ($($label),+) = {
        $(let $label:Label = Label::new($func);)+
        $func.insn_jump_table($value, [
            $($label),+
        ].as_mut_slice());
        ($($label),+)
    });
);
#[macro_export]
macro_rules! jit_func(
    ($ctx:expr, $func:ident, $name:ident() -> $ret:ty, $value:expr) => ({
        use std::default::Default;
        let sig = Type::new_signature(Default::default(), &get::<$ret>(), &mut []);
        $ctx.build_func(&sig, |$func| $value)
    });
    ($ctx:expr, $func:ident, $name:ident($($arg:ident:$ty:ty),+) -> $ret:ty, $value:expr) => ({
        use std::default::Default;
        let sig = Type::new_signature(Default::default(), &get::<$ret>(), &mut [$(&get::<$ty>()),*]);
        $ctx.build_func(&sig, |$func| {
            let mut i = 0;
            $(let $arg = {
                i += 1;
                &$func[i - 1]
            };)*
            $value
        })
    });
    ($ctx:expr, $func:ident, $name:ident() -> $ret:ty, $value:expr, |$comp_func:ident| $comp:expr) => ({
        use std::default::Default;
        let sig = Type::new_signature(Default::default(), &get::<$ret>(), &mut []);
        $ctx.build_func(&sig, |$func| $value)
            .with::<(), $ret, _>(|$comp_func| $comp)
    });
    ($ctx:expr, $func:ident, $name:ident($($arg:ident:$arg_ty:ty),+) -> $ret:ty, $value:expr, |$comp_func:ident| $comp:expr) => ({
        use std::default::Default;
        let sig = Type::new_signature(Default::default(), &get::<$ret>(), &mut [$(&get::<$arg_ty>()),*]);
        $ctx.build_func(&sig, |$func| {
            let mut i = 0;
            $(let $arg = {
                i += 1;
                &$func[i - 1]
            };)*
            $value
        }).with::<($($arg_ty),*), $ret, _>(|$comp_func|
            $comp)
    });
);
