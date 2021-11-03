NAME=sdusrun
TARGET=x86_64-unknown-linux-gnu

cbuild:
	cargo build --release
	cargo strip
	strip -s target/release/$(NAME)
	upx --best target/release/$(NAME)

xbuild:
	xargo build --target $(TARGET) --release
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
