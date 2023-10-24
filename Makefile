
fmt:
	cargo fmt --all

clippyfix:
	cargo clippy --fix -- -D warnings

changelog:
	git-cliff --prepend --output CHANGELOG.md

fix: fmt clippyfix

dump-default-config:
	cargo run -- --dump-default-config > config.toml

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
	cargo run -- --dump-default-config | diff config.toml -
	@echo "Sample config is up to date!"

lint: check-fmt check-clippy check-sample-config check-unused-deps

test:
	cargo test --all --all-features --locked

.phony:
	echo hello
