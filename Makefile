# 项目信息
PROJECT_NAME = saber
DOCKER_REGISTRY ?= docker.io
DOCKER_USERNAME ?= yourusername
DOCKER_IMAGE = $(DOCKER_REGISTRY)/$(DOCKER_USERNAME)/$(PROJECT_NAME)
VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")

# 默认目标
.PHONY: help
help:
	@echo "Saber Game Server - Makefile Commands"
	@echo ""
	@echo "Development:"
	@echo "  make build         - 构建Rust项目"
	@echo "  make run           - 运行本地服务器"
	@echo "  make test          - 运行测试"
	@echo "  make fmt           - 格式化代码"
	@echo "  make lint          - 运行代码检查"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build  - 构建Docker镜像"
	@echo "  make docker-run    - 运行Docker容器"
	@echo "  make docker-push   - 推送镜像到Docker Hub"
	@echo "  make docker-all    - 构建并推送镜像"
	@echo ""
	@echo "Docker Compose:"
	@echo "  make compose-up    - 启动所有服务"
	@echo "  make compose-down  - 停止所有服务"
	@echo "  make compose-logs  - 查看服务日志"
	@echo ""
	@echo "Database:"
	@echo "  make db-init       - 初始化数据库"
	@echo "  make db-migrate    - 运行数据库迁移"

# 开发命令
.PHONY: build
build:
	cargo build --release

.PHONY: run
run:
	cargo run

.PHONY: test
test:
	cargo test

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: lint
lint:
	cargo clippy -- -D warnings

# Docker命令
.PHONY: docker-build
docker-build:
	@echo "构建Docker镜像: $(DOCKER_IMAGE):$(VERSION)"
	docker build -t $(DOCKER_IMAGE):$(VERSION) -t $(DOCKER_IMAGE):latest .

.PHONY: docker-run
docker-run:
	@echo "运行Docker容器"
	docker run -d \
		--name $(PROJECT_NAME) \
		-p 8000:8000 \
		--env-file .env \
		$(DOCKER_IMAGE):latest

.PHONY: docker-push
docker-push:
	@echo "推送Docker镜像到仓库"
	docker push $(DOCKER_IMAGE):$(VERSION)
	docker push $(DOCKER_IMAGE):latest

.PHONY: docker-all
docker-all: docker-build docker-push

.PHONY: docker-clean
docker-clean:
	@echo "清理Docker容器和镜像"
	docker stop $(PROJECT_NAME) || true
	docker rm $(PROJECT_NAME) || true
	docker rmi $(DOCKER_IMAGE):$(VERSION) || true
	docker rmi $(DOCKER_IMAGE):latest || true

# Docker Compose命令
.PHONY: compose-up
compose-up:
	docker-compose up -d

.PHONY: compose-down
compose-down:
	docker-compose down

.PHONY: compose-logs
compose-logs:
	docker-compose logs -f

.PHONY: compose-build
compose-build:
	docker-compose build

.PHONY: compose-restart
compose-restart: compose-down compose-up

# 数据库命令
.PHONY: db-init
db-init:
	@echo "初始化数据库"
	docker-compose exec postgres psql -U postgres -c "CREATE DATABASE saber_game;"
	docker-compose exec postgres psql -U postgres -d saber_game -f /docker-entrypoint-initdb.d/schema.sql

.PHONY: db-migrate
db-migrate:
	@echo "运行数据库迁移"
	sqlx migrate run

# 清理命令
.PHONY: clean
clean:
	cargo clean
	rm -rf target/

# 检查环境变量
.PHONY: check-env
check-env:
	@test -f .env || (echo "错误: 缺少.env文件" && exit 1)
	@echo "环境变量检查通过"