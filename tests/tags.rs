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
    let pos_t = TaggedType::new(pos_t, kind, Box::new(PanicDrop(42)));
    assert_eq!(pos_t.get_tagged_data(), Some(&PanicDrop(42)));
}

#[test]
fn test_tags() {
    let pos_t = jit_struct!{
        x: f64,
        y: f64
    };
    let kind = pos_t.get_kind();
    let new_pos_t = TaggedType::new(pos_t.clone(), kind, Box::new(42us));
    assert!(new_pos_t.get_tagged_data() == Some(&42us));
    assert!(new_pos_t.get_tagged_type() == pos_t);
}
