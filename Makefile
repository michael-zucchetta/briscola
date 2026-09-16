.PHONY: init wasm-pack build build-production-white build-production-terminal github-pages serve verify-firefox

# cargo  install miniserve
#
init:
	rustup toolchain install nightly # set the nightly with rustup default 

wasm-pack:
	cargo install wasm-pack

build:
	CARGO_HOME=$(CURDIR)/.cargo-home TMPDIR=/tmp CARGO_TARGET_DIR=/tmp/briscola-wasm-pack-target wasm-pack build --target web --out-name wasm --out-dir ./static
	printf '%s\n' '*' '!.gitignore' '!index.html' '!card-zoom.js' '!card-zoom.css' '!wasm.js' '!wasm_bg.js' '!wasm_bg.wasm' '!wasm.d.ts' '!wasm_bg.wasm.d.ts' '!package.json' > ./static/.gitignore

build-production-white:
	CARGO_HOME=$(CURDIR)/.cargo-home TMPDIR=/tmp CARGO_TARGET_DIR=/tmp/briscola-wasm-pack-target RUSTFLAGS="-C opt-level=3" wasm-pack build --release --target web --out-name wasm --out-dir ./static
	printf '%s\n' '*' '!.gitignore' '!index.html' '!card-zoom.js' '!card-zoom.css' '!wasm.js' '!wasm_bg.js' '!wasm_bg.wasm' '!wasm.d.ts' '!wasm_bg.wasm.d.ts' '!package.json' > ./static/.gitignore
	sed -i '/<body /s/data-production-theme="[^"]*"/data-production-theme="white"/' static/index.html

build-production-terminal:
	CARGO_HOME=$(CURDIR)/.cargo-home TMPDIR=/tmp CARGO_TARGET_DIR=/tmp/briscola-wasm-pack-target RUSTFLAGS="-C opt-level=3" wasm-pack build --release --target web --out-name wasm --out-dir ./static
	printf '%s\n' '*' '!.gitignore' '!index.html' '!card-zoom.js' '!card-zoom.css' '!wasm.js' '!wasm_bg.js' '!wasm_bg.wasm' '!wasm.d.ts' '!wasm_bg.wasm.d.ts' '!package.json' > ./static/.gitignore
	sed -i '/<body /s/data-production-theme="[^"]*"/data-production-theme="terminal"/' static/index.html

github-pages: build-production-white
	mkdir -p docs
	cp static/index.html static/card-zoom.css static/card-zoom.js static/wasm.js static/wasm_bg.wasm docs/
	touch docs/.nojekyll
	@echo "GitHub Pages files ready in docs/. Commit and push docs/, then select your branch and /docs in Settings > Pages."

serve: build
	miniserve ./static --index index.html

verify-firefox: build
	python3 -m http.server 8001 --directory static > /tmp/briscola-http.log 2>&1 & server_pid=$$!; \
	trap 'kill $$server_pid' EXIT; \
	sleep 1; \
	curl -I http://127.0.0.1:8001/; \
	env MOZ_HEADLESS=1 MOZ_WEBRENDER=0 LIBGL_ALWAYS_SOFTWARE=1 timeout 30s firefox --headless --new-instance --profile /tmp/briscola-firefox-profile --window-size=1280,900 --screenshot /tmp/briscola-firefox.png http://127.0.0.1:8001/; \
	test -s /tmp/briscola-firefox.png

.PHONY: test-visual update-visual-baselines
test-visual: build
	python3 scripts/test_visual.py

update-visual-baselines: build
	python3 scripts/test_visual.py --update-baselines
