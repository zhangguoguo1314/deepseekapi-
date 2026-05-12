# Development Guide

## Prerequisites

- Rust **1.95.0+** (see `rust-toolchain.toml`)
- Bun **1.3+** (for web panel development and build)
- `cmake`, `g++`, `libclang-dev` (required to compile BoringSSL for `wreq`)
- `just` command runner (used for `just serve` / `just check` etc.)

## First-time Setup

```bash
# 1. Copy configuration
cp config.example.toml config.toml

# 2. Build web frontend (compiled into binary via rust_embed, rebuild on frontend changes)
cd web && bun install && bun run build && cd ..

# 3. Start dev server
just serve
```

Access `http://localhost:22217` — it redirects to the admin panel.

> **Frontend HMR development**: Run `cd web && bun run dev` (Vite HMR) alongside
> `just serve`. The backend reads files from the `web/dist/` directory when available,
> so frontend changes reflect immediately without rebuilding the binary.

## Release Build

```bash
# 1. Build web frontend
cd web && bun install && bun run build && cd ..

# 2. Build release binary
cargo build --release

# 3. Run (binary includes embedded frontend, no extra files needed)
./target/release/ds-free-api
```

The release binary embeds the frontend assets via `rust_embed` at compile time.
When `web/dist/` is absent at runtime, the server falls back to the embedded copy.
No extra files needed for deployment.

## CI Build Pipeline

On tag push, GitHub Actions (`.github/workflows/release.yml`) runs:

```
build-frontend (bun install --frozen-lockfile + bun run build)
  ├── build-linux-gnu (cross)    │
  ├── build-linux-musl (cross)   │── release (tar.gz + zip)
  ├── build-macos (cargo build)  │
  └── build-windows (cargo build)│
  └── docker (ghcr.io image)
```

The `build-frontend` job produces a `web-dist` artifact. Every build job downloads it
before running `cargo build` / `cross build`, ensuring `rust_embed` embeds the real
frontend assets.

Docker images are pushed automatically to `ghcr.io/niyueee/ds-free-api:latest`.

## Docker Deployment (Production)

Pull from ghcr.io (recommended):

```bash
docker compose -f docker/docker-compose.yaml up -d
```

On first run, the container auto-creates a minimal config — no need to prepare
`config.toml` upfront. Configuration and data persist via bind mounts at
`docker/config/` and `docker/data/` on the host.

Local Docker image build:

```bash
# 1. Build frontend + cross-compile binary
cd web && bun install && bun run build && cd ..
cargo zigbuild --release --target x86_64-unknown-linux-gnu

# 2. Build Docker image
docker build -f docker/Dockerfile -t ds-free-api .

# 3. Export and transfer to server
docker save ds-free-api | gzip > ds-free-api.tar.gz
scp ds-free-api.tar.gz user@server:/tmp/

# 4. Load and run on server
ssh user@server
docker load < /tmp/ds-free-api.tar.gz
docker compose -f docker/docker-compose.yaml up -d
```

> Building directly on a native x86 server is faster. The Docker image contains
> only the pre-compiled binary with embedded frontend — no compilation inside the container.

## Commands

```bash
# One-pass check (check + clippy + fmt + audit + unused deps)
just check

# Run tests
cargo test --lib

# Run HTTP server
just serve

# Unified protocol debug CLI
just adapter-cli

# Start server with e2e config
just e2e-serve
```

## e2e Testing

The `py-e2e-tests/` framework uses JSON-driven scenarios (no pytest required):

| Layer       | Command            | Coverage                                              |
| ----------- | ------------------ | ----------------------------------------------------- |
| **Basic**   | `just e2e-basic`   | Core functionality, both OpenAI + Anthropic endpoints |
| **Repair**  | `just e2e-repair`  | Tool call malformed format repair (OpenAI only)       |
| **Stress**  | `just e2e-stress`  | All scenarios × 3 iterations, safe concurrency + 1    |

Start the server first:

```bash
just e2e-serve
```

Then in another terminal, run the tests:

```bash
# Basic scenarios
just e2e-basic

# Tool call repair
just e2e-repair
```

Scenario files are organized under `scenarios/`:

```
py-e2e-tests/
├── scenarios/
│   ├── basic/
│   │   ├── openai/         # 10 scenarios (chat, reasoning, streaming, tools, files, images, HTTP links)
│   │   └── anthropic/      # 6 scenarios (chat, reasoning, tools, documents, images, HTTP links)
│   └── repair/             # 10 malformed tool call scenarios
├── runner.py               # Single-run entry
├── stress_runner.py        # Multi-iteration stress test entry
└── config.toml             # e2e server configuration
```

Each scenario is a standalone JSON file with request params and validation rules:

```json
{
  "name": "Scenario name",
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

### e2e CLI Arguments

**`just e2e-basic` / `just e2e-repair` (single run):**

| Argument | Description |
|----------|-------------|
| `scenario_dir` | Scenario directory, e.g. `scenarios/basic` |
| `--endpoint` | Filter by endpoint: `openai` / `anthropic` |
| `--model` | Filter by model: `deepseek-default` / `deepseek-expert` |
| `--filter` | Filter by scenario name keywords (space-separated) |
| `--parallel` | Parallelism, default `accounts ÷ 2` |
| `--show-output` | Show model response summary |
| `--report` | Output JSON report path |

**`just e2e-stress` (stress test):**

| Argument | Description |
|----------|-------------|
| `--iterations` | Iterations per scenario, default 3 |
| `--models` | Filter by model list |
| `--filter` | Filter by scenario name keywords |
| `--parallel` | Parallelism, default `accounts ÷ 2 + 1` |
| `--show-output` | Show model output |
| `--report` | Output JSON report path |

Examples:

```bash
# Quick verification of new file upload scenarios
just e2e-basic --filter file image --show-output

# Run OpenAI-only with expert model
just e2e-basic --endpoint openai --model deepseek-expert

# Serial debugging
just e2e-basic --endpoint openai --parallel 1 --show-output

# Stress test: repair scenarios × 5 iterations
just e2e-stress --filter repair --iterations 5

# Output JSON report
just e2e-basic --report result.json
```

## More Documentation

- [Code Style](code-style.md)
- [Logging Spec](logging-spec.md)
- [DeepSeek API Reference](deepseek-api-reference.md)
- [Prompt Injection Strategy](deepseek-prompt-injection.md)