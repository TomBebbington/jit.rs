JIT.rs
======
[![Build Status](https://travis-ci.org/TomBebbington/jit.rs.svg?branch=master)](https://travis-ci.org/TomBebbington/jit.rs)
[![Latest Version](https://img.shields.io/crates/v/jit.svg)](https://crates.io/crates/jit)

NOTE
----
Because LibJIT has been inactive for a year, it seems there won't be any more updates. On the other hand, LLVM is still very active, so I have been working on bindings for a couple of days now. There will be news soon about a replacement.

What is LibJIT?
---------------
LibJIT is a portable, lightweight JIT library developed by GNU in C. It aims to
have an IR which is compatible with any language or runtime you throw at it, without
forcing the programmer to use a specific one.

What's jit.rs?
--------------
jit.rs is a Rust library that wraps LibJIT in an idiomatic Rust way, which
includes a macro for easy compilation of abritary types into JIT types. It also
uses the Rust memory model to save some otherwide pointless operations that
would me used if it were implemented in a GC language


Why are there so many packages?
-------------------------------
+ `libjit-sys` - this contains the bindings to JIT functions,
constants etc. If you want to stick to the original API, you should use this.
+ `jit` - this contains the Rust-style wrappers of the JIT
functions and structures.
+ `jit_macros` - this contains the macro definitions that help
to streamline code interacting with JIT.


How do I build this?
--------------------
If you want to build this, you'll need to install these packages for Ubuntu or Debian:

``` bash
sudo apt install autoconf automake texinfo libtool bison flex g++

```

Then, you can just use cargo to build it

```bash
cargo build
```

How do I use the macro?
-----------------------
Just annotate your types you want to pass into LibJIT like this
``` rust
#[derive(Compile)]
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

Are there any examples?
-----------------------
There's a Brainfuck virtual machine example with an nice command-line interface
and everything. You can run it with this command from the project root:

``` bash
cargo run --example brainfuck
```

You should then enter in the brainfuck code followed by a newline to run it.

``` bash
> ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
Hello, world!
>
```
You can also view its source code [here](https://github.com/TomBebbington/jit.rs/blob/master/examples/brainfuck.rs).
