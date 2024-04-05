.SUFFIXES: # disable builtin rules

.PHONY: all
all: lint test


.PHONY: build
build:
	cargo build --all-targets --all

.PHONY: test
test:
	cargo test --all-targets --all

.PHONY: check
check:
	cargo check --all-targets --all --verbose

.PHONY: lint
lint:
	cargo clippy --all-targets --all -- -D warnings
