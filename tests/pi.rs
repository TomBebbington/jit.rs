#![feature(plugin)]
#![plugin(jit_macros)]
#[no_link] #[macro_use]
extern crate jit_macros;
extern crate jit;
use jit::*;
use std::num::Float;
use std::iter::IteratorExt;
/*
#[jit]
fn pi() -> f64 {
    let mut pi = 3.0f64;
    for ln in range(0us, 25) {
        let ln = (ln * 4) as f64;
        let n_two = ln + 2.0;
        let first_part = 4.0 / (ln * (ln + 1.0) * n_two);
        let second_part = 4.0 / (n_two * (ln + 3.0) * (ln + 4.0));
        let new_pi = first_part - second_part;
        pi += new_pi;
    }
    pi
}
#[test]
fn test_pi() {
    let mut ctx = Context::new();
    jit_pi(ctx).with(|&:pi| {
        let pi = pi(());
        assert!((pi - 3.1415).abs() < 0.01);
    });
}
*/
