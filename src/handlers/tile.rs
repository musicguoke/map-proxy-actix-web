use crate::models::app_state::AppState;
use crate::service::tile::TileService;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use std::sync::Arc;
use log::debug;

#[derive(Deserialize)]
pub struct TileXyz {
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_l")]
    pub z: i32,
    #[serde(default = "default_service_name")]
    pub service_name: String,
}
fn default_l() -> i32 {
    0
}
fn default_service_name() -> String {
    String::new()
}
pub async fn get_tile(
    path: web::Path<TileXyz>,
    app_config: web::Data<AppState>,
    tile_service: web::Data<Arc<TileService>>,
) -> impl Responder {
    let xyz = path.into_inner();
    let service_name = xyz.service_name.clone();
    let services_map = app_config.config.services.clone();

    if service_name.trim().is_empty() {
        return HttpResponse::BadRequest().finish();
    }

    // ✅ 检查服务是否存在
    if let Some(_target_url) = services_map.get(&service_name) {
        // ✅ 调用 tile_service 获取 tile
        match tile_service.get_tile(xyz, app_config).await {
            Ok(tile_data) => HttpResponse::Ok()
                .content_type(ContentType::jpeg())
                .body(tile_data),
            Err(e) => {
                eprintln!("❌ Failed to get tile: {}", e);
                HttpResponse::InternalServerError().body(format!("Failed to get tile: {}", e))
            }
        }
    } else {
        HttpResponse::NotFound().body(format!("Service '{}' not found", service_name))
    }
}

pub async fn get_tianditu_swdx_tile(
    query: web::Query<TileXyz>,
    app_config: web::Data<AppState>,
    tile_service: web::Data<Arc<TileService>>,
) -> impl Responder {
    let xyz = query.into_inner();
    // ✅ 调用 tile_service 获取 tile
    debug!("get_tianditu_swdx_tile");
    match tile_service.get_tianditu_swdx_tile(xyz, app_config).await {
        Ok(tile_data) => HttpResponse::Ok()
            .content_type("image/tile;charset=UTF-8")
            .body(tile_data),
        Err(e) => {
            eprintln!("❌ Failed to get tile: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to get tile: {}", e))
        }
    }
}

pub async fn get_tianditu_tile(
    path: web::Path<TileXyz>,
    app_config: web::Data<AppState>,
    tile_service: web::Data<Arc<TileService>>,
) -> impl Responder {
    let xyz = path.into_inner();
    let service_name = xyz.service_name.clone();
    let tianditu_service = app_config.config.tianditu_service.clone();

    if service_name.trim().is_empty() {
        return HttpResponse::BadRequest().finish();
    }
    if xyz.z > 18 {
        return HttpResponse::NotFound().finish();
    }

    if tianditu_service.contains(&service_name) {
        // ✅ 调用 tile_service 获取 tile
        match tile_service.get_tianditu_tile(xyz, app_config).await {
            Ok(tile_data) => HttpResponse::Ok()
                .content_type(ContentType::jpeg())
                .body(tile_data),
            Err(e) => {
                eprintln!("❌ Failed to get tile: {}", e);
                HttpResponse::InternalServerError().body(format!("Failed to get tile: {}", e))
            }
        }
    } else {
        HttpResponse::NotFound().body(format!("Service '{}' not found", service_name))
    }
}
