use crate::middleware::AuthUser;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, FromRow};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GameSession {
    pub id: String,
    pub user_id: String,
    pub start_time: i64,  // 时间戳
    pub end_time: Option<i64>,  // 时间戳
    pub score: i32,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    pub game_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScoreRequest {
    pub score: i32,
}

/// 创建新游戏会话
pub async fn create_game_session(
    pool: web::Data<SqlitePool>,
    auth_user: AuthUser,
    req: web::Json<CreateGameRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    
    // 插入游戏会话
    sqlx::query(
        r#"
        INSERT INTO game_sessions (id, user_id, start_time, score, status, game_type)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&session_id)
    .bind(&auth_user.user_id)
    .bind(now)
    .bind(0)
    .bind("active")
    .bind(&req.game_type)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 查询刚创建的会话
    let session = sqlx::query_as::<_, GameSession>(
        "SELECT id, user_id, start_time, end_time, score, status FROM game_sessions WHERE id = ?"
    )
    .bind(&session_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Created().json(session))
}

/// 更新游戏分数
pub async fn update_game_score(
    pool: web::Data<SqlitePool>,
    auth_user: AuthUser,
    session_id: web::Path<String>,
    req: web::Json<UpdateScoreRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // 验证游戏会话属于当前用户
    #[derive(FromRow)]
    struct SessionCheck {
        id: String,
    }
    
    let session = sqlx::query_as::<_, SessionCheck>(
        "SELECT id FROM game_sessions WHERE id = ? AND user_id = ? AND status = 'active'"
    )
    .bind(session_id.as_str())
    .bind(&auth_user.user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if session.is_none() {
        return Err(actix_web::error::ErrorNotFound("游戏会话不存在或已结束"));
    }

    // 更新分数
    sqlx::query("UPDATE game_sessions SET score = ? WHERE id = ?")
        .bind(req.score)
        .bind(session_id.as_str())
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "分数更新成功",
        "score": req.score
    })))
}

/// 结束游戏会话
pub async fn end_game_session(
    pool: web::Data<SqlitePool>,
    auth_user: AuthUser,
    session_id: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let now = Utc::now().timestamp();
    
    let result = sqlx::query(
        "UPDATE game_sessions SET end_time = ?, status = 'completed' WHERE id = ? AND user_id = ? AND status = 'active'"
    )
    .bind(now)
    .bind(session_id.as_str())
    .bind(&auth_user.user_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("游戏会话不存在或已结束"));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "游戏会话已结束"
    })))
}

/// 获取排行榜
pub async fn get_leaderboard(
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse, actix_web::Error> {
    #[derive(FromRow)]
    struct LeaderboardEntry {
        username: String,
        score: i32,
        end_time: Option<i64>,
    }
    
    let leaderboard = sqlx::query_as::<_, LeaderboardEntry>(
        r#"
        SELECT u.username, gs.score, gs.end_time
        FROM game_sessions gs
        JOIN users u ON gs.user_id = u.id
        WHERE gs.status = 'completed'
        ORDER BY gs.score DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let result: Vec<_> = leaderboard
        .into_iter()
        .map(|row| {
            use chrono::TimeZone;
            serde_json::json!({
                "username": row.username,
                "score": row.score,
                "completed_at": row.end_time.map(|ts| Utc.timestamp_opt(ts, 0).unwrap())
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}