toolchain:
	./scripts/init.sh

build:
	cargo build --release
	cp ./customspec.json ./target/release/customspec.json

install:
	cargo install --force --path .
init: toolchain
