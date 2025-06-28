# Saber - Web Game Server

一个使用 Rust 和 Actix-web 构建的 Web 游戏服务器项目。

A web game server project built with Rust and Actix-web.

## 项目状态 / Project Status

🚧 **开发中 / Under Development**

查看 [PROGRESS.md](./PROGRESS.md) 了解详细的项目进度和计划。

See [PROGRESS.md](./PROGRESS.md) for detailed project progress and plans.

## 技术栈 / Tech Stack

- **语言 / Language**: Rust
- **Web框架 / Web Framework**: Actix-web
- **数据库 / Database**: PostgreSQL
- **ORM**: SQLx
- **部署平台 / Deployment**: Shuttle

## 快速开始 / Quick Start

### 本地开发 / Local Development

1. 克隆项目 / Clone the project
```bash
git clone <repository-url>
cd saber
```

2. 设置数据库 / Set up database
```bash
# 创建PostgreSQL数据库并运行schema.sql
```

3. 运行服务器 / Run the server
```bash
cargo run
```

服务器将在 `http://127.0.0.1:8080` 启动。

The server will start at `http://127.0.0.1:8080`.

## API 端点 / API Endpoints

当前可用的端点 / Currently available endpoints:

- `GET /{id}` - 获取待办事项 / Get a todo item
- `POST /add` - 添加待办事项 / Add a todo item

## 贡献 / Contributing

欢迎贡献！请查看 [PROGRESS.md](./PROGRESS.md) 了解当前需要帮助的任务。

Contributions are welcome! Please check [PROGRESS.md](./PROGRESS.md) for tasks that need help.

## 许可证 / License

待定 / TBD
