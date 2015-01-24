#![feature(plugin)]
#![allow(unstable)]
#[no_link] #[plugin] #[macro_use]
extern crate jit_macros;
extern crate jit;
extern crate test;
use jit::*;
use std::default::Default;
macro_rules! test_compile(
    ($ty:ty, $test_name:ident, $kind:expr) => (
        #[test]
        fn $test_name() {
            let default_value:$ty = Default::default();
            let mut ctx = Context::new();
            jit_func!(ctx, func, gen_value() -> $ty, {
                let val = func.insn_of(&default_value);
                func.insn_return(val);
            }, |func| {
                assert_eq!(func(()), default_value);
            });
        }
    );
);
test_compile!((), test_compile_void, Void);
test_compile!(f64, test_compile_f64, Float64);
test_compile!(f32, test_compile_f32, Float32);
test_compile!(isize, test_compile_isize, NInt);
test_compile!(usize, test_compile_usize, NUInt);
test_compile!(i32, test_compile_i32, Int);
test_compile!(u32, test_compile_u32, UInt);
test_compile!(i16, test_compile_i16, Short);
test_compile!(u16, test_compile_u16, UShort);
test_compile!(i8, test_compile_i8, SByte);
test_compile!(u8, test_compile_u8, UByte);