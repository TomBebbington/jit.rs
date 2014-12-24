extern crate test;
use context::Context;
use get;
use std::default::Default;
use types::Type;
use types::kind::*;
use function::{Function, flags};
use label::Label;
use test::Bencher;
macro_rules! test_compile(
    ($ty:ty, $test_name:ident, $kind:expr) => (
        #[test]
        fn $test_name() {
            let default_value:$ty = Default::default();
            let mut ctx = Context::new();
            jit_func!(ctx, func, fn gen_value() -> $ty {
                let ref val = func.insn_of(&default_value);
                assert!(val.get_type().get_kind() == $kind);
                func.insn_return(val);
            }, |func| {
                assert_eq!(func(()), default_value);
            });
        }
    );
);
#[bench]
fn bench_raw_pi(b: &mut Bencher) {
    fn raw_pi() -> f64 {
        let mut pi = 3.0f64;
        let mut n = 2.0f64;
        let four = 4.0f64;
        while n < 100.0 {
            pi += four / (n * (n + 1.0) * (n + 2.0)) - four / ((n + 2.0) * (n + 3.0) * (n + 4.0));
            n += four;
        }
        pi
    }
    b.iter(|| raw_pi());
}
#[bench]
fn bench_pi(b: &mut Bencher) {
    let mut ctx = Context::new();
    jit_func!(ctx, func, fn pi() -> f64 {
        let f64_size = func.insn_of(&8u);
        let four = func.insn_of(&4.0f64);
        let three = func.insn_of(&3.0f64);
        let two = func.insn_of(&2.0f64);
        let one = func.insn_of(&1.0f64);
        let pi = func.insn_alloca(&f64_size);
        func.insn_store_relative(&pi, 0, &three);
        let n = func.insn_alloca(&f64_size);
        func.insn_store(&n, &two);
        let limit = func.insn_of(&100.0f64);
        func.insn_while(|| func.insn_lt(&func.insn_load(&n), &limit), || {
            let ln = func.insn_load(&n);
            let n_two = ln + two;
            let first_part = four / (ln * (ln + one) * n_two);
            let second_part = four / (n_two * (ln + three) * (ln + four));
            let new_pi = first_part - second_part;
            func.insn_store_relative(&pi, 0, &(func.insn_load_relative(&pi, 0, get::<f64>()) + new_pi));
            func.insn_store(&n, &(ln + four));
        });
        func.insn_return(&func.insn_load_relative(&pi, 0, get::<f64>()));
    }, |pi| {
        b.iter(|| pi(()));
    });
}
#[bench]
fn bench_raw_gcd(b: &mut Bencher) {
    fn gcd(x: uint, y: uint) -> uint {
        if x == y {
            x
        } else if x < y {
            gcd(x, y - x)
        } else {
            gcd(x - y, y)
        }
    }
    b.iter(|| gcd(70, 81));
}

#[bench]
fn bench_gcd(b: &mut Bencher) {
    let mut ctx = Context::new();
    jit_func!(ctx, func, fn gcd(x: uint, y:uint) -> uint {
        func.insn_if(&func.insn_eq(x, y), || func.insn_return(x));
        func.insn_if(&func.insn_lt(x, y), || {
            let mut args = [x, &(*y - *x)];
            let v = func.insn_call(Some("gcd"), func, None, args.as_mut_slice(), flags::JIT_CALL_NO_THROW);
            func.insn_return(&v);
        });
        let mut args = [&(*x - *y), y];
        let temp4 = func.insn_call(Some("gcd"), func, None, args.as_mut_slice(), flags::JIT_CALL_NO_THROW);
        func.insn_return(&temp4);
    }, |gcd| {
        b.iter(|| gcd((70, 81)));
    });
}
#[test]
fn test_sqrt() {
    let mut ctx = Context::new();
    assert_eq!(ctx.functions().count(), 0);
    jit_func!(ctx, func, fn sqrt(num: uint) -> uint {
        let sqrt = func.insn_sqrt(num);
        let sqrt_arg_ui = func.insn_convert(&sqrt, get::<uint>(), false);
        func.insn_return(&sqrt_arg_ui);
    }, |sqrt| {
        assert_eq!(sqrt(64), 8);
        assert_eq!(sqrt(16), 4);
        assert_eq!(sqrt(9), 3);
        assert_eq!(sqrt(4), 2);
        assert_eq!(sqrt(1), 1);
    });
    assert_eq!(ctx.functions().count(), 1);
}
#[test]
fn test_struct() {
    let pos_t = jit_struct!{
        x: f64,
        y: f64
    };
    for (i, field) in pos_t.fields().enumerate() {
        assert_eq!(field.get_type(), get::<f64>());
        assert_eq!(field.get_name().unwrap()[], match i {
            0 => "x",
            1 => "y",
            _ => unimplemented!()
        })
    }
}
struct PanicDrop {
    val: i32
}
impl Drop for PanicDrop {
    fn drop(&mut self) {
        panic!("Dropped {}", self.val)
    }
}
#[test]
#[should_fail]
fn test_tags() {
    let pos_t = jit_struct!{
        x: f64,
        y: f64
    };
    let kind = pos_t.get_kind();
    let pos_t = Type::create_tagged(pos_t, kind, box PanicDrop{val: 323});
    assert_eq!(pos_t.get_tagged_data::<PanicDrop>().map(|v| v.val), Some(323));
}

test_compile!((), test_compile_void, Void);
test_compile!(f64, test_compile_f64, Float64);
test_compile!(f32, test_compile_f32, Float32);
test_compile!(int, test_compile_int, NInt);
test_compile!(uint, test_compile_uint, NUInt);
test_compile!(i32, test_compile_i32, Int);
test_compile!(u32, test_compile_u32, UInt);
test_compile!(i16, test_compile_i16, Short);
test_compile!(u16, test_compile_u16, UShort);
test_compile!(i8, test_compile_i8, SByte);
test_compile!(u8, test_compile_u8, UByte);