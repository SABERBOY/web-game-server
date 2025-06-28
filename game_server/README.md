# Rust 游戏服务器

一个使用 Rust 构建的游戏服务器，包含用户认证、JWT token 验证和基本的游戏功能。

## 功能特性

- 用户注册和登录
- JWT token 认证
- 密码安全哈希存储（使用 Argon2）
- 游戏会话管理
- 分数记录和排行榜
- RESTful API 设计
- SQLite 数据库

## 技术栈

- **Actix-web**: 高性能 Web 框架
- **SQLx**: 异步 SQL 工具包
- **JWT**: JSON Web Token 认证
- **Argon2**: 密码哈希
- **SQLite**: 轻量级数据库

## 快速开始

### 环境要求

- Rust 1.70+
- SQLite

### 安装和运行

1. 克隆项目
```bash
cd /workspace/game_server
```

2. 配置环境变量（可选，已有默认值）
```bash
# 编辑 .env 文件
```

3. 构建并运行
```bash
cargo build --release
cargo run
```

服务器将在 `http://127.0.0.1:8080` 启动

## API 文档

### 认证相关

#### 注册
- **POST** `/api/auth/register`
- **Body**:
```json
{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}
```
- **Response**:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "created_at": "2024-01-01T00:00:00Z",
    "last_login": null
  }
}
```

#### 登录
- **POST** `/api/auth/login`
- **Body**:
```json
{
  "username_or_email": "testuser",
  "password": "password123"
}
```
- **Response**: 同注册接口

### 用户相关（需要认证）

#### 获取当前用户信息
- **GET** `/api/me`
- **Headers**: `Authorization: Bearer <token>`
- **Response**:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "testuser",
  "email": "test@example.com",
  "created_at": "2024-01-01T00:00:00Z",
  "last_login": "2024-01-01T12:00:00Z"
}
```

### 游戏相关

#### 创建游戏会话（需要认证）
- **POST** `/api/game/session`
- **Headers**: `Authorization: Bearer <token>`
- **Body**:
```json
{
  "game_type": "puzzle"
}
```
- **Response**:
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "start_time": "2024-01-01T12:00:00Z",
  "end_time": null,
  "score": 0,
  "status": "active"
}
```

#### 更新游戏分数（需要认证）
- **PUT** `/api/game/session/{session_id}/score`
- **Headers**: `Authorization: Bearer <token>`
- **Body**:
```json
{
  "score": 1000
}
```

#### 结束游戏会话（需要认证）
- **POST** `/api/game/session/{session_id}/end`
- **Headers**: `Authorization: Bearer <token>`

#### 获取排行榜（公开）
- **GET** `/api/leaderboard`
- **Response**:
```json
[
  {
    "username": "player1",
    "score": 5000,
    "completed_at": "2024-01-01T12:30:00Z"
  },
  {
    "username": "player2",
    "score": 4500,
    "completed_at": "2024-01-01T11:45:00Z"
  }
]
```

### 系统相关

#### 健康检查
- **GET** `/health`
- **Response**:
```json
{
  "status": "ok",
  "service": "game_server",
  "version": "0.1.0"
}
```

## 项目结构

```
game_server/
├── src/
│   ├── main.rs           # 主程序入口
│   ├── models/           # 数据模型
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── handlers/         # API 处理器
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   └── game.rs
│   ├── middleware/       # 中间件
│   │   ├── mod.rs
│   │   └── auth.rs
│   └── utils/            # 工具函数
│       ├── mod.rs
│       ├── jwt.rs
│       └── password.rs
├── migrations/           # 数据库迁移
│   └── init.sql
├── Cargo.toml           # 项目依赖
├── .env                 # 环境变量
└── README.md            # 项目说明
```

## 安全注意事项

1. 在生产环境中，请务必更改 `.env` 文件中的 `JWT_SECRET`
2. 使用 HTTPS 来保护 API 通信
3. 实施请求限流防止暴力破解
4. 定期更新依赖项以修复安全漏洞

## 扩展建议

1. 添加更多游戏类型和功能
2. 实现 WebSocket 支持实时游戏
3. 添加好友系统和社交功能
4. 实现游戏内物品和货币系统
5. 添加数据分析和监控
6. 支持多种数据库（PostgreSQL、MySQL等）

## 许可证

MIT License