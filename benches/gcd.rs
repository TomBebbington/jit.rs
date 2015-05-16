#![feature(test, plugin)]
#![plugin(jit_macros)]
#[no_link] #[macro_use]
extern crate jit_macros;
extern crate jit;
extern crate test;
use test::Bencher;
use jit::*;

#[bench]
fn bench_gcd(b: &mut Bencher) {
    let mut ctx = Context::<()>::new();
    jit_func!(ctx, func, gcd(x: usize, y:usize) -> usize, {
        func.insn_if(func.insn_eq(x, y), || func.insn_return(x));
        func.insn_if(func.insn_lt(x, y), || {
            let mut args = [x, y - x];
            let v = func.insn_call(Some("gcd"), func, None, args.as_mut_slice(), flags::CallFlags::NO_THROW);
            func.insn_return(v);
        });
        let mut args = [x - y, y];
        let temp4 = func.insn_call(Some("gcd"), func, None, args.as_mut_slice(), flags::CallFlags::NO_THROW);
        func.insn_return(temp4);
    }, |gcd| {
        b.iter(|| {
            assert_eq!(gcd((90, 50)), 10)
        })
    });
}
#[bench]
fn bench_raw_gcd(b: &mut Bencher) {
    fn gcd(x: usize, y: usize) -> usize {
        if x == y {
            x
        } else if x < y {
            gcd(x, y - x)
        } else {
            gcd(x - y, y)
        }
    }
    b.iter(|| {
        assert_eq!(gcd(90, 50), 10)
    })
}
