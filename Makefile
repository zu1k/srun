NAME=sdusrun
TARGET=x86_64-unknown-linux-musl

cbuild:
	cargo build --target $(TARGET) --release
	cargo strip

xbuild:
	xargo build --target $(TARGET) --release

min:
	strip -s target/$(TARGET)/release/$(NAME)
	upx --best target/$(TARGET)/release/$(NAME)

clean:
	cargo clean

install_deps:
	cargo install cargo-strip
	cargo install xargo

fmt:
	cargo fmt --all

clippy:
	cargo clippy
