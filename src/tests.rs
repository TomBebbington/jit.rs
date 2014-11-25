use compile::Compile;
use context::Context;
use function::UncompiledFunction;
use types::*;
use types::get as get_type;
use std::default::Default;
fn with_empty_func(cb:|&Context, &UncompiledFunction| -> ()) -> () {
    let ref ctx = Context::new();
    ctx.build(|| {
        let sig = get::<fn() -> ()>();
        let ref func = UncompiledFunction::new(ctx, sig);
        cb(ctx, func)
    })
}
macro_rules! test_compile(
    ($ty:ty, $test_name:ident, $kind:expr) => (
        #[test]
        fn $test_name() {
            with_empty_func(|_, func| {
                let pval:$ty = Default::default();
                let val = pval.compile(func);
                assert_eq!(val.get_type().get_kind(), $kind);
            })
        }
    );
)
#[test]
fn test_sqrt() {
    let sig = get::<fn(uint) -> uint>();
    let context = Context::new();
    let func = UncompiledFunction::new(&context, sig);
    context.build(|| {
        let arg = func.get_param(0);
        let sqrt_arg = func.insn_sqrt(&arg);
        let sqrt_arg_ui = func.insn_convert(&sqrt_arg, get::<uint>(), false);
        func.insn_return(&sqrt_arg_ui);
    });
    func.compile().with_closure1(|sqrt:extern fn(uint) -> uint| {
        assert_eq!(sqrt(64), 8);
        assert_eq!(sqrt(16), 4);
        assert_eq!(sqrt(9), 3);
        assert_eq!(sqrt(4), 2);
        assert_eq!(sqrt(1), 1);
    });
}
#[test]
fn test_struct() {
    ::init();
    let double_float_t = jit!(struct {
        "first": f64,
        "second": f64
    });
    assert_eq!(double_float_t.find_name("first").index, 0);
    assert_eq!(double_float_t.find_name("second").index, 1);
    let fields:Vec<String> = double_float_t.fields().map(|field| field.get_name().unwrap()).collect();
    assert_eq!(fields[], [
        "first".into_string(),
        "second".into_string()
    ][]);
    assert_eq!(double_float_t.fields().count(), 2);
    assert!({
        let field:Option<String> = double_float_t.fields().nth(1).unwrap().get_name();
        field.unwrap().as_slice() == "second"
    });
}
test_compile!((), test_compile_void, Void)
test_compile!(f64, test_compile_f64, Float64)
test_compile!(f32, test_compile_f32, Float32)
test_compile!(int, test_compile_int, NInt)
test_compile!(uint, test_compile_uint, NUInt)
test_compile!(i32, test_compile_i32, Int)
test_compile!(u32, test_compile_u32, UInt)
test_compile!(i16, test_compile_i16, Short)
test_compile!(u16, test_compile_u16, UShort)
test_compile!(i8, test_compile_i8, SByte)
test_compile!(u8, test_compile_u8, UByte)