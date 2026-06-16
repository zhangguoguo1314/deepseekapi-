# 使用预构建二进制文件的 Dockerfile
# 二进制文件由 GitHub Actions 构建并提交到 bin/ 目录

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制预构建的二进制文件
COPY bin/ds-free-api /app/ds-free-api
RUN chmod +x /app/ds-free-api

# 创建配置目录和数据目录
RUN mkdir -p /app/config /app/data /app/logs

# 设置环境变量（HF 适配）
ENV RUST_LOG=info \
    DS_DATA_DIR=/app/data \
    DS_CONFIG_PATH=/app/config/config.toml

# Hugging Face Spaces 要求使用 7860 端口
EXPOSE 7860

ENTRYPOINT ["/app/ds-free-api"]
