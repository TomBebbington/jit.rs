#![feature(core, rustc_private, plugin_registrar, quote, plugin)]

extern crate syntax;
extern crate rustc;
#[no_link]
#[macro_use]
#[plugin]
extern crate matches;

use syntax::codemap::*;
use syntax::parse::*;
use syntax::parse::token::{Token, IdentStyle};
use syntax::abi::Abi;
use syntax::ast_util::empty_generics;
use syntax::ast::*;
use syntax::ext::base::*;
use syntax::ext::build::*;
use syntax::ext::quote::rt::ToSource;
use syntax::ext::source_util::*;
use syntax::ptr::P;
use syntax::owned_slice::OwnedSlice;
use rustc::plugin::Registry;

static BAD_RETURN:&'static str = "bad return type";
static BAD_ABI:&'static str = "jit-compilable functions must have Rust or C ABI";
static BAD_EXPR:&'static str = "bad jit expression";
static BAD_STRUCT:&'static str = "jit-compatible structs must be packed, mark with #[repr(packed)] to fix";
static BAD_ITEM:&'static str = "only structs can be compatible with LibJIT";

fn simple_type(cx: &mut ExtCtxt, name: &'static str) -> Option<P<Expr>> {
    let name = cx.ident_of(name);
    Some(quote_expr!(cx, *jit::typecs::$name))
}
fn type_expr(cx: &mut ExtCtxt, sp: Span, ty: P<Ty>) -> Option<P<Expr>> {
    match ty.node {
        Ty_::TyParen(ref ty) => type_expr(cx, sp, ty.clone()),
        Ty_::TyPtr(_) | Ty_::TyRptr(_, _) => simple_type(cx, "VOID_PTR"),
        Ty_::TyPath(ref path, _) => {
            let path_parts = path.segments.iter().map(|s| s.identifier.as_str()).collect::<Vec<_>>();
            match &*path_parts {
                ["i8"] => simple_type(cx, "SBYTE"),
                ["u8"] => simple_type(cx, "UBYTE"),
                ["i16"] => simple_type(cx, "SHORT"),
                ["u16"] => simple_type(cx, "USHORT"),
                ["i32"] => simple_type(cx, "INT"),
                ["u32"] => simple_type(cx, "UINT"),
                ["i64"] => simple_type(cx, "LONG"),
                ["u64"] => simple_type(cx, "ULONG"),
                ["isize"] => simple_type(cx, "NINT"),
                ["usize"] => simple_type(cx, "NUINT"),
                ["f32"] => simple_type(cx, "FLOAT32"),
                ["f64"] => simple_type(cx, "FLOAT64"),
                ["bool"] => simple_type(cx, "SYS_BOOL"),
                ["char"] => simple_type(cx, "SYS_CHAR"),
                _ => {
                    let jit = cx.ident_of("jit");
                    let jit_compile = cx.path(sp, vec![jit, cx.ident_of("Compile")]);
                    let qpath = cx.expr(sp, Expr_::ExprQPath(P(QPath {
                        self_type: ty.clone(),
                        trait_ref: P(cx.trait_ref(jit_compile)),
                        item_path: PathSegment {
                            identifier: cx.ident_of("get_type"),
                            parameters: PathParameters::AngleBracketedParameters(AngleBracketedParameterData {
                                lifetimes: vec![],
                                types: OwnedSlice::empty(),
                                bindings: OwnedSlice::empty()
                            })
                        }
                    })));
                    Some(cx.expr_call(sp, qpath, vec![]))
                }
            }
        },
        _ => {
            let error = format!("could not resolve type {}", ty.to_source());
            cx.span_err(sp, &*error);
            None
        }
    }
}

fn expand_jit(cx: &mut ExtCtxt, sp: Span, meta: &MetaItem, item: &Item, mut push: Box<FnMut(P<Item>)>) {
    let name = item.ident;
    let jit = cx.ident_of("jit");
    let jit_compile = cx.path(sp, vec![jit, cx.ident_of("Compile")]);
    let jit_type = cx.path(sp, vec![jit, cx.ident_of("Type")]);
   //let jit_static_type = cx.path(sp, vec![jit, cx.ident_of("StaticType")]);
    let jit_life = cx.lifetime(sp, token::intern("a"));
    let jit_func = cx.path_all(sp, false, vec![jit, cx.ident_of("UncompiledFunction")], vec![jit_life], vec![], vec![]);
    let jit_value = cx.path_all(sp, false, vec![jit, cx.ident_of("Value")], vec![jit_life], vec![], vec![]);
    let jit_value_new = cx.path(sp, vec![jit, cx.ident_of("Value"), cx.ident_of("new")]);
    let new_struct = cx.path(sp, vec![jit, cx.ident_of("Type"), cx.ident_of("new_struct")]);
    let func = cx.ident_of("func");
    let value = cx.ident_of("value");
    let offset = cx.ident_of("offset");
    let mut is_packed = false;
    //push(cx.item_use_simple(sp, Visibility::Inherited, jit_static_type));
    for attr in item.attrs.iter() {
        if let MetaItem_::MetaList(ref name, ref items) = attr.node.value.node {
            if name.get() == "repr" && items.len() == 1 {
                if let MetaItem_::MetaWord(ref text) = items[0].node {
                    if text.get() == "packed" {
                        is_packed = true;
                    }
                } 
            }
        } 
    }
    match item.node {
        Item_::ItemFn(ref dec, Unsafety::Normal, abi, _, ref block) => {
            if !matches!(abi, Abi::Rust | Abi::C | Abi::Cdecl) {
                cx.span_err(sp, BAD_ABI);
                return;
            }
            let ret = match dec.output {
                FunctionRetTy::NoReturn(_) => {
                    quote_expr!(cx, jit::typecs::VOID_PTR.get())
                },
                FunctionRetTy::Return(ref ty) => {
                    if let Some(ex) = type_expr(cx, sp, ty.clone()) {
                        ex
                    } else {
                        return;
                    }
                },
                FunctionRetTy::DefaultReturn(sp) => {
                    cx.span_err(sp, BAD_RETURN);
                    return;
                }
            };
            let mut should_ret = false;
            let args = dec.inputs.iter().map(|arg| {
                if let Some(ex) = type_expr(cx, sp, arg.ty.clone()) {
                    ex
                } else {
                    should_ret = true;
                    cx.expr_int(sp, 32)
                }
            }).collect::<Vec<_>>();
            if should_ret {
                return;
            }
            let args = cx.expr_vec(sp, args);
            let sig = quote_expr!(cx, jit::Type::new_signature(jit::Abi::CDecl, $ret, &mut $args));
            let mut block = vec![];
            for (index, arg) in dec.inputs.iter().enumerate() {
                let index = cx.expr_int(sp, index as isize);
                if let Pat_::PatIdent(BindingMode::BindByValue(_), ident, _) = arg.pat.node {
                    let name = ident.node;
                    block.push(quote_stmt!(cx, let $name = func[$index]))
                }
            }
            let block = cx.block(sp, block, None);
            let block = cx.block_expr(quote_expr!(cx, ctx.build_func($sig, |func| $block)));
            let args = vec![
                cx.arg(sp, cx.ident_of("ctx"), quote_ty!(cx, &jit::Context))
            ];
            let item = cx.item_fn(sp, cx.ident_of(&*format!("jit_{:?}", name)), args, quote_ty!(cx, jit::CompiledFunction), block);
            println!("{}", item.to_source());
            push(item);
        },
        Item_::ItemStruct(ref def, _) => {
            if !is_packed {
                cx.span_err(sp, BAD_STRUCT);
                return;
            }
            let mut fields = Vec::with_capacity(def.fields.len());
            let mut names = Some(Vec::with_capacity(fields.len()));
            let mut compiler = Vec::with_capacity(def.fields.len() + 1);
            let self_ty = cx.ty_ident(sp, name);
            let self_type = if let Some(expr) = type_expr(cx, sp, self_ty) {
                expr
            } else {
                return
            };
            compiler.push(cx.stmt_let(sp, false, value, cx.expr_call(
                sp,
                cx.expr_path(jit_value_new),
                vec![
                    cx.expr_ident(sp, func),
                    self_type
                ]
            )));
            let lit_usize = LitIntType::UnsignedIntLit(UintTy::TyUs(false));
            if def.fields.len() > 1 {
                compiler.push(cx.stmt_let(sp, true, offset, cx.expr_lit(sp, Lit_::LitInt(0, lit_usize))));
            }
            for (index, field) in def.fields.iter().enumerate() {
                if let Some(expr) = type_expr(cx, sp, field.node.ty.clone()) {
                    fields.push(cx.expr_method_call(sp, expr, cx.ident_of("get"), vec![]));
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
                    compiler.push(quote_stmt!(cx, func.insn_store_relative(value, $current_offset, self.$name.compile(func))));
                    let size_of = cx.expr_path(cx.path_all(sp, false, vec![cx.ident_of("mem"), cx.ident_of("size_of")], vec![], vec![field.node.ty.clone()], vec![]));
                    if def.fields.len() > 1 && index < def.fields.len() - 1 {
                        compiler.push(quote_stmt!(cx, offset += $size_of()));
                    }
                } else {
                    return;
                }
            }
            let fields = cx.expr_mut_addr_of(sp, cx.expr_vec(sp, fields));
            let mut type_expr = cx.expr_call(sp, cx.expr_path(new_struct), vec![fields]);
            if let Some(names) = names {
                let names = cx.expr_addr_of(sp, cx.expr_vec(sp, names));
                type_expr = cx.expr_method_call(sp, type_expr, cx.ident_of("with_names"), vec![names]);
            }
            push(cx.item(sp, name, vec![], Item_::ItemImpl(
                Unsafety::Normal,
                ImplPolarity::Positive,
                empty_generics(),
                Some(cx.trait_ref(jit_compile)),
                cx.ty_ident(sp, name),
                vec![
                    ImplItem::MethodImplItem(P(Method {
                        attrs: vec![],
                        id: DUMMY_NODE_ID,
                        span: sp,
                        node: Method_::MethDecl(
                            cx.ident_of("get_type"),
                            empty_generics(),
                            Abi::Rust,
                            Spanned {
                                node: ExplicitSelf_::SelfStatic,
                                span: sp
                            },
                            Unsafety::Normal,
                            cx.fn_decl(vec![], cx.ty_path(jit_type)),
                            cx.block_expr(
                                type_expr
                            ),
                            Visibility::Inherited
                        )
                    })),
                    ImplItem::MethodImplItem(P(Method {
                        attrs: vec![],
                        id: DUMMY_NODE_ID,
                        span: sp,
                        node: Method_::MethDecl(
                            cx.ident_of("compile"),
                            Generics {
                                lifetimes: vec![LifetimeDef {
                                    lifetime: jit_life,
                                    bounds: vec![]
                                }],
                                ty_params: OwnedSlice::empty(),
                                where_clause: WhereClause {
                                    id: DUMMY_NODE_ID,
                                    predicates: vec![]
                                }
                            },
                            Abi::Rust,
                            Spanned {
                                node: ExplicitSelf_::SelfRegion(None, Mutability::MutImmutable, cx.ident_of("b")),
                                span: sp
                            },
                            Unsafety::Normal,
                            cx.fn_decl(vec![
                                Arg::new_self(sp, Mutability::MutImmutable, cx.ident_of("self")),
                                cx.arg(sp, func, cx.ty_rptr(sp, cx.ty_path(jit_func) ,None, Mutability::MutImmutable))
                            ], cx.ty_path(jit_value)),
                            cx.block(sp, compiler, Some(cx.expr_ident(sp, value))),
                            Visibility::Inherited
                        )
                    }))
                ]
            )))
        },
        _ => {
            cx.span_err(sp, BAD_ITEM);
            return;
        }
    }
}
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern("jit"), SyntaxExtension::Decorator(Box::new(expand_jit) as Box<ItemDecorator>));
}

#[macro_export]
/// Construct a JIT struct with the fields given
macro_rules! jit_struct(
    ($($name:ident: $ty:ty),*) => ({
        Type::new_struct([
            $(get::<$ty>()),*
        ].as_mut_slice()).with_names(&[$(stringify!($name)),*])
    });
    ($($ty:ty),+ ) => ({
        Type::new_struct([
            $(get::<$ty>()),+
        ].as_mut_slice())
    })
);

#[macro_export]
/// Construct a JIT union with the fields given
macro_rules! jit_union(
    ($($name:ident: $ty:ty),*) => ({
        Type::new_union([
            $(get::<$ty>()),*
        ].as_mut_slice()).with_names(&[$(stringify!($name)),*])
    });
    ($($ty:ty),+ ) => ({
        Type::new_union([
            $(get::<$ty>()),+
        ].as_mut_slice())
    })
);
#[macro_export]
/// Construct a JIT function signature with the arguments and return type given
macro_rules! jit_fn(
    ($($arg:ty),* -> $ret:ty) => ({
        use std::default::Default;
        Type::new_signature(Default::default(), get::<$ret>(), [
            $(get::<$arg>()),*
        ].as_mut_slice())
    });
    (raw $($arg:expr),* -> $ret:expr) => ({
        use std::default::Default;
        Type::new_signature(Default::default(), $ret, [
            $($arg),*
        ].as_mut_slice())
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
        $func.insn_store($var, jit($func, $val));
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
        let sig = Type::new_signature(Default::default(), get::<$ret>(), [].as_mut_slice());
        $ctx.build_func(sig, |$func| $value)
    });
    ($ctx:expr, $func:ident, $name:ident($($arg:ident:$ty:ty),+) -> $ret:ty, $value:expr) => ({
        use std::default::Default;
        let sig = Type::new_signature(Default::default(), get::<$ret>(), [$(get::<$arg_ty>()),*].as_mut_slice());
        $ctx.build_func(sig, |$func| {
            let mut i = 0u;
            $(let $arg = {
                i += 1;
                $func[i - 1]
            };)*
            $value
        })
    });
    ($ctx:expr, $func:ident, $name:ident() -> $ret:ty, $value:expr, |$comp_func:ident| $comp:expr) => ({
        use std::default::Default;
        let sig = Type::new_signature(Default::default(), get::<$ret>(), [].as_mut_slice());
        $ctx.build_func(sig, |$func| $value)
            .with::<(), $ret, _>(|$comp_func| $comp)
    });
    ($ctx:expr, $func:ident, $name:ident($($arg:ident:$arg_ty:ty),+) -> $ret:ty, $value:expr, |$comp_func:ident| $comp:expr) => ({
        use std::default::Default;
        let sig = Type::new_signature(Default::default(), get::<$ret>(), [$(get::<$arg_ty>()),*].as_mut_slice());
        $ctx.build_func(sig, |$func| {
            let mut i = 0us;
            $(let $arg = {
                i += 1;
                $func[i - 1]
            };)*
            $value
        }).with::<($($arg_ty),*), $ret, _>(|$comp_func|
            $comp)
    });
);