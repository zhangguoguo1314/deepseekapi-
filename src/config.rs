//! 配置加载模块 —— 统一配置入口
//!
//! 支持 `-c <path>` 命令行参数，默认值见下方函数。
//! config.toml 中注释项使用代码默认值。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 应用配置根结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// DeepSeek 核心配置（账号、客户端、模型等）
    pub ds_core: DsCoreSection,
    /// HTTP 服务器配置（必填）
    pub server: ServerConfig,
    /// 代理配置（可选，用于绕过 WAF）
    #[serde(default)]
    pub proxy: ProxyConfig,
    /// Admin 配置（bcrypt 密码哈希、JWT 密钥等，由管理面板管理）
    #[serde(default)]
    pub admin: AdminConfig,
    /// API Key 列表（由管理面板管理）
    #[serde(default)]
    pub api_keys: Vec<ApiKeyEntry>,
}

/// DeepSeek 核心配置段 —— 对应 config.toml 的 [ds_core]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DsCoreSection {
    /// 账号池（必需，可为空——启动后通过管理面板添加）
    #[serde(default)]
    pub accounts: Vec<Account>,
    /// API 基础地址
    #[serde(default = "default_api_base")]
    pub api_base: String,
    /// WASM 文件完整 URL（PoW 计算所需，版本号可能变动）
    #[serde(default = "default_wasm_url")]
    pub wasm_url: String,
    /// User-Agent 请求头
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    /// X-Client-Version 请求头（用于 expert 模型等功能）
    #[serde(default = "default_client_version")]
    pub client_version: String,
    /// X-Client-Platform 请求头
    #[serde(default = "default_client_platform")]
    pub client_platform: String,
    /// X-Client-Locale 请求头
    #[serde(default = "default_client_locale")]
    pub client_locale: String,
    /// 定义支持的模型类型列表，每种类型会自动映射为 OpenAI 的 model_id：deepseek-<type>
    #[serde(default = "default_model_types")]
    pub model_types: Vec<String>,
    /// 各模型类型的输入 token 限制（与 model_types 按索引一一对应）
    #[serde(default = "default_max_input_tokens")]
    pub max_input_tokens: Vec<u32>,
    /// 各模型类型的输出 token 限制（与 model_types 按索引一一对应）
    #[serde(default = "default_max_output_tokens")]
    pub max_output_tokens: Vec<u32>,
    /// 各模型类型的单次输入字符数限制（与 model_types 按索引一一对应）
    #[serde(default = "default_input_character_limits")]
    pub input_character_limits: Vec<u32>,
    /// 模型别名：按 index 对齐 model_types，默认无别名
    #[serde(default)]
    pub model_aliases: Vec<String>,
    /// 工具调用标签配置（自定义回退标签）
    #[serde(default)]
    pub tool_call: ToolCallTagConfig,
}

impl DsCoreSection {
    /// 生成 OpenAI 模型注册表映射
    #[must_use]
    pub fn model_registry(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        for (i, ty) in self.model_types.iter().enumerate() {
            map.insert(format!("deepseek-{}", ty).to_lowercase(), ty.clone());
            if let Some(alias) = self.model_aliases.get(i) {
                let alias = alias.trim().to_lowercase();
                if !alias.is_empty() {
                    map.insert(alias, ty.clone());
                }
            }
        }
        map
    }
}

/// Admin 配置
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdminConfig {
    /// bcrypt 哈希后的密码
    #[serde(default)]
    pub password_hash: String,
    /// JWT 签名密钥（hex 编码的 32 字节随机值）
    #[serde(default)]
    pub jwt_secret: String,
    /// 最近一次 JWT 签发时间（用于吊销旧 token）
    #[serde(default)]
    pub jwt_issued_at: u64,
    /// 修改密码：旧密码明文（仅 PUT 接收，不落地 config.toml）
    #[serde(default, skip_serializing)]
    pub old_password: String,
    /// 修改密码：新密码明文（仅 PUT 接收，不落地 config.toml）
    #[serde(default, skip_serializing)]
    pub new_password: String,
}

/// API Key 条目
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiKeyEntry {
    pub key: String,
    pub description: String,
}

/// 单个账号配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    /// 邮箱（与 mobile 二选一）
    pub email: String,
    /// 手机号（与 email 二选一）
    pub mobile: String,
    /// 区号（与 mobile 配合使用，如 "+86"）
    pub area_code: String,
    /// 密码
    pub password: String,
}

/// 代理配置
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProxyConfig {
    /// 代理 URL，如 http://127.0.0.1:7890 或 socks5://127.0.0.1:7891
    pub url: Option<String>,
}

/// 工具调用标签配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallTagConfig {
    /// 额外开始标签（内置 `<|tool▁calls▁begin|>` + 模糊匹配，此处只加格式完全不同的变体）
    #[serde(default = "default_tool_call_starts")]
    pub extra_starts: Vec<String>,
    /// 额外结束标签（内置 `<|tool▁calls▁end|>` + 模糊匹配，此处只加格式完全不同的变体）
    #[serde(default = "default_tool_call_ends")]
    pub extra_ends: Vec<String>,
}

impl Default for ToolCallTagConfig {
    fn default() -> Self {
        Self {
            extra_starts: default_tool_call_starts(),
            extra_ends: default_tool_call_ends(),
        }
    }
}

// ── 默认值函数 ──────────────────────────────────────────────────────────

fn default_tool_call_starts() -> Vec<String> {
    vec![
        "<|tool_call_begin|>".into(),
        "<tool_calls>".into(),
        "<tool_call>".into(),
    ]
}

fn default_tool_call_ends() -> Vec<String> {
    vec![
        "<|tool_call_end|>".into(),
        "</tool_calls>".into(),
        "</tool_call>".into(),
    ]
}

fn default_model_types() -> Vec<String> {
    vec![
        "default".to_string(),
        "expert".to_string(),
        "vision".to_string(),
    ]
}

fn default_max_input_tokens() -> Vec<u32> {
    vec![1_048_576, 1_048_576, 1_048_576]
}

fn default_max_output_tokens() -> Vec<u32> {
    vec![384_000, 384_000, 384_000]
}

fn default_input_character_limits() -> Vec<u32> {
    vec![2_621_440, 163_840, 2_621_440]
}

fn default_api_base() -> String {
    "https://chat.deepseek.com/api/v0".to_string()
}

fn default_wasm_url() -> String {
    "https://fe-static.deepseek.com/chat/static/sha3_wasm_bg.7b9ca65ddd.wasm".to_string()
}

fn default_user_agent() -> String {
    "DeepSeek/2.1.1 Android/35".to_string()
}

fn default_client_version() -> String {
    "2.0.0".to_string()
}

fn default_client_platform() -> String {
    "android".to_string()
}

fn default_client_locale() -> String {
    "zh_CN".to_string()
}

/// HTTP 服务器配置（必填）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// CORS 允许的 Origin 列表，默认 ["http://localhost:22217"]
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:22217".to_string()]
}

// ── Config 实现 ─────────────────────────────────────────────────────────

impl Config {
    /// 从指定路径加载配置
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Self = toml::de::from_str(&content)?;
        config.dedup_accounts();
        config.validate()?;
        Ok(config)
    }

    /// 按 email（优先）或 mobile 去重，保留首次出现的账号
    fn dedup_accounts(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.ds_core.accounts.retain(|a| {
            let key = if a.email.is_empty() {
                a.mobile.clone()
            } else {
                a.email.clone()
            };
            seen.insert(key)
        });
    }

    /// 解析命令行参数并加载配置
    pub fn load_with_args(
        args: impl Iterator<Item = String>,
    ) -> Result<(Self, PathBuf), ConfigError> {
        let mut explicit_c = false;
        let mut config_path = None;
        let mut iter = args.skip(1);

        while let Some(arg) = iter.next() {
            if arg == "-c" {
                explicit_c = true;
                if let Some(path) = iter.next() {
                    config_path = Some(path);
                } else {
                    return Err(ConfigError::Cli("-c 参数需要指定路径".to_string()));
                }
            }
        }

        let path: PathBuf = config_path
            .map(PathBuf::from)
            .or_else(|| std::env::var("DS_CONFIG_PATH").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("config.toml"));

        if !path.exists() {
            if explicit_c {
                return Err(ConfigError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("指定配置文件不存在: {}", path.display()),
                )));
            }
            let default = Config {
                ds_core: DsCoreSection {
                    accounts: Vec::new(),
                    ..Default::default()
                },
                server: ServerConfig {
                    host: "0.0.0.0".into(),
                    port: 7860,
                    cors_origins: vec!["*".to_string()],
                },
                proxy: ProxyConfig::default(),
                admin: AdminConfig::default(),
                api_keys: Vec::new(),
            };
            if let Some(parent) = path.parent() {
                let parent_str = parent.as_os_str();
                if !parent_str.is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            default.save(&path)?;
            log::info!(target: "config", "已创建默认配置文件: {}", path.display());
            return Ok((default, path));
        }

        let config = Self::load(&path)?;
        Ok((config, path))
    }

    /// 验证配置有效性
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        if self.ds_core.model_types.is_empty() {
            return Err(ConfigError::Validation("model_types 不能为空".to_string()));
        }
        let n = self.ds_core.model_types.len();
        if self.ds_core.max_input_tokens.len() != n {
            return Err(ConfigError::Validation(format!(
                "max_input_tokens 长度({})必须与 model_types 长度({})一致",
                self.ds_core.max_input_tokens.len(),
                n
            )));
        }
        if self.ds_core.max_output_tokens.len() != n {
            return Err(ConfigError::Validation(format!(
                "max_output_tokens 长度({})必须与 model_types 长度({})一致",
                self.ds_core.max_output_tokens.len(),
                n
            )));
        }
        if self.ds_core.input_character_limits.len() != n {
            return Err(ConfigError::Validation(format!(
                "input_character_limits 长度({})必须与 model_types 长度({})一致",
                self.ds_core.input_character_limits.len(),
                n
            )));
        }
        let mut seen_keys = std::collections::HashSet::new();
        for k in &self.api_keys {
            if !seen_keys.insert(&k.key) {
                let prefix = if k.key.len() > 12 {
                    &k.key[..12]
                } else {
                    &k.key
                };
                return Err(ConfigError::Validation(format!(
                    "API key 重复: {}...",
                    prefix
                )));
            }
        }
        Ok(())
    }

    /// 原子保存配置到文件
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let toml_str = toml::to_string_pretty(self).map_err(ConfigError::TomlSerialization)?;
        let tmp = path.as_ref().with_extension("toml.tmp");
        std::fs::write(&tmp, &toml_str)?;
        std::fs::rename(&tmp, path.as_ref())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path.as_ref(), perms)?;
        }
        Ok(())
    }
}

impl Default for DsCoreSection {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            api_base: default_api_base(),
            wasm_url: default_wasm_url(),
            user_agent: default_user_agent(),
            client_version: default_client_version(),
            client_platform: default_client_platform(),
            client_locale: default_client_locale(),
            model_types: default_model_types(),
            max_input_tokens: default_max_input_tokens(),
            max_output_tokens: default_max_output_tokens(),
            input_character_limits: default_input_character_limits(),
            model_aliases: Vec::new(),
            tool_call: ToolCallTagConfig::default(),
        }
    }
}

/// 配置加载错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML 解析错误: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("配置验证错误: {0}")]
    Validation(String),
    #[error("命令行参数错误: {0}")]
    Cli(String),
    #[error("TOML 序列化错误: {0}")]
    TomlSerialization(#[from] toml::ser::Error),
}
