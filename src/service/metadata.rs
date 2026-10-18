use crate::models::app_state::AppState;
use crate::util::app_error::AppError;
use crate::util::request::requests::send_tile_request;
use actix_web::web;
use log::{debug, error};

pub struct MetadataService {}

impl MetadataService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_service_json(
        &self,
        service_name: String,
        force: bool,
        app_state: web::Data<AppState>,
    ) -> Result<String, AppError> {
        let pool = match app_state.get_or_create_pool(service_name.as_str()).await {
            Ok(pool) => pool,
            Err(e) => {
                return Err(AppError::PoolError(format!("get pool error:{e}")));
            }
        };
        let query_result = sqlx::query_scalar(
            "select value from metadata where name = 'JSON' and value is not null ",
        )
        .fetch_one(&pool)
        .await;
        let service_json: Vec<u8> = match query_result {
            Ok(json) => json,
            Err(sqlx::Error::RowNotFound) => {
                debug!("WMTS 文件未找到，从远端获取");
                Vec::new()
            }
            Err(e) => {
                // 其他数据库错误
                error!("数据库查询错误: {}", e);
                return Err(AppError::DatabaseError(e));
            }
        };

        if service_json.is_empty() || force == true {
            let service_base_url = match app_state.get_service_url(service_name.as_str()) {
                Ok(service_base_url) => service_base_url,
                Err(e) => {
                    return Err(AppError::DatabaseError(sqlx::Error::RowNotFound));
                }
            };
            let tk = match app_state.get_token() {
                Some(k) => k,
                None => {
                    return Err(AppError::PoolError("get token error".to_string()));
                }
            };
            let token_name = app_state.get_token_name();
            let url = format!("{}?f=pjson&{}={}", service_base_url, token_name, tk);
            let json_data = match send_tile_request(&url, None, None).await {
                Ok(json_data) => json_data,
                Err(e) => return Err(e),
            };
            if json_data.is_empty() {
                return Err(AppError::ApiCallFound("获取的JSON数据为空".to_string()));
            }
            let data = String::from_utf8(json_data)
                .unwrap_or_else(|e| String::new());
            println!("save json to db:{}", service_name);
            sqlx::query("insert into metadata(value,name) values(?,?)")
                .bind(&data)
                .bind("JSON")
                .execute(&pool)
                .await
                .map_err(|e| AppError::DatabaseError(e))?;
            return Ok(data);
        }
        match String::from_utf8(service_json) {
            Ok(data) => Ok(data),
            Err(e) => Err(AppError::PoolError(format!("get pool error:{e}"))),
        }
    }

    pub async fn get_wmts_xml(
        &self,
        service_name: String,
        force: bool,
        app_state: web::Data<AppState>,
    ) -> Result<String, AppError> {
        let pool = match app_state.get_or_create_pool(service_name.as_str()).await {
            Ok(pool) => pool,
            Err(e) => {
                return Err(AppError::PoolError(format!("get pool error:{e}")));
            }
        };
        let query_result = sqlx::query_scalar(
            "select value from metadata where name = 'WMTS' and value is not null ",
        )
        .fetch_one(&pool)
        .await;
        let wmts_xml: Vec<u8> = match query_result {
            Ok(xml) => xml,
            Err(sqlx::Error::RowNotFound) => {
                debug!("WMTS 文件未找到，从远端获取");
                Vec::new()
            }
            Err(e) => {
                // 其他数据库错误
                error!("数据库查询错误: {}", e);
                return Err(AppError::DatabaseError(e));
            }
        };
        if wmts_xml.is_empty() || force == true {
            let service_base_url = match app_state.get_service_url(service_name.as_str()) {
                Ok(service_base_url) => service_base_url,
                Err(e) => {
                    return Err(AppError::DatabaseError(sqlx::Error::RowNotFound));
                }
            };
            let tk = match app_state.get_token() {
                Some(k) => k,
                None => {
                    return Err(AppError::PoolError("get token error".to_string()));
                }
            };
            let token_name = app_state.get_token_name();
            let url = format!("{}/WMTS?{}={}", service_base_url, token_name, tk);
            let wmts_data = match send_tile_request(&url, None, None).await {
                Ok(tile_data) => tile_data,
                Err(e) => return Err(e),
            };
            if wmts_data.is_empty() {
                return Err(AppError::ApiCallFound("获取的WMTS数据为空".to_string()));
            }
            let source_base_url = app_state.config.source_base_url.clone();

            let data = match String::from_utf8(wmts_data) {
                Ok(data) => {
                    let base_url = app_state.config.base_url.clone();
                    data.replace(&source_base_url, &*base_url)
                }
                Err(e) => String::new(),
            };
            println!("save xml to db:{}", service_name);
            sqlx::query("insert into metadata(value,name) values(?,?)")
                .bind(&data)
                .bind("WMTS")
                .execute(&pool)
                .await
                .map_err(|e| AppError::DatabaseError(e))?;
            return Ok(data);
        }
        match String::from_utf8(wmts_xml) {
            Ok(data) => Ok(data),
            Err(e) => Err(AppError::PoolError(format!("get pool error:{e}"))),
        }
    }
}
