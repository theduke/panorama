
# Actions

fmt:
	cargo fmt --all

clippyfix:
	cargo clippy --fix -- -D warnings

changelog:
	git-cliff --prepend --output CHANGELOG.md

fix: clippyfix fmt

dump-default-config:
	cargo run -- --dump-default-config > config.yaml

# Linting

check-fmt:
	@echo Checking formatting...
	@echo Rust version:
	cargo fmt --version
	cargo fmt --all -- --check
	@echo Code is formatted!

check-clippy:
	@echo Checking for clippy warnings...
	@echo Clippy version:
	cargo clippy --version
	cargo clippy --locked --all -- -D warnings

# Find unused dependencies:
check-unused-deps:
	@echo Checking for unused dependencies...
	RUSTC_BOOTSTRAP=1 cargo udeps --all-targets --backend depinfo
	@echo No unused dependencies found!

check-sample-config:
	@echo "Checking for outdated sample config..."
	cargo run -- --dump-default-config | diff config.yaml -
	@echo "Sample config is up to date!"

check-cargo-deny:
	@echo "Checking for insecure dependencies..."
	cargo deny check -A warnings
	@echo "No insecure dependencies found!"

pre_lint:
	@echo "Running all lints..."

lint: pre_lint check-fmt check-clippy check-sample-config check-unused-deps check-cargo-deny
	@echo "All checks passed!"

test:
	@echo "Running all tests..."
	cargo test --all --all-features --locked
	@echo "All tests passed!"

ci: lint test
	@echo "All CI checks passed!"

.phony:
	echo hello
