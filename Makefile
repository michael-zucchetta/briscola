

# cargo  install miniserve
#
init:
	rustup toolchain install nightly # set the nightly with rustup default 

build:
	wasm-pack build --target web --out-name wasm --out-dir ./static
	miniserve ./static --index index.html

