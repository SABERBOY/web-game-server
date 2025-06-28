use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use validator::Validate;

/// 数据库中的用户模型
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: i64,  // 时间戳
    pub updated_at: i64,  // 时间戳
    pub last_login: Option<i64>,  // 时间戳
    pub is_active: bool,
}

/// 用户注册请求
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 20))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 6))]
    pub password: String,
}

/// 用户登录请求
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

/// 认证响应
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

/// 用户信息响应（不包含敏感信息）
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        use chrono::TimeZone;
        
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: Utc.timestamp_opt(user.created_at, 0).unwrap(),
            last_login: user.last_login.map(|ts| Utc.timestamp_opt(ts, 0).unwrap()),
        }
    }
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}