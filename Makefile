# Makefile to build and run Messaging plugin for Spin framework.

.PHONY: all
all: build-plugin install-plugin

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

.PHONY: build-plugin
build-plugin:
	@echo "Building Messaging Plugin..."
	cargo build --release

.PHONY: install-plugin
install-plugin:
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

.PHONY: start-rabbitmq
start-rabbitmq:
	podman run -it --hostname spin-rabbitmq --name spin-rabbitmq -p 5672:5672,15672:15672 -e RABBITMQ_DEFAULT_USER=user -e RABBITMQ_DEFAULT_PASS=password docker.io/library/rabbitmq:3.8.22-management
