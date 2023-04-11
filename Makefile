NAME=srun
BINDIR=bin
VERSION=$(shell git describe --tags || echo "unknown version")
STRIP=llvm-strip -s
CARGO_BUILD=cargo build --release --target
CROSS_BUILD=cross build --release --target
UPX=-upx --best
TLS=FALSE

CROSS_TARGET_LIST = \
	x86_64-unknown-linux-musl \
	i686-unknown-linux-musl \
	aarch64-unknown-linux-musl \
	armv7-unknown-linux-musleabihf \
	mips-unknown-linux-musl \
	mipsel-unknown-linux-musl \
	mips64-unknown-linux-muslabi64 \
	mips64el-unknown-linux-muslabi64

$(CROSS_TARGET_LIST):
ifeq ($(TLS),TRUE)
	$(CROSS_BUILD) $@ --features tls
	cp "target/$@/release/$(NAME)" "$(BINDIR)/$(NAME)-tls-$@"
	$(STRIP) "$(BINDIR)/$(NAME)-tls-$@"
	$(UPX) "$(BINDIR)/$(NAME)-tls-$@"
else
	$(CROSS_BUILD) $@
	cp "target/$@/release/$(NAME)" "$(BINDIR)/$(NAME)-$@"
	$(STRIP) "$(BINDIR)/$(NAME)-$@"
	$(UPX) "$(BINDIR)/$(NAME)-$@"
endif

WINDOWS_TARGET_LIST = \
	x86_64-pc-windows-msvc \
	aarch64-pc-windows-msvc

$(WINDOWS_TARGET_LIST):
ifeq ($(TLS),TRUE)
	$(CARGO_BUILD) $@ --features tls
	cp "target/$@/release/$(NAME).exe" "$(BINDIR)/$(NAME)-tls-$@.exe"
	$(STRIP) "$(BINDIR)/$(NAME)-tls-$@.exe"
	zip -m -j "$(BINDIR)/$(NAME)-tls-$@-$(VERSION).zip" "$(BINDIR)/$(NAME)-tls-$@.exe"
else
	$(CARGO_BUILD) $@
	cp "target/$@/release/$(NAME).exe" "$(BINDIR)/$(NAME)-$@.exe"
	$(STRIP) "$(BINDIR)/$(NAME)-$@.exe"
	zip -m -j "$(BINDIR)/$(NAME)-$@-$(VERSION).zip" "$(BINDIR)/$(NAME)-$@.exe"
endif

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

windows: $(WINDOWS_TARGET_LIST)

bindir:
	rm -rf $(BINDIR)
	mkdir $(BINDIR)

bin_gz=$(addsuffix .gz, $(CROSS_TARGET_LIST))

$(bin_gz): %.gz : %
ifeq ($(TLS),TRUE)
	chmod +x $(BINDIR)/$(NAME)-tls-$(basename $@)
	gzip -f -S -$(VERSION).gz $(BINDIR)/$(NAME)-tls-$(basename $@)
else
	chmod +x $(BINDIR)/$(NAME)-$(basename $@)
	gzip -f -S -$(VERSION).gz $(BINDIR)/$(NAME)-$(basename $@)
endif

gz_release: $(bin_gz)

release: bindir gz_release

minsize:
	rustup toolchain install nightly
	rustup component add rust-src --toolchain nightly
	cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu --release
	upx --best --lzma ./target/x86_64-unknown-linux-gnu/release/srun
