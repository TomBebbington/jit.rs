#![feature(std_misc, plugin)]
#![plugin(jit_macros)]
#[no_link] #[macro_use]
extern crate jit_macros;
extern crate jit;
use jit::*;

#[test]
fn test_sqrt() {
    let mut ctx = Context::<()>::new();
    assert_eq!(ctx.functions().count(), 0);
    jit!(&mut ctx, |num: usize| -> usize {
        (num as f32).sqrt() as usize
    }, |sqrt: extern fn(usize) -> usize| {
        assert_eq!(sqrt(64), 8);
        assert_eq!(sqrt(16), 4);
        assert_eq!(sqrt(9), 3);
        assert_eq!(sqrt(4), 2);
        assert_eq!(sqrt(1), 1);
    });
}

#[test]
fn test_alt_sqrt() {
    let mut ctx = Context::<()>::new();
    assert_eq!(ctx.functions().count(), 0);
    jit!(&mut ctx, |num: f32| -> f32 {
        num.sqrt()
    }, |sqrt: extern fn(f32) -> f32| {
        assert_eq!(sqrt(64.0), 8.0);
        assert_eq!(sqrt(16.0), 4.0);
        assert_eq!(sqrt(9.0), 3.0);
        assert_eq!(sqrt(4.0), 2.0);
        assert_eq!(sqrt(2.25), 1.5);
        assert_eq!(sqrt(1.0), 1.0);
    });
    assert_eq!(ctx.functions().count(), 1);
}
