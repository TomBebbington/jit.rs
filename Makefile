RUSTC ?= rustc
CARGO ?= cargo
RUSTDOC ?= rustdoc
.PHONY: all build doc clean
all: build doc
build:
	$(CARGO) build
doc: build
	rm -rf doc
	$(RUSTDOC) src/libjit/jit.rs -o doc -L target/deps
clean:
	rm -rf target/*jit*
