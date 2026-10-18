use crate::models::app_state::AppState;
use crate::service::metadata::MetadataService;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use std::sync::Arc;
#[derive(Debug, Deserialize)]
pub struct WmtsQuery {
    #[serde(default = "default_force")] // 默认值处理
    pub force: bool,

    #[serde(default = "default_f")]
    pub f: String,
}
fn default_force() -> bool {
    false // 当 force 不存在时默认为 false
}

fn default_f() -> String {
    String::new()
}
pub async fn get_wmts_xml(
    path: web::Path<String>,
    query: web::Query<WmtsQuery>,
    app_state: web::Data<AppState>,
    metadata_service: web::Data<Arc<MetadataService>>,
) -> impl Responder {
    let service_name = path.into_inner();
    if service_name.trim().is_empty() {
        return HttpResponse::BadRequest().finish();
    }
    let force = query.force;
    match metadata_service
        .get_wmts_xml(service_name, force, app_state)
        .await
    {
        Ok(xml) => HttpResponse::Ok()
            .content_type(ContentType::xml())
            .body(xml),
        Err(e) => {
            eprintln!("❌ Failed to get tile: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to get tile: {}", e))
        }
    }
}

pub async fn get_service_json(
    path: web::Path<String>,
    query: web::Query<WmtsQuery>,
    app_state: web::Data<AppState>,
    metadata_service: web::Data<Arc<MetadataService>>,
) -> impl Responder {
    let service_name = path.into_inner();
    if service_name.trim().is_empty() {
        return HttpResponse::BadRequest().finish();
    }
    if query.f.is_empty() {
        return HttpResponse::NotFound().finish();
    }
    let f = query.f.to_lowercase();
    let f_list = vec!["pjson", "json"];
    if !f_list.contains(&&*f) {
        return HttpResponse::BadRequest().finish();
    }
    match metadata_service
        .get_service_json(service_name, query.force, app_state)
        .await
    {
        Ok(_data) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(_data),
        Err(_e) => {
            eprintln!("❌ Failed to get tile: {}", _e);
            HttpResponse::InternalServerError().body(format!("Failed to get tile: {}", _e))
        }
    }
}
