test: release
	python tests/test_c2pa.py

release: 
	cargo build --release --features=uniffi/cli

python: release
	cargo run --release --features=uniffi/cli --bin uniffi_bindgen generate src/c2pa_uniffi.udl -n --language python -o target/python
	cp target/release/libc2pa_uniffi.dylib target/python/libuniffi_c2pa_uniffi.dylib

swift: release
	cargo run --release --features=uniffi/cli --bin uniffi_bindgen generate src/c2pa_uniffi.udl -n --language swift -o target/swift
	cp target/release/libc2pa_uniffi.dylib target/swift/libuniffi_c2pa_uniffi.dylib