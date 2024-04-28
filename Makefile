.SUFFIXES: # disable builtin rules

.PHONY: all
all: lint fmt-check test


.PHONY: build
build:
	cargo build --all-targets --all

.PHONY: test
test:
	cargo test --all

.PHONY: check
check:
	cargo check --all-targets --all

.PHONY: lint
lint:
	cargo clippy --all-targets --all -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check
