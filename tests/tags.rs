#![feature(plugin)]
#![allow(unstable)]
#[no_link] #[plugin] #[macro_use]
extern crate jit_macros;
extern crate jit;
use jit::*;

#[derive(Debug, Eq, PartialEq)]
struct PanicDrop(isize);
impl Drop for PanicDrop {
    fn drop(&mut self) {
        panic!("Dropped {:?}", self)
    }
}
#[test]
#[should_fail]
fn test_panic_tags() {
    let pos_t = jit_struct!{
        x: f64,
        y: f64
    };
    let kind = pos_t.get_kind();
    let pos_t = Type::create_tagged(pos_t, kind, Box::new(PanicDrop(42)));
    assert_eq!(pos_t.get_tagged_data(), Some(&PanicDrop(42)));
}

#[test]
fn test_tags() {
    let pos_t = jit_struct!{
        x: f64,
        y: f64
    };
    let kind = pos_t.get_kind();
    let pos_t = Type::create_tagged(pos_t, kind, Box::new(42us));
    assert_eq!(pos_t.get_tagged_data(), Some(&42us));
    pos_t.set_tagged_data(Box::new(-1.0f64));
    assert_eq!(pos_t.get_tagged_data(), Some(&-1.0f64));
}
