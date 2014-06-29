use compile::Compile;
use context::Context;
use function::{CDECL, Function};
use label::Label;
use std::default::Default;
use std::io::stdio::println;
use test::Bencher;
use types::*;
use get_type = types::get;
static BENCH_SIZE: uint = 200000;
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
    let sig = get::<fn(uint) -> uint>();
    let context = Context::new();
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
    let double_float_t = jit!(struct {
        "first": f64,
        "second": f64
    });
    assert_eq!(double_float_t.find_name("first"), 0);
    assert_eq!(double_float_t.find_name("second"), 1);
    let fields:Vec<String> = double_float_t.iter_fields().map(|field| field.get_name().unwrap()).collect();
    assert!(fields.as_slice() == [
        "first".into_string(),
        "second".into_string()
    ].as_slice())
    let mut iter = double_float_t.iter_fields();
    let (size, _) = iter.size_hint();
    assert_eq!(size, 2);
    assert!({
        let field:Option<String> = iter.nth(1).unwrap().get_name();
        field.unwrap().as_slice() == "second"
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
#[bench]
fn bench_native_fib(b: &mut Bencher) {
    fn fib(n: uint) -> uint {
        match n {
            0     => 0,
            1 | 2 => 1,
            3     => 2,
            _     => fib(n - 1) + fib(n - 2)
        }
    }
    b.iter(|| {
        fib(BENCH_SIZE);
    });
}
#[bench]
fn bench_jit_fib(b: &mut Bencher) {
    let context = Context::new();
    let sig = get::<fn(uint) -> uint>();
    let fib = Function::new(&context, sig.clone());
    let n = fib.get_param(0);
    println("Making jump table");
    jit!(&fib, jump_table(&n, zero, one, two, three));
    println("n - 1");
    let n_m1 = n - jit!(fib, 1u);
    println("fib_m1");
    let fib_m1 = jit!(fib, call(&fib, &n_m1));
    println("fib_m2");
    let n_m2 = n - jit!(fib, 2u);
    let fib_m2 = jit!(fib, call(&fib, &n_m2));
    println("Adding");
    let result = fib_m1 + fib_m2;
    println("Returning");
    jit!(fib, return &result);
    fib.compile();
    fib.with_closure1(|fib:fn(uint) -> uint| {
        b.iter(|| {
            fib(BENCH_SIZE);
        });
    });
}