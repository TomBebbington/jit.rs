#![feature(plugin)]
#![plugin(jit_macros)]
#[no_link] #[macro_use]
extern crate jit_macros;
extern crate jit;
use jit::*;

#[test]
fn test_gcd() {
    let mut ctx = Context::<()>::new();
    jit_func!(ctx, func, gcd(x: usize, y:usize) -> usize, {
        func.insn_if(func.insn_eq(x, y), || func.insn_return(x));
        func.insn_if(func.insn_lt(x, y), || {
            let mut args = [x, y - x];
            let v = func.insn_call(Some("gcd"), func, None, args.as_mut_slice(), flags::NO_THROW);
            func.insn_return(v);
        });
        let mut args = [x - y, y];
        let temp4 = func.insn_call(Some("gcd"), func, None, args.as_mut_slice(), flags::NO_THROW);
        func.insn_return(temp4);
    }, |gcd| {
        assert_eq!(gcd((90, 50)), 10)
    });
}
