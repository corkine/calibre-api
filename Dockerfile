FROM rust:1.80 as builder
WORKDIR /app
COPY . .
ENV RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
ENV RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /calibre-data && mkdir -p /calibre-web

COPY --from=builder /app/target/release/calibre-api /calibre-web/calibre-api

WORKDIR /calibre-web
EXPOSE 8080
CMD ["/calibre-web/calibre-api"]