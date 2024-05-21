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
LIBRARY=libc2pa_bindings.dylib

release: 
	cargo build --release --features=uniffi/cli

test_c: release
	cbindgen --config cbindgen.toml --crate c2pa-bindings --output tests/c/c2pa.h --lang c
	$(CC) $(CFLAGS) tests/c/main.c -o target/ctest -lc2pa_bindings -L./target/release 
	LD_LIBRARY_PATH=target/release target/ctest

python: release
	cargo run --release --features=uniffi/cli --bin uniffi_bindgen generate src/c2pa.udl -n --language python -o target/python
	cp target/release/$(LIBRARY) target/python/

swift: release
	cargo run --release --features=uniffi/cli --bin uniffi_bindgen generate src/c2pa.udl -n --language swift -o target/swift
	cp target/release/$(LIBRARY) target/swift/

test_python: python
	$(PYTHON) tests/python/test.py

test: test_python test_c

