#![feature(convert, custom_attribute, plugin)]
#![plugin(jit_macros)]
#[no_link] #[macro_use]
extern crate jit_macros;
extern crate jit;
use jit::*;

#[jit]
#[repr(packed)]
pub struct Position {
    x: f64,
    y: f64
}

#[jit]
#[repr(packed)]
pub struct Vector2<T> {
    a: T,
    b: T
}

#[test]
fn test_struct() {
    let pos_t = get::<Position>();
    for (i, field) in pos_t.fields().enumerate() {
        assert_eq!(field.get_name().unwrap(), match i {
            0 => "x",
            1 => "y",
            _ => unimplemented!()
        })
    }
}
