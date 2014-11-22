extern crate jit;
use jit::{get, Context, Function};
use std::f64::consts::PI;
use std::str::from_str;
use std::io::stdio::stdin;
use std::io::Buffer;
fn main() {
	let context = Context::new();
	let func = Function::new(&context, get::<fn(f64) -> f64>());
	context.build(|_| {
		let pi = func.insn_of(&PI);
		let radius = func.get_param(0);
		let radius_square = func.insn_mul(&radius, &radius);
		let result = func.insn_mul(&pi, &radius_square);
		func.insn_return(&result);
	});
	func.compile();
	func.with_closure1(|compute_area:fn(f64) -> f64| {
		let mut input = stdin();
		loop {
			let line = input.read_line().unwrap();
			let radius = from_str(line.as_slice().trim()).expect("Expected number for circle radius");
			let result = compute_area(radius);
			println!("Area of circle with radius {} = {}", radius, result);
		}
	})
}