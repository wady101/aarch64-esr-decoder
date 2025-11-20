BIN_NAME := aarch64-esr-decoder
PREFIX ?= $(HOME)/.local/fm_scripts

.PHONY: all build install install-release uninstall

all: build

build:
	cargo build

install: target/debug/$(BIN_NAME)
	@mkdir -p "$(PREFIX)"
	@install -m 0755 target/debug/$(BIN_NAME) "$(PREFIX)/$(BIN_NAME)"
	@echo "Installed debug binary to $(PREFIX)/$(BIN_NAME)"

install-release: target/release/$(BIN_NAME)
	@mkdir -p "$(PREFIX)"
	@install -m 0755 target/release/$(BIN_NAME) "$(PREFIX)/$(BIN_NAME)"
	@echo "Installed release binary to $(PREFIX)/$(BIN_NAME)"

uninstall:
	@rm -f "$(PREFIX)/$(BIN_NAME)"
	@echo "Removed $(PREFIX)/$(BIN_NAME)"

target/debug/$(BIN_NAME):
	cargo build

target/release/$(BIN_NAME):
	cargo build --release

