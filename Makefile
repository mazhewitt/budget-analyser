.PHONY: test lint rust-test build serve

test: lint rust-test

lint:
	npx eslint static/

rust-test:
	cargo test

build:
	cargo build --release

serve:
	cargo run --release -- serve
