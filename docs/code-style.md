# 代码风格规范

## 注释风格

### 模块文档（//!）
- 第一行：模块职责 —— 具体描述
- 空行后：关键设计决策或限制

```rust
//! 账号池管理 —— 多账号负载均衡
//!
//! 1 account = 1 session = 1 concurrency
```

### 公有 API 文档（///）
- 使用动词开头："返回"、"创建"、"发送"
- 明确副作用："自动释放"、"清理 session"
- 标注 Panic 条件（如有）

```rust
/// 轮询获取一个空闲账号
///
/// 返回的 AccountGuard 在 Drop 时自动释放 busy 标记
pub fn get_account(&self) -> Option<AccountGuard>
```

### 行内注释（//）
- 解释"为什么"而非"做什么"
- 标注临时方案或外部依赖

```rust
// 顺序很重要：health_check 必须在 update_title 之前，
// 否则空 session 会导致 EMPTY_CHAT_SESSION 错误
```

## 命名规范

| 类型 | 风格 | 示例 |
|------|------|------|
| 模块/文件 | snake_case | `ds_core`, `accounts.rs` |
| 类型/结构体 | PascalCase | `AccountPool`, `CoreError` |
| 函数/方法 | snake_case | `get_account()`, `compute_pow()` |
| 常量 | SCREAMING_SNAKE_CASE | `ENDPOINT_USERS_LOGIN` |
| 枚举变体 | PascalCase | `AllAccountsFailed` |

## 错误消息

- **中文**：配置验证、账号管理等面向用户的错误消息使用中文
- **英文**：内部库错误（`ds_core`、`client`、`adapter`、`anthropic_compat`）使用英文，供开发者调试
- 包含上下文："账号 {} 初始化失败"
- 避免泄露敏感信息（token 只打印前8位）
- 服务器层的 `ServerError::Display` 向 API 客户端展示错误时，保持适配器原始消息不变

## 枚举变体命名

- 所有枚举变体使用 PascalCase（如 `AllAccountsFailed`、`BadRequest`）
- 仅在 serde 序列化时通过 `#[serde(rename = "...")]` 使用非 PascalCase

## 日志规范

见 `docs/logging-spec.md`

## 导入分组

1. 标准库 (`std::`)
2. 第三方库 (`tokio::`, `wreq::`)
3. 内部模块 (`crate::`)
4. 本地 use (super, self)

组间空行分隔。

## 测试代码规范

- 测试函数内部允许使用 `println!` 输出中间结果，便于失败时观测解析内容
- 库代码（`src/` 非 `#[cfg(test)]` 区域）仍禁止直接使用 `println!` / `eprintln!`
