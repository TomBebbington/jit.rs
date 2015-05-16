#![feature(plugin)]
#![plugin(jit_macros)]
extern crate jit;
#[no_link] #[macro_use]
extern crate jit_macros;
static TEXT:&'static str = "Hello, world!";


extern fn print(text: &str) {
    println!("Print called");
    assert_eq!(text.len(), TEXT.len());
    assert_eq!(text.as_bytes().as_ptr(), TEXT.as_bytes().as_ptr());
    println!("{}", text);
}

fn main() {
    use jit::*;
    let mut ctx = Context::<()>::new();
    let sig = get::<fn()>();
    ctx.build_func(&sig, |func| {
        let text = TEXT.compile(func);
        let sig = get::<fn(&str)>();
        func.insn_call_native1(Some("print"), print, &sig, [text], flags::CallFlags::NO_THROW);
        func.insn_default_return();
        println!("{:?}", func);
    }).with(|cb| {
        cb(())
    });
}
