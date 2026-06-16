# 构建阶段
FROM rust:1.95-alpine AS builder

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
    perl

WORKDIR /app

# 复制完整源码
COPY . .

# 设置环境变量
ENV OPENSSL_DIR=/usr \
    OPENSSL_INCLUDE_DIR=/usr/include/openssl \
    OPENSSL_LIB_DIR=/usr/lib

# 构建 release 版本
RUN cargo build --release

# 运行阶段
FROM alpine:3.21

RUN apk add --no-cache ca-certificates libssl3

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/ds-free-api /app/ds-free-api

# 创建配置目录和数据目录
RUN mkdir -p /app/config /app/data

# 设置环境变量（HF 适配）
ENV RUST_LOG=info \
    DS_DATA_DIR=/app/data \
    DS_CONFIG_PATH=/app/config/config.toml

# Hugging Face Spaces 要求使用 7860 端口
EXPOSE 7860

ENTRYPOINT ["/app/ds-free-api"]
