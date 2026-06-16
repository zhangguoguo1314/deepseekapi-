# 前端构建阶段
FROM oven/bun:alpine AS web-builder

WORKDIR /app/web

# 复制前端依赖文件
COPY web/package.json web/bun.lockb ./
RUN bun install --frozen-lockfile

# 复制前端源码
COPY web/ .

# 构建前端
RUN bun run build

# Rust 构建阶段
FROM rust:1.95-alpine AS rust-builder

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    cmake \
    clang \
    llvm-dev \
    linux-headers \
    git \
    perl \
    g++

WORKDIR /app

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./
COPY ds_core/Cargo.toml ds_core/
COPY ds_core/src ds_core/src/

# 复制前端构建结果
COPY --from=web-builder /app/web/dist ./web/dist

# 复制 Rust 源码
COPY src ./src

# 设置环境变量
ENV OPENSSL_DIR=/usr

# 构建 release 版本
RUN cargo build --release

# 运行阶段
FROM alpine:3.21

RUN apk add --no-cache ca-certificates libssl3

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=rust-builder /app/target/release/ds-free-api /app/ds-free-api

# 创建配置目录和数据目录
RUN mkdir -p /app/config /app/data

# 设置环境变量（HF 适配）
ENV RUST_LOG=info \
    DS_DATA_DIR=/app/data \
    DS_CONFIG_PATH=/app/config/config.toml

# Hugging Face Spaces 要求使用 7860 端口
EXPOSE 7860

ENTRYPOINT ["/app/ds-free-api"]
