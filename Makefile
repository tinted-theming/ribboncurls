.PHONY: test test_cli test_lib build install

test: test_lib test_cli
	@echo "----------------------"
	@echo "Run all tests and lint"
	@echo "----------------------"

test_cli: build
	@echo "--------------------------"
	@echo "Running cli tests and lint"
	@echo "--------------------------"
	cargo fmt -p ribboncurls-cli --check || { echo "Formatting check failed"; exit 1; }
	git submodule update --init
	cargo test -p ribboncurls-cli --release

test_lib: build
	@echo "--------------------------"
	@echo "Running lib tests and lint"
	@echo "--------------------------"
	cargo fmt -p ribboncurls --check || { echo "Formatting check failed"; exit 1; }
	git submodule update --init
	cargo test -p ribboncurls --release

build: install
	@echo "-------------"
	@echo "Running build"
	@echo "-------------"
	cargo build --workspace --verbose --release
	cargo deny check || { echo "Dependency version mismatch error"; exit 1; }

install: 
	@echo "---------------"
	@echo "Installing deps"
	@echo "---------------"
	@if [ ! "$$(command -v cargo &>/dev/null)" ]; then \
		echo "Installing rustup"; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
	else \
		echo "rustup already installed"; \
	fi
	@if [ ! "$$(command -v cargo-deny &>/dev/null)" ]; then \
		echo "Installing cargo-deny"; \
		cargo install --locked cargo-deny; \
	else \
		echo "cargo-deny already installed"; \
	fi
