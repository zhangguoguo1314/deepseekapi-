---
title: DS-Free-API
emoji: 🐋
colorFrom: blue
colorTo: purple
sdk: docker
app_port: 7860
---

<p align="center">
  <img src="https://raw.githubusercontent.com/NIyueeE/ds-free-api/main/assets/logo.svg" width="81" height="66">
</p>

<h1 align="center">DS-Free-API - Hugging Face Spaces 版</h1>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/NIyueeE/ds-free-api.svg"></a>
  <img src="https://img.shields.io/github/v/release/NIyueeE/ds-free-api.svg">
  <img src="https://img.shields.io/badge/rust-1.95.0+-93450a.svg">
</p>

将免费的 DeepSeek 网页端对话反代并适配转换为标准的 OpenAI 与 Anthropic 兼容 API 协议。

## 快速部署到 Hugging Face

1. 点击上方 **Duplicate this Space** 按钮复制此项目
2. 在 Space 设置中的 **Variables and Secrets** 添加环境变量（可选，也可在管理面板配置）：
   - `DS_ADMIN_PASSWORD`：管理面板密码（首次访问时设置）
3. 等待构建完成后访问 `/admin` 路径

## 配置 DeepSeek 账号

### 方式一：通过管理面板（推荐）

1. 访问 `https://你的用户名-仓库名.hf.space/admin`
2. 首次访问设置管理密码
3. 进入 **账号池** 页面，点击 **添加账号**
4. 填写 DeepSeek 账号信息：
   - **邮箱**：你的 DeepSeek 账号邮箱
   - **密码**：DeepSeek 账号密码
   - 或 **手机号** + **区号** + **密码**

### 方式二：通过环境变量预配置

在 HF Space 的 **Variables and Secrets** 中添加：

```
DS_ACCOUNTS=[{"email":"your@email.com","password":"yourpassword"}]
```

## 获取 API Key

1. 登录管理面板 `/admin`
2. 进入 **API Keys** 页面
3. 点击 **创建 API Key**
4. 复制生成的 Key 用于调用 API

## API 使用示例

### OpenAI 格式

```python
from openai import OpenAI

client = OpenAI(
    api_key="你的API Key",
    base_url="https://你的用户名-仓库名.hf.space/v1"
)

response = client.chat.completions.create(
    model="deepseek-default",
    messages=[{"role": "user", "content": "你好"}]
)
print(response.choices[0].message.content)
```

### Anthropic 格式

```python
import anthropic

client = anthropic.Anthropic(
    api_key="你的API Key",
    base_url="https://你的用户名-仓库名.hf.space/anthropic"
)

message = client.messages.create(
    model="deepseek-default",
    max_tokens=1024,
    messages=[{"role": "user", "content": "你好"}]
)
print(message.content)
```

## 支持的模型

| 模型 ID | 说明 |
|---------|------|
| `deepseek-default` | 快速模式 |
| `deepseek-expert` | 专家模式（深度思考） |
| `deepseek-vision` | 视觉模式 |

## 功能特性

- ✅ OpenAI Chat Completions API 兼容
- ✅ Anthropic Messages API 兼容
- ✅ 流式响应（SSE）
- ✅ 工具调用（Function Calling）
- ✅ 文件上传（图片、文档）
- ✅ 多账号池自动轮询
- ✅ Web 管理面板
- ✅ 请求日志查看

## 注意事项

- Hugging Face Spaces 免费版 48 小时无活动后会休眠
- 首次访问可能需要等待服务唤醒
- 管理面板密码请妥善保管，丢失后需重新部署

## 原始项目

[https://github.com/NIyueeE/ds-free-api](https://github.com/NIyueeE/ds-free-api)
