

# cargo  install miniserve
#
init:
	rustup toolchain install nightly # set the nightly with rustup default 

wasm-pack:
	cargo install wasm-pack

build:
	CARGO_HOME=$(CURDIR)/.cargo-home TMPDIR=/tmp CARGO_TARGET_DIR=/tmp/briscola-wasm-pack-target wasm-pack build --target web --out-name wasm --out-dir ./static
	miniserve ./static --index index.html
