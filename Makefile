RUSTC ?= rustc
RUSTC_ARGS ?= -O -L target -L target/deps --out-dir=target
CARGO ?= cargo
RUSTDOC ?= rustdoc
.PHONY: all native build doc clean
all: build doc
build: native
	$(RUSTC) $(RUSTC_ARGS) src/libjit/jit.rs
	$(RUSTC) $(RUSTC_ARGS) src/libjit_macros/jit_macros.rs
native:
	cd native && ./auto_gen.sh
	cd native && ./configure
	cd native && make
	cd native && sudo make install
doc: build
	rm -rf doc
	$(RUSTDOC) src/libjit/jit.rs -o doc -L target -L target/deps
clean:
	rm -rf target/*jit*
