# 开发指南

## 环境要求

- Rust **1.95.0+**（见 `rust-toolchain.toml`）
- Bun **1.3+**（Web 面板构建与开发）
- `cmake`、`g++`、`libclang-dev`（编译 `wreq` 依赖的 BoringSSL）
- `just` 命令运行器（用于 `just serve` / `just check` 等快捷命令）

## 首次启动

```bash
# 1. 复制配置
cp config.example.toml config.toml

# 2. 构建 Web 前端（编译时嵌入二进制，每次前端变更需要重构建）
cd web && bun install && bun run build && cd ..

# 3. 运行开发服务器
just serve
```

服务器启动后访问 `http://localhost:22217` 自动跳转到管理面板。

> **前端热更新开发**：同时运行 `cd web && bun run dev`（Vite HMR 模式）
> 和 `just serve`，后端优先使用文件系统 `web/dist/` 目录中的静态文件。
> 无需每次前端改动都重构建二进制。

## Release 构建

```bash
# 1. 构建 Web 前端
cd web && bun install && bun run build && cd ..

# 2. 构建 Release 二进制
cargo build --release

# 3. 运行（也可直接运行二进制，无需 web/dist/ 目录）
./target/release/ds-free-api
```

Release 二进制通过 `rust_embed` 编译时嵌入前端资源，`web/dist/` 目录不存在时
自动使用嵌入资源。发布版无需额外文件。

## CI 自动构建

GitHub Actions（`.github/workflows/release.yml`）在 tag push 时自动执行：

```
build-frontend (bun install --frozen-lockfile + bun run build)
  ├── build-linux-gnu (cross)    │
  ├── build-linux-musl (cross)   │── release (tar.gz + zip)
  ├── build-macos (cargo build)  │
  └── build-windows (cargo build)│
  └── docker (ghcr.io image)
```

`build-frontend` 产出 `web-dist` artifact，各编译 job 下载后再执行 `cargo build` /
`cross build`，保证 `rust_embed` 嵌入真实前端文件。

Docker 镜像自动推送到 `ghcr.io/niyueee/ds-free-api:latest`。

## Docker 部署（生产）

从 ghcr.io 拉取（推荐）：

```bash
# 确认已创建 docker/config/ 目录（自动创建或手动 mkdir）
docker compose -f docker/docker-compose.yaml up -d
```

容器首次启动时自动创建最小配置，无需提前准备 `config.toml`。
配置和数据通过 bind mount 持久化到宿主机的 `docker/config/` 和 `docker/data/`。

从源码构建本地 Docker 镜像：

```bash
# 1. 构建前端 + 交叉编译二进制
cd web && bun install && bun run build && cd ..
cargo zigbuild --release --target x86_64-unknown-linux-gnu

# 2. 构建 Docker 镜像
docker build -f docker/Dockerfile -t ds-free-api .

# 3. 导出并传输到服务器
docker save ds-free-api | gzip > ds-free-api.tar.gz
scp ds-free-api.tar.gz user@server:/tmp/

# 4. 服务器加载并启动
ssh user@server
docker load < /tmp/ds-free-api.tar.gz
docker compose -f docker/docker-compose.yaml up -d
```

> 服务器原生 x86 环境可直接在服务器上执行上述构建，速度更快。
> Docker 镜像仅包含预编译二进制 + 嵌入的前端资源，无需在容器内编译。

## 命令参考

```bash
# 一键检查（check + clippy + fmt + audit + unused deps）
just check

# 运行测试
cargo test --lib

# 运行 HTTP 服务
just serve

# 统一协议调试 CLI（内置对话/比较/并发等模式）
just adapter-cli

# 使用 e2e 专属配置启动服务
just e2e-serve
```

## e2e 测试

`py-e2e-tests/` 是基于 JSON 场景驱动的端到端测试框架，无需 pytest 依赖。分为三层：

| 层级       | 命令              | 覆盖范围                                              |
| ---------- | ----------------- | ----------------------------------------------------- |
| **Basic**  | `just e2e-basic`  | 基础功能场景（双端点 OpenAI + Anthropic），安全并发数 |
| **Repair** | `just e2e-repair` | 工具调用异常格式修复专项（OpenAI 单端点），安全并发数 |
| **Stress** | `just e2e-stress` | 全部场景 × 3 次迭代，安全并发数 + 1 并发              |

先启动服务端：

```bash
just e2e-serve
```

再在另一个终端运行 e2e 测试：

```bash
# 基础场景测试
just e2e-basic

# 工具修复测试
just e2e-repair
```

场景文件在 `scenarios/` 中按类型独立存放：

```
py-e2e-tests/
├── scenarios/
│   ├── basic/
│   │   ├── openai/         # 10 个基础场景（对话、推理、流式、工具调用、文件上传、图片上传、HTTP链接等）
│   │   └── anthropic/      # 6 个基础场景（对话、推理、工具调用、文档上传、图片上传、HTTP链接）
│   └── repair/             # 10 个工具损坏格式场景
├── runner.py               # 单次运行入口
├── stress_runner.py        # 多迭代压测入口
└── config.toml             # e2e 专用服务端配置
```

每个场景为独立 JSON 文件，包含请求参数和校验规则：

```json
{
  "name": "场景名称",
  "endpoint": "openai|anthropic",
  "category": "basic|repair",
  "models": ["deepseek-default", "deepseek-expert"],
  "messages": [{"role": "user", "content": "..."}],
  "tools": [...],
  "tool_choice": "auto",
  "request": {"stream": false},
  "checks": {
    "has_tool_calls": true,
    "tool_names": ["get_weather"],
    "finish_reason": "tool_calls",
    "no_error": true
  }
}
```

### e2e CLI 参数

**`just e2e-basic` 和 `just e2e-repair`（单次运行）：**

| 参数 | 作用 |
|------|------|
| `scenario_dir` | 场景目录，如 `scenarios/basic` 或 `scenarios/repair` |
| `--endpoint` | 端点过滤：`openai` / `anthropic` |
| `--model` | 模型过滤：`deepseek-default` / `deepseek-expert` |
| `--filter` | 场景名称关键字过滤（多个用空格分隔，如 `--filter 文件 图片`）|
| `--parallel` | 并行数，默认 `账号数 ÷ 2` |
| `--show-output` | 显示模型回复摘要、工具调用、结束原因 |
| `--report` | 输出 JSON 报告路径 |

**`just e2e-stress`（压测）：**

| 参数 | 作用 |
|------|------|
| `--iterations` | 每场景迭代次数，默认 3 |
| `--models` | 模型列表过滤 |
| `--filter` | 场景名称关键字过滤（多个用空格分隔）|
| `--parallel` | 并行数，默认 `账号数 ÷ 2 + 1` |
| `--show-output` | 显示模型输出 |
| `--report` | 输出 JSON 报告路径 |

使用示例：

```bash
# 快速验证新加的文件上传场景
just e2e-basic --filter 文件 图片 --show-output

# 仅查看 OpenAI 端点的 expert 模型
just e2e-basic --endpoint openai --model deepseek-expert

# 串行调试
just e2e-basic --endpoint openai --parallel 1 --show-output

# 压测：工具调用修复场景 × 5 次迭代
just e2e-stress --filter 修复 --iterations 5

# 输出 JSON 报告
just e2e-basic --report result.json
```

## 更多文档

- [代码规范](code-style.md)
- [日志规范](logging-spec.md)
- [DeepSeek API 参考](deepseek-api-reference.md)
- [Prompt 注入策略](deepseek-prompt-injection.md)