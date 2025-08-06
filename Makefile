# Makefile untuk projek Rust static (musl)
TARGET = x86_64-unknown-linux-musl
BIN = target/$(TARGET)/release/vantara_de
USERLAND := ../vantara_os
ROOTFS := build/rootfs
DE_OUTPUT := $(USERLAND)/$(ROOTFS)/usr/bin
DE_DIR := ../vantara_os
DE_BINARY := target/$(TARGET)/release/vantara_de

.PHONY: all build clean run

all: build

build:
	@echo ">> Building static binary..."
	rustup target add $(TARGET)
	cargo build --release --target=$(TARGET)

copy-de:
	@echo "[Copy] DE to $(DE_OUTPUT)..."
	@mkdir -p $(DE_OUTPUT)
	cp $(DE_BINARY) $(DE_OUTPUT)

clean:
	@echo ">> Cleaning build artefacts..."
	cargo clean

run: build
	@echo ">> Running binary..."
	$(BIN)
