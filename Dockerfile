# 简化版 Dockerfile - 直接在 HF 上构建
# 注意：构建时间约 10-15 分钟，请耐心等待

FROM rust:1.95-slim-bookworm AS builder

# 安装 Node.js 和构建工具
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    cmake \
    clang \
    libssl-dev \
    pkg-config \
    git \
    perl \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 先复制并构建前端
COPY web/package.json web/package-lock.json* web/bun.lock* ./web/
RUN cd web && npm install
COPY web/ ./web/
RUN cd web && npm run build

# 复制 Rust 项目文件
COPY Cargo.toml Cargo.lock ./
COPY ds_core ./ds_core
COPY src ./src

# 构建 Rust 项目
ENV OPENSSL_DIR=/usr
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/ds-free-api /app/ds-free-api

# 创建必要目录
RUN mkdir -p /app/config /app/data /app/logs

# 环境变量
ENV RUST_LOG=info \
    DS_DATA_DIR=/app/data \
    DS_CONFIG_PATH=/app/config/config.toml

EXPOSE 7860

ENTRYPOINT ["/app/ds-free-api"]
