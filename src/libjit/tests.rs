use compile::Compile;
use context::Context;
use function::{CDECL, Function};
use std::default::Default;
use types::*;
fn with_empty_func(cb:|&Context, &Function| -> ()) -> () {
    let ctx = Context::new();
    ctx.build(|| {
        let sig = Type::create_signature(CDECL, get::<()>(), &mut[]);
        let func = Function::new(&ctx, sig);
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
    let func = Function::new(&context, sig);
    let arg = func.get_param(0);
    let sqrt_arg = func.insn_sqrt(&arg);
    let sqrt_arg_ui = func.insn_convert(&sqrt_arg, get::<uint>(), false);
    func.insn_return(&sqrt_arg_ui);
    func.compile();
    func.with_closure1(|sqrt:fn(uint) -> uint| {
        assert_eq!(sqrt(64), 8);
        assert_eq!(sqrt(16), 4);
        assert_eq!(sqrt(9), 3);
        assert_eq!(sqrt(4), 2);
        assert_eq!(sqrt(1), 1);
    })
}
#[test]
fn test_struct() {
    ::init();
    let float_t = get::<f64>();
    let double_float_t = Type::create_struct(&mut [float_t.clone(), float_t.clone()]);
    double_float_t.set_names(&["first", "second"]);
    assert_eq!(double_float_t.find_name("first"), 0);
    assert_eq!(double_float_t.find_name("second"), 1);
    let fields:Vec<(String, Type)> = double_float_t.iter_fields().collect();
    assert!(fields.as_slice() == [
        ("first".into_string(), float_t.clone()),
        ("second".into_string(), float_t)
    ])
    let mut iter = double_float_t.iter_fields();
    let (size, _) = iter.size_hint();
    assert_eq!(size, 2);
    assert!({
        let (field, _) = iter.nth(1).unwrap();
        field.as_slice() == "second"
    });
    assert_eq!(iter.count(), 0);
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