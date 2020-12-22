toolchain:
	./scripts/init.sh

build:
	SKIP_WASM_BUILD= cargo build

install:
	cargo install --force --path .

init: toolchain