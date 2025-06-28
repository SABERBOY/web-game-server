use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse};
use crate::utils::{JwtUtil, PasswordUtil};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

/// 用户注册
pub async fn register(
    pool: web::Data<SqlitePool>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // 验证请求数据
    req.validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;

    // 检查用户名是否已存在
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ? OR email = ?"
    )
    .bind(&req.username)
    .bind(&req.email)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if existing_user.is_some() {
        return Err(actix_web::error::ErrorConflict("用户名或邮箱已存在"));
    }

    // 创建新用户
    let user_id = Uuid::new_v4().to_string();
    let password_hash = PasswordUtil::hash_password(&req.password)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let now = Utc::now();
    
    // 插入新用户
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&user_id)
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(now.timestamp())
    .bind(now.timestamp())
    .bind(true)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 查询刚创建的用户
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = ?"
    )
    .bind(&user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 生成 JWT token
    let token = JwtUtil::generate_token(&user.id, &user.username)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let response = AuthResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok(HttpResponse::Created().json(response))
}

/// 用户登录
pub async fn login(
    pool: web::Data<SqlitePool>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // 验证请求数据
    req.validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;

    // 查找用户
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ? OR email = ?"
    )
    .bind(&req.username_or_email)
    .bind(&req.username_or_email)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let user = match user {
        Some(u) => u,
        None => return Err(actix_web::error::ErrorUnauthorized("用户名或密码错误")),
    };

    // 验证密码
    let is_valid = PasswordUtil::verify_password(&req.password, &user.password_hash)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if !is_valid {
        return Err(actix_web::error::ErrorUnauthorized("用户名或密码错误"));
    }

    // 检查用户是否激活
    if !user.is_active {
        return Err(actix_web::error::ErrorForbidden("账户已被禁用"));
    }

    // 更新最后登录时间
    sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
        .bind(Utc::now().timestamp())
        .bind(&user.id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 生成 JWT token
    let token = JwtUtil::generate_token(&user.id, &user.username)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let response = AuthResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// 获取当前用户信息（需要认证）
pub async fn get_current_user(
    pool: web::Data<SqlitePool>,
    auth_user: crate::middleware::AuthUser,
) -> Result<HttpResponse, actix_web::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = ?"
    )
    .bind(&auth_user.user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match user {
        Some(u) => Ok(HttpResponse::Ok().json(UserResponse::from(u))),
        None => Err(actix_web::error::ErrorNotFound("用户不存在")),
    }
}