# jit.rs [![Build Status](https://travis-ci.org/TomBebbington/jit.rs.svg?branch=master)](https://travis-ci.org/TomBebbington/jit.rs)
jit.rs is a Rust library that wraps LibJIT, the lightweight open-source JIT
compiler library. It also includes a macro for parsing JIT-compatible
structs into the actual Type, and will soon have macro for generating JIT
instructions from Rust expressions.

## Building
If you want to build this, you'll need to install these packages:
``` bash
sudo apt install autoconf automake texinfo libtool bison flex g++

```

## Using the macro
Just annotate your types you want to pass into LibJIT like this
``` rust
#[jit]
struct Position {
  x: f64,
  y: f64
}

fn main() {
  ...
  let pos = Position {
    x: 5.0,
    y: -32.2
  };
  func.insn_return(func.insn_of(&pos))
  ...
}
```

## Brainfuck VM
You can run the simple brainfuck VM with this command:
``` bash
cargo run --example brainfuck
```
You should then type in the brainfuck code followed by a newline to run it. For example:
``` bash
cargo run --example brainfuck
> ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
Hello, world!
>
```
