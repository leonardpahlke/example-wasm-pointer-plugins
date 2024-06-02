.PHONY: build build-wasm-plugin build-plugin-runner clean clean-build run help install uninstall

PLUGIN_RUNNER_PATH := ./plugin-runner
WASM_PLUGIN_PATH := ./wasm-plugin
PLUGIN_SHARED_PATH := ./plugin-shared
RELEASE_TARGET := --release

help:
	@echo "Available commands:"
	@echo "  build                   Build the plugin runner and wasm plugin"
	@echo "  clean-build             Clean and then build both components"
	@echo "  build-wasm-plugin Build the wasm plugin"
	@echo "  build-plugin-runner     Build the plugin runner"
	@echo "  clean                   Clean build artifacts"
	@echo "  run                     Run the plugin runner"
	@echo "  test                    Run all Rust tests"

build: build-plugin-shared build-plugin-runner build-wasm-plugin

clean-build: clean build

build-plugin-shared:
	cargo build $(RELEASE_TARGET) --manifest-path $(PLUGIN_SHARED_PATH)/Cargo.toml

build-wasm-plugin:
	cargo build $(RELEASE_TARGET) --target wasm32-wasi --manifest-path $(WASM_PLUGIN_PATH)/Cargo.toml

build-plugin-runner:
	cargo build $(RELEASE_TARGET) --manifest-path $(PLUGIN_RUNNER_PATH)/Cargo.toml

clean:
	cargo clean --manifest-path $(PLUGIN_SHARED_PATH)/Cargo.toml
	cargo clean --manifest-path $(WASM_PLUGIN_PATH)/Cargo.toml
	cargo clean --manifest-path $(PLUGIN_RUNNER_PATH)/Cargo.toml

run:
	$(PLUGIN_RUNNER_PATH)/target/release/plugin-runner

test:
	cargo test --manifest-path $(PLUGIN_SHARED_PATH)/Cargo.toml
	cargo test --manifest-path $(WASM_PLUGIN_PATH)/Cargo.toml
	cargo test --manifest-path $(PLUGIN_RUNNER_PATH)/Cargo.toml