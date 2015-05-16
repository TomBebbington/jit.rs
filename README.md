jit.rs
======
[![Build Status](https://travis-ci.org/TomBebbington/jit.rs.svg?branch=master)](https://travis-ci.org/TomBebbington/jit.rs)
[![Latest Version](https://img.shields.io/crates/v/jit.svg)](https://crates.io/crates/jit)
jit.rs is a Rust library that wraps LibJIT, a lightweight open-source JIT
compiler library in an idiomatic Rust way, which includes a macro for easy
compilation of abritary types into JIT types

Building
--------
If you want to build this, you'll need to install these packages:
``` bash
sudo apt install autoconf automake texinfo libtool bison flex g++

```

Using the macro
---------------
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
  func.insn_return(func.insn_of(pos))
  ...
}
```

Brainfuck VM
------------
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
