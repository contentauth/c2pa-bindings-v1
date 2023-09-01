OS := $(shell uname)
CFLAGS = -I. -Wall 
ifeq ($(OS), Darwin)
CFLAGS += -framework Security
endif
ifeq ($(OS), Linux)
CFLAGS = -pthread -Wl,--no-as-needed -ldl -lm
endif

# default version of python is 3.11
PYTHON=python3.11

release: 
	cargo build --release --features=uniffi/cli

test_c: release
	cbindgen --config cbindgen.toml --crate c2pa-bindings --output tests/c/c2pa.h --lang c
	$(CC) $(CFLAGS) tests/c/main.c -o target/ctest -lc2pa_bindings -L./target/release 
	target/ctest

python: release
	cargo run --release --features=uniffi/cli --bin uniffi_bindgen generate src/c2pa.udl -n --language python -o target/python
	cp target/release/libc2pa_bindings.dylib target/python/libuniffi_c2pa.dylib

swift: release
	cargo run --release --features=uniffi/cli --bin uniffi_bindgen generate src/c2pa.udl -n --language swift -o target/swift
	cp target/release/libc2pa_bindings.dylib target/swift/libuniffi_c2pa.dylib

test_python: python
	$(PYTHON) tests/python/test.py

test: test_python test_c

