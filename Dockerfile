FROM --platform=$BUILDPLATFORM rust:1.61.0-buster AS build

ARG TARGETARCH
ARG AUTH_SERVER_IP
ENV AUTH_SERVER_IP=$AUTH_SERVER_IP

RUN apt-get update && apt-get install -y build-essential curl musl-tools upx

WORKDIR /root/sdusrun

ADD . .

RUN rustup install nightly && rustup default nightly && \
    case "$TARGETARCH" in \
    "386") \
        RUST_TARGET="i686-unknown-linux-musl" \
        MUSL="i686-linux-musl" \
        ;; \
    "amd64") \
        RUST_TARGET="x86_64-unknown-linux-musl" \
        MUSL="x86_64-linux-musl" \
        ;; \
    "arm64") \
        RUST_TARGET="aarch64-unknown-linux-musl" \
        MUSL="aarch64-linux-musl" \
        ;; \
    *) \
        echo "Doesn't support $TARGETARCH architecture" \
        exit 1 \
        ;; \
    esac && \
    wget -qO- "https://musl.cc/$MUSL-cross.tgz" | tar -xzC /root/ && \
    CC=/root/$MUSL-cross/bin/$MUSL-gcc && \
    rustup target add $RUST_TARGET && \
    RUSTFLAGS="-C linker=$CC" CC=$CC cargo build --target "$RUST_TARGET" --release && \
    mv target/$RUST_TARGET/release/sdusrun target/release/ && \
    upx -9 target/release/sdusrun

FROM alpine:3.14 AS sdusrun

COPY --from=build /root/sdusrun/target/release/sdusrun /usr/bin
ENTRYPOINT [ "sdusrun" ]
