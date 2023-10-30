
# Actions

fmt:
	@echo "Formatting rust code with cargo fmt..."
	cargo fmt --all
	@echo "Code formatted!"
	@echo ""

clippyfix:
	@echo "Fixing clippy lints..."
	cargo clippy --fix -- -D warnings
	@echo ""

changelog:
	@echo "Generating changelog..."
	git-cliff --prepend --output CHANGELOG.md
	@echo "Changelog generated!"
	@echo ""

fix: clippyfix fmt
	@echo "All fixes applied!"
	@echo ""

dump-default-config:
	@echo "Dumping default config..."
	cargo run -- --dump-default-config > config.yaml

# Linting

check-fmt:
	@echo Checking formatting...
	@echo Rust version:
	cargo fmt --version
	cargo fmt --all -- --check
	@echo Code is formatted!
	@echo ""

check-clippy:
	@echo Checking for clippy warnings...
	@echo Clippy version:
	cargo clippy --version
	cargo clippy --locked --all -- -D warnings
	@echo ""

# Find unused dependencies:
check-unused-deps:
	@echo Checking for unused dependencies...
	RUSTC_BOOTSTRAP=1 cargo udeps --all-targets --backend depinfo
	@echo No unused dependencies found!
	@echo ""

check-sample-config:
	@echo "Checking for outdated sample config..."
	cargo run -- --dump-default-config | diff config.yaml -
	@echo "Sample config is up to date!"
	@echo ""

check-cargo-deny:
	@echo "Checking for insecure dependencies..."
	cargo deny check -A warnings
	@echo "No insecure dependencies found!"
	@echo ""

pre_lint:
	@echo "Running all lints..."
	@echo ""

lint: pre_lint check-fmt check-clippy check-sample-config check-unused-deps check-cargo-deny
	@echo "All checks passed!"
	@echo ""

test:
	@echo "Running all tests..."
	cargo test --all --all-features --locked
	@echo "All tests passed!"
	@echo ""

ci: lint test
	@echo "All CI checks passed!"
	@echo ""

.phony:
	echo hello
