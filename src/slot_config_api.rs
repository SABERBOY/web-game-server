use crate::universal_slots::{Payline, SlotConfig, SlotConfigBuilder, SlotSymbol, SymbolType};
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSlotConfigRequest {
    pub name: String,
    pub rows: i32,
    pub reels: i32,
    pub is_megaway: bool,
    pub min_megaway_rows: i32,
    pub max_megaway_rows: i32,
    pub default_bet: i32,
    pub min_bet: i32,
    pub max_bet: i32,
    pub wild_enabled: bool,
    pub free_spins_enabled: bool,
    pub rtp_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSymbolRequest {
    pub slot_config_id: i32,
    pub name: String,
    pub symbol_type: String,
    pub value: i32,
    pub image_url: Option<String>,
    pub payouts: HashMap<i32, i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReelSymbolRequest {
    pub slot_config_id: i32,
    pub reel_number: i32,
    pub position: i32,
    pub symbol_id: i32,
    pub weight: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaylineRequest {
    pub slot_config_id: i32,
    pub line_number: i32,
    pub pattern: Vec<(i32, i32)>,
    pub is_active: bool,
}

// 创建新的slot配置
pub async fn create_slot_config(
    pool: web::Data<Pool<Postgres>>,
    req: web::Json<CreateSlotConfigRequest>,
) -> Result<HttpResponse> {
    let result = sqlx::query!(
        r#"
        INSERT INTO slot_configurations 
        (name, rows, reels, is_megaway, min_megaway_rows, max_megaway_rows,
         default_bet, min_bet, max_bet, wild_enabled, free_spins_enabled, rtp_percentage)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id
        "#,
        req.name,
        req.rows,
        req.reels,
        req.is_megaway,
        req.min_megaway_rows,
        req.max_megaway_rows,
        req.default_bet,
        req.min_bet,
        req.max_bet,
        req.wild_enabled,
        req.free_spins_enabled,
        req.rtp_percentage as f64
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(rec) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "id": rec.id,
            "message": "Slot configuration created successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create slot configuration: {}", e)
        }))),
    }
}

// 获取所有slot配置列表
pub async fn list_slot_configs(pool: web::Data<Pool<Postgres>>) -> Result<HttpResponse> {
    let configs = sqlx::query!(
        "SELECT * FROM slot_configurations WHERE is_active = true ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await;

    match configs {
        Ok(configs) => {
            let serializable_configs: Vec<serde_json::Value> = configs
                .into_iter()
                .map(|config| {
                    serde_json::json!({
                        "id": config.id,
                        "name": config.name,
                        "rows": config.rows,
                        "reels": config.reels,
                        "is_megaway": config.is_megaway,
                        "min_megaway_rows": config.min_megaway_rows,
                        "max_megaway_rows": config.max_megaway_rows,
                        "default_bet": config.default_bet,
                        "min_bet": config.min_bet,
                        "max_bet": config.max_bet,
                        "wild_enabled": config.wild_enabled,
                        "free_spins_enabled": config.free_spins_enabled,
                        "rtp_percentage": config.rtp_percentage.map(|r| r.to_string()),
                        "is_active": config.is_active,
                        "created_at": config.created_at.map(|t| t.to_string()),
                        "updated_at": config.updated_at.map(|t| t.to_string())
                    })
                })
                .collect();
            Ok(HttpResponse::Ok().json(serializable_configs))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch slot configurations: {}", e)
        }))),
    }
}

// 获取单个slot配置详情
pub async fn get_slot_config(
    pool: web::Data<Pool<Postgres>>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let config_id = path.into_inner();

    let config = sqlx::query!("SELECT * FROM slot_configurations WHERE id = $1", config_id)
        .fetch_one(pool.get_ref())
        .await;

    match config {
        Ok(config) => {
            let serializable_config = serde_json::json!({
                "id": config.id,
                "name": config.name,
                "rows": config.rows,
                "reels": config.reels,
                "is_megaway": config.is_megaway,
                "min_megaway_rows": config.min_megaway_rows,
                "max_megaway_rows": config.max_megaway_rows,
                "default_bet": config.default_bet,
                "min_bet": config.min_bet,
                "max_bet": config.max_bet,
                "wild_enabled": config.wild_enabled,
                "free_spins_enabled": config.free_spins_enabled,
                "rtp_percentage": config.rtp_percentage.map(|r| r.to_string()),
                "is_active": config.is_active,
                "created_at": config.created_at.map(|t| t.to_string()),
                "updated_at": config.updated_at.map(|t| t.to_string())
            });
            Ok(HttpResponse::Ok().json(serializable_config))
        }
        Err(e) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Slot configuration not found: {}", e)
        }))),
    }
}

// 添加符号到slot配置
pub async fn add_symbol(
    pool: web::Data<Pool<Postgres>>,
    req: web::Json<CreateSymbolRequest>,
) -> Result<HttpResponse> {
    let symbol_id = sqlx::query!(
        r#"
        INSERT INTO slot_symbols 
        (slot_config_id, name, symbol_type, value, image_url)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        req.slot_config_id,
        req.name,
        req.symbol_type,
        req.value,
        req.image_url
    )
    .fetch_one(pool.get_ref())
    .await;

    match symbol_id {
        Ok(rec) => {
            // 添加赔付表
            for (count, payout) in &req.payouts {
                let column = match count {
                    2 => "payout_2x",
                    3 => "payout_3x",
                    4 => "payout_4x",
                    5 => "payout_5x",
                    6 => "payout_6x",
                    _ => continue,
                };

                let query = format!(
                    "UPDATE slot_symbols SET {} = {} WHERE id = {}",
                    column, payout, rec.id
                );

                sqlx::query(&query).execute(pool.get_ref()).await.ok();
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": rec.id,
                "message": "Symbol created successfully"
            })))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create symbol: {}", e)
        }))),
    }
}

// 配置卷轴符号
pub async fn add_reel_symbol(
    pool: web::Data<Pool<Postgres>>,
    req: web::Json<CreateReelSymbolRequest>,
) -> Result<HttpResponse> {
    let result = sqlx::query!(
        r#"
        INSERT INTO slot_reel_symbols 
        (slot_config_id, reel_number, position, symbol_id, weight)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (slot_config_id, reel_number, position) 
        DO UPDATE SET symbol_id = $4, weight = $5
        "#,
        req.slot_config_id,
        req.reel_number,
        req.position,
        req.symbol_id,
        req.weight
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Reel symbol configured successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to configure reel symbol: {}", e)
        }))),
    }
}

// 添加支付线
pub async fn add_payline(
    pool: web::Data<Pool<Postgres>>,
    req: web::Json<CreatePaylineRequest>,
) -> Result<HttpResponse> {
    let pattern_json = serde_json::to_value(&req.pattern).unwrap();

    let result = sqlx::query!(
        r#"
        INSERT INTO slot_paylines 
        (slot_config_id, line_number, pattern, is_active)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (slot_config_id, line_number)
        DO UPDATE SET pattern = $3, is_active = $4
        "#,
        req.slot_config_id,
        req.line_number,
        pattern_json,
        req.is_active
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Payline configured successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to configure payline: {}", e)
        }))),
    }
}

// 获取slot配置的所有符号
pub async fn get_slot_symbols(
    pool: web::Data<Pool<Postgres>>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let config_id = path.into_inner();

    let symbols = sqlx::query!(
        "SELECT * FROM slot_symbols WHERE slot_config_id = $1 ORDER BY value",
        config_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match symbols {
        Ok(symbols) => {
            let serializable_symbols: Vec<serde_json::Value> = symbols
                .into_iter()
                .map(|symbol| {
                    serde_json::json!({
                        "id": symbol.id,
                        "slot_config_id": symbol.slot_config_id,
                        "name": symbol.name,
                        "symbol_type": symbol.symbol_type,
                        "value": symbol.value,
                        "image_url": symbol.image_url,
                        "payout_2x": symbol.payout_2x,
                        "payout_3x": symbol.payout_3x,
                        "payout_4x": symbol.payout_4x,
                        "payout_5x": symbol.payout_5x,
                        "payout_6x": symbol.payout_6x,
                        "created_at": symbol.created_at.map(|t| t.to_string())
                    })
                })
                .collect();
            Ok(HttpResponse::Ok().json(serializable_symbols))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch symbols: {}", e)
        }))),
    }
}

// 获取slot配置的卷轴配置
pub async fn get_slot_reels(
    pool: web::Data<Pool<Postgres>>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let config_id = path.into_inner();

    let reels = sqlx::query!(
        r#"
        SELECT sr.*, s.name as symbol_name, s.symbol_type 
        FROM slot_reel_symbols sr
        JOIN slot_symbols s ON sr.symbol_id = s.id
        WHERE sr.slot_config_id = $1 
        ORDER BY sr.reel_number, sr.position
        "#,
        config_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match reels {
        Ok(reels) => {
            let serializable_reels: Vec<serde_json::Value> = reels
                .into_iter()
                .map(|reel| {
                    serde_json::json!({
                        "id": reel.id,
                        "slot_config_id": reel.slot_config_id,
                        "reel_number": reel.reel_number,
                        "position": reel.position,
                        "symbol_id": reel.symbol_id,
                        "weight": reel.weight,
                        "symbol_name": reel.symbol_name,
                        "symbol_type": reel.symbol_type
                    })
                })
                .collect();
            Ok(HttpResponse::Ok().json(serializable_reels))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch reel configuration: {}", e)
        }))),
    }
}

// 获取slot配置的支付线
pub async fn get_slot_paylines(
    pool: web::Data<Pool<Postgres>>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let config_id = path.into_inner();

    let paylines = sqlx::query!(
        "SELECT * FROM slot_paylines WHERE slot_config_id = $1 ORDER BY line_number",
        config_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match paylines {
        Ok(paylines) => {
            let serializable_paylines: Vec<serde_json::Value> = paylines
                .into_iter()
                .map(|payline| {
                    serde_json::json!({
                        "id": payline.id,
                        "slot_config_id": payline.slot_config_id,
                        "line_number": payline.line_number,
                        "pattern": payline.pattern,
                        "is_active": payline.is_active
                    })
                })
                .collect();
            Ok(HttpResponse::Ok().json(serializable_paylines))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch paylines: {}", e)
        }))),
    }
}

// 构建完整的slot机器实例
pub async fn build_slot_machine(
    pool: &Pool<Postgres>,
    config_id: i32,
) -> Result<crate::universal_slots::UniversalSlotMachine, String> {
    // 获取配置
    let config = sqlx::query!("SELECT * FROM slot_configurations WHERE id = $1", config_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to fetch config: {e}"))?;

    // 获取符号
    let symbol_records = sqlx::query!(
        "SELECT * FROM slot_symbols WHERE slot_config_id = $1",
        config_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch symbols: {e}"))?;

    let mut symbols = Vec::new();
    for record in symbol_records {
        let mut payouts = HashMap::new();
        if record.payout_2x.is_some() && record.payout_2x.unwrap() > 0 {
            payouts.insert(2, record.payout_2x.unwrap() as u32);
        }
        if record.payout_3x.is_some() && record.payout_3x.unwrap() > 0 {
            payouts.insert(3, record.payout_3x.unwrap() as u32);
        }
        if record.payout_4x.is_some() && record.payout_4x.unwrap() > 0 {
            payouts.insert(4, record.payout_4x.unwrap() as u32);
        }
        if record.payout_5x.is_some() && record.payout_5x.unwrap() > 0 {
            payouts.insert(5, record.payout_5x.unwrap() as u32);
        }
        if record.payout_6x.is_some() && record.payout_6x.unwrap() > 0 {
            payouts.insert(6, record.payout_6x.unwrap() as u32);
        }

        let symbol_type = match record.symbol_type.as_str() {
            "wild" => SymbolType::Wild,
            "scatter" => SymbolType::Scatter,
            "bonus" => SymbolType::Bonus,
            _ => SymbolType::Normal,
        };

        symbols.push(SlotSymbol {
            id: record.id,
            name: record.name,
            symbol_type,
            value: record.value as u32,
            image_url: record.image_url,
            payouts,
        });
    }

    // 获取卷轴配置
    let reel_records = sqlx::query!(
        "SELECT * FROM slot_reel_symbols WHERE slot_config_id = $1 ORDER BY reel_number, position",
        config_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch reels: {e}"))?;

    let mut reel_compositions = HashMap::new();
    for record in reel_records {
        reel_compositions
            .entry(record.reel_number as usize)
            .or_insert_with(Vec::new)
            .push((
                record.symbol_id.unwrap_or(0),
                record.weight.unwrap_or(1) as u32,
            ));
    }

    // 获取支付线
    let payline_records = sqlx::query!(
        "SELECT * FROM slot_paylines WHERE slot_config_id = $1 AND is_active = true ORDER BY line_number",
        config_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch paylines: {e}"))?;

    let mut paylines = Vec::new();
    for record in payline_records {
        let pattern: Vec<Vec<i32>> = serde_json::from_value(record.pattern)
            .map_err(|e| format!("Failed to parse payline pattern: {e}"))?;

        let pattern_tuples: Vec<(usize, usize)> = pattern
            .into_iter()
            .map(|p| (p[0] as usize, p[1] as usize))
            .collect();

        paylines.push(Payline {
            line_number: record.line_number as usize,
            pattern: pattern_tuples,
            is_active: record.is_active.unwrap_or(false),
        });
    }

    // 构建slot配置
    let slot_config = SlotConfig {
        id: config.id,
        name: config.name,
        rows: config.rows as usize,
        reels: config.reels as usize,
        is_megaway: config.is_megaway.unwrap_or(false),
        min_megaway_rows: config.min_megaway_rows.unwrap_or(2) as usize,
        max_megaway_rows: config.max_megaway_rows.unwrap_or(7) as usize,
        default_bet: config.default_bet.unwrap_or(1) as u32,
        min_bet: config.min_bet.unwrap_or(1) as u32,
        max_bet: config.max_bet.unwrap_or(1000) as u32,
        wild_enabled: config.wild_enabled.unwrap_or(false),
        free_spins_enabled: config.free_spins_enabled.unwrap_or(false),
        rtp_percentage: config
            .rtp_percentage
            .map(|r| r.to_string().parse::<f64>().unwrap_or(96.0))
            .unwrap_or(96.0),
    };

    let builder = SlotConfigBuilder {
        config: slot_config,
        symbols,
        reel_compositions,
        paylines,
    };

    Ok(builder.build())
}

// 测试spin接口
#[derive(Debug, Serialize, Deserialize)]
pub struct SpinRequest {
    pub slot_config_id: i32,
    pub bet_per_line: u32,
}

pub async fn test_spin(
    pool: web::Data<Pool<Postgres>>,
    req: web::Json<SpinRequest>,
) -> Result<HttpResponse> {
    match build_slot_machine(pool.get_ref(), req.slot_config_id).await {
        Ok(machine) => {
            let result = machine.spin(req.bet_per_line);
            Ok(HttpResponse::Ok().json(result))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        }))),
    }
}
