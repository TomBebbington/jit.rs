jit.rs [![Build Status](https://travis-ci.org/TomBebbington/jit.rs.svg?branch=master)](https://travis-ci.org/TomBebbington/jit.rs)
======
jit.rs is a Rust library that wraps LibJIT, the lightweight open-source JIT compiler library. It also includes a macro for parsing JIT-compatible types into the actual Type, and will soon have macro for generating JIT instructions
from Rust expressions.

If you want to build this, you'll need to install these packages:
```
sudo apt install autoconf automake texinfo libtool bison flex g++
```

Brainfuck VM
------------
You can run the simple brainfuck VM with this command:
```
cargo run --example brainfuck
```
You should then type in the brainfuck code followed by a newline to run it. For example:
```
cargo run --example brainfuck
> ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
Hello, world!
>
```