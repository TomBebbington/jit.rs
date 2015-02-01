#![feature(plugin)]
#[no_link] #[plugin] #[macro_use]
extern crate jit_macros;
extern crate jit;
use jit::*;
use std::num::Float;

#[test]
fn test_pi() {
    let mut ctx = Context::new();
    jit_func!(ctx, func, pi() -> f64, {
        let f64_size = func.insn_of(&8us);
        let four = func.insn_of(&4.0f64);
        let three = func.insn_of(&3.0f64);
        let two = func.insn_of(&2.0f64);
        let one = func.insn_of(&1.0f64);
        let pi = func.insn_alloca(f64_size);
        func.insn_store_relative(pi, 0, three);
        let n = func.insn_alloca(f64_size);
        func.insn_store(n, two);
        let limit = func.insn_of(&100.0f64);
        func.insn_while(|| func.insn_lt(func.insn_load(n), limit), || {
            let ln = func.insn_load(n);
            let n_two = ln + two;
            let first_part = four / (ln * (ln + one) * n_two);
            let second_part = four / (n_two * (ln + three) * (ln + four));
            let new_pi = first_part - second_part;
            func.insn_store_relative(pi, 0, func.insn_load_relative(pi, 0, get::<f64>()) + new_pi);
            func.insn_store(n, ln + four);
        });
        func.insn_return(func.insn_load_relative(pi, 0, get::<f64>()));
    }, |pi| {
        let pi = pi(());
        assert!((pi - 3.1415).abs() < 0.01);
    });
}