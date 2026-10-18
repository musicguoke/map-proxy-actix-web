use crate::util::app_error::AppError;
use dashmap::DashMap;
use rand::prelude::IndexedRandom;
use serde::Deserialize;
use sqlx_sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct MapProxy {
    pub port: u16,
    pub cache_root: String,
    pub token_name: String,
    pub token_value: Vec<String>,
    pub base_url: String,
    pub source_base_url: String,
    pub services: HashMap<String, String>,
    pub tianditu_service: Vec<String>,
    pub tianditu_token: Vec<String>,
}

pub fn build_sqlite_url(cache_root: &str, db_name: &str) -> String {
    let path = PathBuf::from(cache_root).join(format!("{db_name}.db"));
    format!("sqlite://{}", path.display())
}
// ===== AppState =====
#[derive(Clone)]
pub struct AppState {
    // 使用 Arc 避免配置被多次克隆（配置通常较大）
    pub config: Arc<MapProxy>,
    // 可以添加其他共享状态
    db_pool: Arc<DashMap<String, SqlitePool>>,
    // pub cache: Arc<Cache>,
}

impl AppState {
    pub async fn new(config: MapProxy) -> Result<AppState, sqlx::Error> {
        let state = Self {
            config: Arc::new(config),
            db_pool: Arc::new(Default::default()),
        };
        // 2. 初始化所有服务的数据库连接
        for (service_name, _service_config) in &state.config.services {
            // 获取或创建该服务的数据库连接池
            // let pool = state.get_or_create_pool(service_name).await?;
            // 初始化该服务的 tiles 表
            state.init_tiles_table(service_name.clone()).await?;
            println!("✅ Initialized service: {}", service_name);
        }
        for (service_name) in &state.config.tianditu_service {
            // 获取或创建该服务的数据库连接池
            // let pool = state.get_or_create_pool(service_name).await?;
            // 初始化该服务的 tiles 表
            state.init_tiles_table(service_name.clone()).await?;
            println!("✅ Initialized service: {}", service_name);
        }

        Ok(state)
    }

    /// 获取配置的引用
    pub fn config(&self) -> &MapProxy {
        &self.config
    }

    pub fn get_token(&self) -> Option<String> {
        self.config.token_value.choose(&mut rand::rng()).cloned()
    }
    pub fn get_tianditu_token(&self) -> Option<String> {
        self.config.tianditu_token.choose(&mut rand::rng()).cloned()
    }

    pub fn get_token_name(&self) -> String {
        self.config.token_name.to_string()
    }
    pub async fn get_or_create_pool(&self, db_name: &str) -> Result<SqlitePool, sqlx::Error> {
        if let Some(pool) = self.db_pool.get(db_name) {
            return Ok(pool.clone());
        }
        if let Err(e) = fs::create_dir_all(self.config.cache_root.clone()) {
            eprintln!(
                "Failed to create directory {:?}: {}",
                self.config.cache_root, e
            );
            return Err(sqlx::Error::Configuration(
                format!("Failed to create cache directory: {}", e).into(),
            ));
        }

        let path = build_sqlite_url(&*self.config.cache_root, db_name);
        println!("db file path: {}", path);
        // 创建新连接池
        let opts = SqliteConnectOptions::from_str(&path)?
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .pragma("wal_autocheckpoint", "200");
        let pool = SqlitePool::connect_with(opts).await?;
        let entry = self.db_pool.entry(db_name.to_string());
        let pool = entry.or_insert(pool).clone();
        Ok(pool)
    }

    pub fn get_service_url(&self, service_name: &str) -> Result<&String, AppError> {
        self.config
            .services
            .get(service_name)
            .ok_or(AppError::ServiceNotFound(service_name.to_string()))
    }

    // 初始化 tiles 表
    async fn init_tiles_table(&self, service_name: String) -> Result<(), sqlx::Error> {
        // 获取或创建默认数据库连接池
        let pool = self.get_or_create_pool(&service_name).await?;

        // 创建表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tiles (
                zoom_level INTEGER NOT NULL,
                tile_column INTEGER NOT NULL,
                tile_row INTEGER NOT NULL,
                tile_data BLOB NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // 创建索引
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_tile
            ON tiles (zoom_level, tile_column, tile_row)
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS metadata (name text, value text)")
            .execute(&pool)
            .await?;
        println!("Tiles table and index created successfully");
        Ok(())
    }
    pub async fn close_all(&self) {
        println!("🔄 Closing all database connections...");
        let pools: Vec<SqlitePool> = self
            .db_pool
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        for pool in pools {
            if !pool.is_closed() {
                pool.close().await;
                println!("✅ Database connection closed");
            }
        }
        self.db_pool.clear();
        println!("✅ All database connections closed");
    }
}

impl MapProxy {
    /// 从 YAML 文件加载配置
    pub fn from_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: MapProxy = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

/// 查找配置文件的路径，按顺序查找
fn find_config_file() -> Result<String, String> {
    // 获取当前运行目录
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    // 定义要查找的路径列表
    let config_paths = vec![
        // 1. 当前目录下的 map-proxy-config.yaml
        current_dir.join("map-proxy-config.yaml"),
        // 2. 当前目录下的 map-proxy-config.yml (支持 .yml 扩展名)
        current_dir.join("map-proxy-config.yml"),
        // 3. /home/.map-proxy-config.yaml
        PathBuf::from("/home/.map-proxy-config.yaml").to_path_buf(),
    ];

    // 遍历查找
    for path in config_paths {
        if path.exists() && path.is_file() {
            let path_str = path.to_string_lossy().to_string();
            println!("✅ Found config file: {}", path_str);
            return Ok(path_str);
        }
    }

    // 如果都没找到，返回错误信息，列出所有查找过的路径
    let searched_paths: Vec<String> = vec![
        current_dir
            .join("map-proxy-config.yaml")
            .to_string_lossy()
            .to_string(),
        current_dir
            .join("map-proxy-config.yml")
            .to_string_lossy()
            .to_string(),
        "/home/.map-proxy-config.yaml".to_string(),
    ];

    Err(format!(
        "❌ Configuration file not found!\nSearched in:\n  - {}\n  - {}\n  - {}",
        searched_paths[0], searched_paths[1], searched_paths[2]
    ))
}

/// 加载配置的主函数
pub fn load_map_proxy_config() -> Result<MapProxy, String> {
    // 1. 查找配置文件
    let config_path = find_config_file()?;

    // 2. 加载并解析配置
    MapProxy::from_yaml(&config_path)
        .map_err(|e| format!("Failed to parse config file {}: {}", config_path, e))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载配置
    match load_map_proxy_config() {
        Ok(cfg) => {
            println!("✅ Configuration loaded successfully!");
            println!("   Cache root: {}", cfg.cache_root);
            println!("   Token name: {}", cfg.token_name);
            println!("   Token count: {}", cfg.token_value.len());
            println!("   Services count: {}", cfg.services.len());

            // 打印服务列表
            for (name, url) in &cfg.services {
                println!("     - {}: {}", name, url);
            }
            cfg
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // 这里启动你的服务器
    println!("🚀 Starting server...");

    // ... 你的服务器代码

    Ok(())
}
