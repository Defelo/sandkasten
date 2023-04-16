FROM rust:alpine as deps

WORKDIR /deps

COPY . .

RUN find . -type f ! -name Cargo.toml ! -name Cargo.lock -delete \
    && find . -type d -empty -delete \
    && find . -type d | while read d; do mkdir "$d/src" && touch "$d/src/lib.rs"; done


FROM rust:alpine AS builder

WORKDIR /build

RUN apk add --no-cache musl-dev clang mold

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_TARGET_DIR=/target
ENV RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=/usr/bin/mold"

COPY --from=deps /deps .

RUN cargo build --locked --release && rm -rf /build

COPY . .

RUN find . -exec touch {} \; \
    && cargo build --locked --release \
    && mkdir dist \
    && cp $(find /target/release/ -maxdepth 1 -executable -type f) dist/ \
    && strip dist/*


FROM scratch

ENV RUST_LOG=info

COPY --from=builder /build/dist /

ENTRYPOINT ["/changeme"]
