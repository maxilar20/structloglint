.PHONY: fmt lint test check build dev clean install-hooks

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

test:
	cargo test

check: fmt lint test

build:
	cargo build --release

dev:
	maturin develop --release

clean:
	cargo clean

install-hooks:
	pre-commit install --hook-type pre-commit --hook-type commit-msg
