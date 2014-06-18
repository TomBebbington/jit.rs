RUSTC ?= rustc
RUSTDOC ?= rustdoc
.PHONY: all build doc libjit libjit_macros update-doc clean
all: build
libjit:
	mkdir -p target
	cd target && $(RUSTC) ../src/libjit/jit.rs -L .
libjit_macros:
	mkdir -p target
	cd target && $(RUSTC) ../src/libjit_macros/jit_macros.rs -L .
test:
	mkdir -p target
	cd target && $(RUSTC) --test ../src/libjit/jit.rs -L . -o jit_tests
build: libjit libjit_macros test
install:
	sudo cp -f target/libjit*.so /usr/local/lib
doc:
	$(RUSTDOC) src/libjit/jit.rs -o doc -L target
	$(RUSTDOC) src/libjit_macros/jit_macros.rs -o doc -L target
update-doc: doc
	rm -rf /tmp/doc
	mv doc /tmp/doc
	git checkout gh-pages
	rm -rf ./*
	mv /tmp/doc/* .
	-git add -A .
	-git commit -a -m "Auto-update docs"
	-git push origin gh-pages
	git checkout master
clean:
	rm -rf target/*
