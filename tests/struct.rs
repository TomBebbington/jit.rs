#![feature(core, plugin)]
#[no_link] #[plugin] #[macro_use]
extern crate jit_macros;
extern crate jit;
use jit::*;

#[test]
fn test_struct() {
    let pos_t = jit_struct!{
        x: f64,
        y: f64
    };
    for (i, field) in pos_t.fields().enumerate() {
        assert_eq!(field.get_name().unwrap(), match i {
            0 => "x",
            1 => "y",
            _ => unimplemented!()
        })
    }
}