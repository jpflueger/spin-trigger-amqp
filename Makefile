# Makefile to build and run Messaging plugin for Spin framework.

.PHONY: all
all: build_plugin install_plugin

.PHONY: lint
lint:
	@echo "Running linting check..."
	cargo clippy --all --all-targets -- -D warnings
	cargo fmt --all -- --check

.PHONY: lint-rust-examples
lint-rust-examples:	
		@echo "Running linting check on example..." \
		&& cargo clippy --manifest-path "examples/amqp-rust/Cargo.toml" -- -D warnings \
		&& cargo fmt --manifest-path "examples/amqp-rust/Cargo.toml" -- --check \

.PHONY: lint-all
lint-all: lint lint-rust-examples

.PHONY: build_plugin
build_plugin:
	@echo "Building Messaging Plugin..."
	cargo build --release

.PHONY: install_plugin
install_plugin:
	@echo "Installing Messaging Plugin in Spin..."
	spin plugins update && spin plugins upgrade pluginify -y
	spin pluginify --install
	
.PHONY: clean
clean:
	@echo "Cleaning up..."
	cargo clean
	cargo clean --manifest-path ./examples/amqp-rust/Cargo.toml
	rm -f trigger-amqp-*.tar.gz
	rm -f trigger-amqp.json
	spin plugin uninstall trigger-amqp