# Droid Provider

Droid Provider 是 ProxyCast 的 Factory.ai Droid 平台插件，支持 WorkOS OAuth 和 API Key 两种认证方式。

## 支持的认证方式

| 认证方式 | 说明 | 适用场景 |
|---------|------|---------|
| **WorkOS OAuth** | 使用 WorkOS OAuth 授权 | Factory.ai 账户登录 |
| **API Key** | 使用 Factory.ai API Key | 直接 API 访问 |

## 支持的端点

| 端点类型 | 路径 | 说明 |
|---------|------|------|
| **Anthropic** | `/a/v1/messages` | Claude Messages API |
| **OpenAI** | `/o/v1/responses` | OpenAI Responses API |
| **Comm** | `/o/v1/chat/completions` | Chat Completions API |

## 支持的模型

- `claude-opus-4-1-20250805` - Claude Opus 4.1
- `claude-sonnet-4-5-20250929` - Claude Sonnet 4.5
- `claude-sonnet-4-20250514` - Claude Sonnet 4
- `gpt-5-2025-08-07` - GPT-5

## 开发

### 前端开发

```bash
# 安装依赖
npm install

# 开发模式
npm run dev

# 构建
npm run build
```

### 后端开发

```bash
cd src-tauri

# 构建
cargo build --release

# 运行
cargo run -- --help
```

## 项目结构

```
droid-provider/
├── plugin/
│   ├── plugin.json          # 插件元数据
│   ├── config.json          # 默认配置
│   └── dist/                # 前端构建输出
├── src/                     # 前端 React UI
│   ├── index.tsx
│   ├── App.tsx
│   └── components/
├── src-tauri/src/           # 后端 Rust 代码
│   ├── main.rs              # CLI 入口
│   ├── provider.rs          # 核心实现
│   ├── credentials.rs       # 凭证数据结构
│   ├── token_refresh.rs     # Token 刷新
│   └── auth/                # 认证模块
│       ├── workos.rs        # WorkOS OAuth
│       └── encryption.rs    # API Key 加密
└── package.json
```

## License

MIT
