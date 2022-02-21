# rustup target add wasm32-unknown-unknown
# cargo run --target wasm32-unknown-unknown
# cargo install wasm-bindgen-cli
# cargo install wasm-pack
# cargo  install miniserve
#
build:
	wasm-pack build --target web --out-name wasm --out-dir ./static
	miniserve ./static --index index.html

