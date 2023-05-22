# c2pa_uniffi

This uses the uniffi tools to create language bindings for the Rust c2pa-rs library.

The result is a dynamic library that can be called directly from python

This is a VERY experimental work in progress and will change considerably

Adding Manifests does not work here yet

# Building for C (or node)

Run `make release`

# Building for python

Run `make python`

# Building for node

Run `make build_node`

# Testing C

Run `make test_c`

# Testing python

Run `make test_python`

# Testing node

Run `make test_node`

# Testing all

Run `make test`