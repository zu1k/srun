NAME=sdusrun
include scripts/Makefile.release

all: fmt clippy cbuild

cbuild:
	cargo build

clean:
	cargo clean

deps:
	cargo install cargo-strip xargo cross

fmt:
	cargo fmt --all

clippy:
	cargo clippy

a: fmt clippy

