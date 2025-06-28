# 第一阶段：构建阶段
FROM rust:1.75-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 创建工作目录
WORKDIR /app

# 复制Cargo文件
COPY Cargo.toml Cargo.lock ./

# 创建src目录和dummy main.rs来缓存依赖
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 构建项目
RUN touch src/main.rs && \
    cargo build --release

# 第二阶段：运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非root用户
RUN useradd -m -u 1001 -s /bin/bash saber

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/saber /app/saber

# 更改所有权
RUN chown -R saber:saber /app

# 切换到非root用户
USER saber

# 暴露端口
EXPOSE 8000

# 运行程序
CMD ["./saber"]