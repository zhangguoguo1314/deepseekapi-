# 构建阶段
FROM rust:1.95-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./
COPY ds_core/Cargo.toml ds_core/

# 创建虚拟 main.rs 来缓存依赖
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# 复制完整源码
COPY . .

# 重新构建（只编译修改的部分）
RUN cargo build --release

# 运行阶段
FROM alpine:3.21

RUN apk add --no-cache ca-certificates

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
