mod handlers;
mod middleware;
mod models;
mod utils;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载环境变量
    dotenv().ok();
    env_logger::init();

    // 获取配置
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    log::info!("正在连接数据库...");
    
    // 创建数据库连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // 执行数据库迁移
    log::info!("正在初始化数据库...");
    let migration_sql = include_str!("../migrations/init.sql");
    sqlx::query(migration_sql)
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    log::info!("游戏服务器启动在 http://{}:{}", host, port);

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        // CORS 配置
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            // 健康检查端点
            .route("/health", web::get().to(health_check))
            // 公开的游戏路由 - 放在认证路由之前
            .route("/api/leaderboard", web::get().to(handlers::get_leaderboard))
            // 认证路由（无需token）
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::register))
                    .route("/login", web::post().to(handlers::login))
            )
            // 需要认证的路由
            .service(
                web::scope("/api")
                    .wrap(HttpAuthentication::bearer(middleware::validator))
                    // 用户相关
                    .route("/me", web::get().to(handlers::get_current_user))
                    // 游戏相关
                    .service(
                        web::scope("/game")
                            .route("/session", web::post().to(handlers::create_game_session))
                            .route("/session/{id}/score", web::put().to(handlers::update_game_score))
                            .route("/session/{id}/end", web::post().to(handlers::end_game_session))
                    )
            )
    })
    .bind((host, port))?
    .run()
    .await
}

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "game_server",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
