# Detect OS and Architecture
OS := $(shell uname -s | tr A-Z a-z)
ARCH := $(shell uname -m)

# e.g., haproxy-add-darwin-arm64 or cleandl-rs-linux-x86_64
TARGET_NAME := haproxy-add-$(OS)-$(ARCH)

install:
	cargo build --release
	cp target/release/haproxy-add ~/.local/bin/$(TARGET_NAME)
	ln -sfn ~/.local/bin/$(TARGET_NAME) ~/.local/bin/haproxy-add
