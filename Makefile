RUSTC ?= rustc
RUSTC_ARGS ?= -O -L target -L target/deps --out-dir=target
CARGO ?= cargo
RUSTDOC ?= rustdoc
.PHONY: all target native build doc clean
all: build doc
target:
	mkdir -p target
build: target
	$(CARGO) build
native:
	cd native && ./auto_gen.sh
	cd native && ./configure
	cd native && make
	mkdir -p native/root
	cd native && make DESTDIR=$(CURDIR)/native/root install
	-ln -s native/root/libjit.so target/deps/libjit.so
doc: build
	rm -rf doc
	$(CARGO) doc
clean:
	rm -rf target/*jit*
