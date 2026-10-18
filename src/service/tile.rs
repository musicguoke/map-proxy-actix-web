use crate::handlers::tile::TileXyz;
use crate::models::app_state::AppState;
use crate::util::app_error::AppError;
use crate::util::request::requests::send_tile_request;
use actix_web::web;
use log::{debug, error};

pub struct TileService {}
use rand::{Rng, RngExt};

impl TileService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_tile(
        &self,
        tile_xyz: TileXyz,
        app_state: web::Data<AppState>,
    ) -> Result<Vec<u8>, AppError> {
        let service_name = tile_xyz.service_name.clone();
        let pool = match app_state.get_or_create_pool(service_name.as_str()).await {
            Ok(pool) => pool,
            Err(e) => {
                return Err(AppError::PoolError(format!("get pool error:{e}")));
            }
        };

        let query_result = sqlx::query_scalar(
            "select tile_data from tiles where tile_row = ? and tile_column = ? and zoom_level = ? and tile_data is not null"
        )
            .bind(tile_xyz.x)
            .bind(tile_xyz.y)
            .bind(tile_xyz.z)
            .fetch_one(&pool)
            .await;

        let tile_data: Vec<u8> = match query_result {
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => {
                debug!("数据库中未找到瓦片，从远程获取");
                Vec::new()
            }
            Err(e) => {
                // 其他数据库错误
                error!("数据库查询错误: {}", e);
                return Err(AppError::DatabaseError(e));
            }
        };
        if tile_data.is_empty() {
            let service_base_url = match app_state.get_service_url(service_name.as_str()) {
                Ok(service_base_url) => service_base_url,
                Err(_e) => {
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
            let url = format!(
                "{}/tile/{}/{}/{}?{}={}",
                service_base_url, tile_xyz.z, tile_xyz.x, tile_xyz.y, token_name, tk
            );
            let tile_data = match send_tile_request(&url, None, None).await {
                Ok(tile_data) => tile_data,
                Err(e) => return Err(e),
            };
            // 存在就保存到sqlte 并返回一份给前端
            if tile_data.is_empty() {
                return Err(AppError::ApiCallFound("获取的瓦片数据为空".to_string()));
            }
            sqlx::query(
                "insert  into  tiles(zoom_level, tile_column, tile_row, tile_data) values (?,?,?,?)"
            )
                .bind(tile_xyz.z)
                .bind(tile_xyz.x)
                .bind(tile_xyz.y)
                .bind(&tile_data)
                .execute(&pool)
                .await
                .map_err(|e| AppError::DatabaseError(e))?;
            return Ok(tile_data);
        };
        Ok(Vec::from(tile_data))
    }
    pub async fn get_tianditu_tile(
        &self,
        tile_xyz: TileXyz,
        app_state: web::Data<AppState>,
    ) -> Result<Vec<u8>, AppError> {
        let service_type = tile_xyz.service_name.clone();
        let pool = match app_state.get_or_create_pool(service_type.as_str()).await {
            Ok(pool) => pool,
            Err(e) => {
                return Err(AppError::PoolError(format!("get pool error:{e}")));
            }
        };
        let query_result = sqlx::query_scalar(
            "select tile_data from tiles where tile_row = ? and tile_column = ? and zoom_level = ? and tile_data is not null"
        )
            .bind(tile_xyz.y)
            .bind(tile_xyz.x)
            .bind(tile_xyz.z)
            .fetch_one(&pool)
            .await;

        let tile_data: Vec<u8> = match query_result {
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => {
                debug!("数据库中未找到瓦片，从远程获取");
                Vec::new()
            }
            Err(e) => {
                // 其他数据库错误
                error!("数据库查询错误: {}", e);
                return Err(AppError::DatabaseError(e));
            }
        };
        if tile_data.is_empty() {
            let tk = match app_state.get_tianditu_token() {
                Some(k) => k,
                None => {
                    return Err(AppError::PoolError("get token error".to_string()));
                }
            };
            let mut rng = rand::rng();
            let domain = rng.random_range(0..7);

            //https://t0.tianditu.gov.cn/cta_w/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=cta&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILEMATRIX=9&TILEROW=216&TILECOL=411&tk=f060cb775b31a988c3c21fb5d9a7ccd7
            let parts: Vec<&str> = service_type.split('_').collect();
            let layer_name = match parts.first() {
                Some(l) => l,
                None => return Err(AppError::PoolError("service type error".to_string())),
            };
            let tilematrixset = match parts.last() {
                Some(l) => l,
                None => return Err(AppError::PoolError("service type error".to_string())),
            };
            let url = format!(
                "https://t{}.tianditu.gov.cn/{}/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER={}&STYLE=default&TILEMATRIXSET={}&FORMAT=tiles&TILEMATRIX={}&TILEROW={}&TILECOL={}&tk={}",
                domain,
                service_type,
                layer_name,
                tilematrixset,
                tile_xyz.z,
                tile_xyz.y,
                tile_xyz.x,
                tk
            );
            println!("request: url {}", url);
            let tile_data = match send_tile_request(&url, None, None).await {
                Ok(tile_data) => tile_data,
                Err(e) => return Err(e),
            };
            // 存在就保存到sqlte 并返回一份给前端
            if tile_data.is_empty() {
                return Err(AppError::ApiCallFound("获取的瓦片数据为空".to_string()));
            }
            if (tile_data.len() > 256) {
                sqlx::query(
                    "insert  into  tiles(zoom_level, tile_column, tile_row, tile_data) values (?,?,?,?)"
                )
                    .bind(tile_xyz.z)
                    .bind(tile_xyz.x)
                    .bind(tile_xyz.y)
                    .bind(&tile_data)
                    .execute(&pool)
                    .await
                    .map_err(|e| AppError::DatabaseError(e))?;
            }
            return Ok(tile_data);
        };
        Ok(Vec::from(tile_data))
    }

    pub async fn get_tianditu_swdx_tile(
        &self,
        tile_xyz: TileXyz,
        app_state: web::Data<AppState>,
    ) -> Result<Vec<u8>, AppError> {
        let service_type = "swdx";
        let pool = match app_state.get_or_create_pool(service_type).await {
            Ok(pool) => pool,
            Err(e) => {
                return Err(AppError::PoolError(format!("get pool error:{e}")));
            }
        };
        let query_result = sqlx::query_scalar(
            "select tile_data from tiles where tile_row = ? and tile_column = ? and zoom_level = ? and tile_data is not null"
        )
            .bind(tile_xyz.x)
            .bind(tile_xyz.y)
            .bind(tile_xyz.z)
            .fetch_one(&pool)
            .await;

        let tile_data: Vec<u8> = match query_result {
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => {
                debug!("数据库中未找到瓦片，从远程获取");
                Vec::new()
            }
            Err(e) => {
                // 其他数据库错误
                error!("数据库查询错误: {}", e);
                return Err(AppError::DatabaseError(e));
            }
        };
        if tile_data.is_empty() {
            let tk = match app_state.get_tianditu_token() {
                Some(k) => k,
                None => {
                    return Err(AppError::PoolError("get token error".to_string()));
                }
            };
            let mut rng = rand::rng();
            let domain = rng.random_range(0..7);
            let url = format!(
                "https://t{}.tianditu.gov.cn/mapservice/swdx?T=elv_c&tk={}&x={}&y={}&l={}",
                domain, tk, tile_xyz.x, tile_xyz.y, tile_xyz.z,
            );
            println!("request: url {}", url);
            let tile_data = match send_tile_request(&url, None, None).await {
                Ok(tile_data) => tile_data,
                Err(e) => return Err(e),
            };
            debug!("{:?}", tile_data);
            println!("Response size:{:?}", tile_data.len() / 1024);
            // 存在就保存到sqlte 并返回一份给前端
            sqlx::query(
                    "insert  into  tiles(zoom_level, tile_column, tile_row, tile_data) values (?,?,?,?)"
                )
                    .bind(tile_xyz.z)
                    .bind(tile_xyz.x)
                    .bind(tile_xyz.y)
                    .bind(&tile_data)
                    .execute(&pool)
                    .await
                    .map_err(|e| AppError::DatabaseError(e))?;
            return Ok(tile_data);
        };
        Ok(Vec::from(tile_data))
    }
}
