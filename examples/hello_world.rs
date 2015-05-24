extern crate jit;
#[no_link] #[macro_use]
extern crate jit_macros;
static TEXT:&'static str = "Hello, world!";

fn main() {
    use jit::*;
    let mut ctx = Context::<()>::new();
    jit_func!(&mut ctx, func, fn() -> &'static str {
        func.insn_return(TEXT.compile(func));
    }, assert_eq!(TEXT, func()));
}
