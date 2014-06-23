use compile::Compile;
use context::Context;
use function::{CDECL, Function};
use std::default::Default;
use types::*;
fn with_empty_func(cb:|&Context, &Function| -> ()) -> () {
    let ctx = Context::new();
    ctx.build(|| {
        let sig = Type::create_signature(CDECL, &get::<()>(), &mut[]);
        let func = Function::new(&ctx, &sig);
        cb(&ctx, &func)
    })
}
fn test_compile<T:Compile+Default>(kind:TypeKind) {
    with_empty_func(|_, func| {
        let pval:T = Default::default();
        let val = pval.compile(func);
        assert!(val.get_type().get_kind().contains(kind));
    })
}
#[test]
fn test_sqrt() {
    let context = Context::new();
    let sig = get::<fn(uint) -> uint>();
    let func = Function::new(&context, &sig);
    let arg = func.get_param(0);
    let sqrt_arg = func.insn_sqrt(&arg);
    let sqrt_arg_ui = func.insn_convert(&sqrt_arg, &get::<uint>(), false);
    func.insn_return(&sqrt_arg_ui);
    func.compile();
    unsafe {
        let comp:fn(uint) -> uint = func.closure1();
        assert_eq!(comp(64), 8);
        assert_eq!(comp(16), 4);
        assert_eq!(comp(9), 3);
        assert_eq!(comp(4), 2);
        assert_eq!(comp(1), 1);
    }
}
#[test]
fn test_struct() {
    ::init();
    let float_t = get::<f64>();
    let double_float_t = Type::create_struct(&mut [&float_t, &float_t]);
    double_float_t.set_names(&["first", "second"]);
    assert_eq!(double_float_t.find_name("first"), 0);
    assert_eq!(double_float_t.find_name("second"), 1);
    let mut iter = double_float_t.iter_fields();
    assert!(iter.next() == Some(("first".into_string(), float_t.clone())));
    assert!(iter.next() == Some(("second".into_string(), float_t)));
}
#[test]
fn test_compiles() {
    test_compile::<()>(Void);
    test_compile::<f64>(Float64);
    test_compile::<f32>(Float32);
    test_compile::<int>(NInt);
    test_compile::<uint>(NUInt);
    test_compile::<i32>(Int);
    test_compile::<u32>(UInt);
    test_compile::<i16>(Short);
    test_compile::<u16>(UShort);
    test_compile::<i8>(SByte);
    test_compile::<u8>(UByte);
    test_compile::<bool>(SysBool);
    test_compile::<char>(SysChar);
}