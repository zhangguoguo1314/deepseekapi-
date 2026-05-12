# Code Style Guide

## Comment Style

### Module Documentation (//!)
- First line: module responsibility — specific description
- After blank line: key design decisions or constraints

```rust
//! 账号池管理 —— 多账号负载均衡
//!
//! 1 account = 1 session = 1 concurrency
```

### Public API Documentation (///)
- Start with verbs: "Returns", "Creates", "Sends"
- Make side effects explicit: "Automatically releases", "Cleans up session"
- Document Panic conditions (if any)

```rust
/// 轮询获取一个空闲账号
///
/// 返回的 AccountGuard 在 Drop 时自动释放 busy 标记
pub fn get_account(&self) -> Option<AccountGuard>
```

### Inline Comments (//)
- Explain "why", not "what"
- Annotate temporary workarounds or external dependencies

```rust
// 顺序很重要：health_check 必须在 update_title 之前，
// 否则空 session 会导致 EMPTY_CHAT_SESSION 错误
```

## Naming Conventions

| Type | Style | Example |
|------|-------|---------|
| Module/File | snake_case | `ds_core`, `accounts.rs` |
| Type/Struct | PascalCase | `AccountPool`, `CoreError` |
| Function/Method | snake_case | `get_account()`, `compute_pow()` |
| Constant | SCREAMING_SNAKE_CASE | `ENDPOINT_USERS_LOGIN` |
| Enum Variant | PascalCase | `AllAccountsFailed` |

## Error Messages

- **Chinese**: user-facing errors like config validation and account management use Chinese
- **English**: internal library errors (`ds_core`, `client`, `adapter`, `anthropic_compat`) use English for developer debugging
- Include context: "Account {} initialization failed"
- Avoid leaking sensitive information (print only the first 8 characters of tokens)
- When `ServerError::Display` at the server layer presents errors to API clients, keep the original adapter message unchanged

## Enum Variant Naming

- All enum variants use PascalCase (e.g. `AllAccountsFailed`, `BadRequest`)
- Only use non-PascalCase via `#[serde(rename = "...")]` for serde serialization

## Logging Conventions

See `docs/logging-spec.md`

## Import Grouping

1. Standard library (`std::`)
2. Third-party libraries (`tokio::`, `wreq::`)
3. Internal modules (`crate::`)
4. Local uses (`super`, `self`)

Separate groups with blank lines.

## Test Code Conventions

- Within test functions, `println!` is allowed for intermediate output to aid debugging when tests fail
- Library code (`src/` outside `#[cfg(test)]`) must still not use `println!` / `eprintln!` directly
