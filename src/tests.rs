use context::Context;
use get;
use std::default::Default;
use types::Type;
use types::kind::*;
use Function;
macro_rules! test_compile(
    ($ty:ty, $test_name:ident, $kind:expr) => (
        #[test]
        fn $test_name() {
            let default_value:$ty = Default::default();
            Context::new().build_func(get::<fn() -> $ty>(), |func| {
                let ref val = func.insn_of(&default_value);
                assert_eq!(val.get_type().get_kind(), $kind);
                func.insn_return(val);
            }).with(|func: extern fn(()) -> $ty| {
                assert_eq!(func(()), default_value);
            });
        }
    );
)
type SQRT = extern fn(uint) -> uint;
#[test]
fn test_sqrt() {
    Context::new().build_func(get::<SQRT>(), |func| {
        let ref arg = func[0];
        assert_eq!(arg.get_type(), get::<uint>());
        let sqrt_arg = func.insn_sqrt(arg);
        let sqrt_arg_ui = func.insn_convert(&sqrt_arg, get::<uint>(), false);
        func.insn_return(&sqrt_arg_ui);
    }).with(|sqrt: SQRT| {
        assert_eq!(sqrt(64), 8);
        assert_eq!(sqrt(16), 4);
        assert_eq!(sqrt(9), 3);
        assert_eq!(sqrt(4), 2);
        assert_eq!(sqrt(1), 1);
    });
}
#[test]
fn test_struct() {
    let pos_t = jit!(struct {
        "x": f64,
        "y": f64
    });
    for (i, field) in pos_t.fields().enumerate() {
        assert_eq!(field.get_type(), get::<f64>());
        assert_eq!(field.get_name().unwrap()[], match i {
            0 => "x",
            1 => "y",
            _ => unimplemented!()
        })
    }
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
