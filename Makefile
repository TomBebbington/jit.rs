CARGO ?= cargo
.PHONY: all build clean
all: build
build:
	$(CARGO) build
clean:
	rm -rf target/*jit*
