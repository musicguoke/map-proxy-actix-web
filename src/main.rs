use actix_web::web::Data;
use actix_web::{App, HttpServer};
use actix_web::{middleware};
use map_proxy_actix_web::handlers::config;
use map_proxy_actix_web::models::app_state::{AppState, load_map_proxy_config};
use map_proxy_actix_web::service::metadata::MetadataService;
use map_proxy_actix_web::service::tile::TileService;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();
    // 加载配置
    let map_proxy_config = match load_map_proxy_config() {
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
            eprintln!("❌ Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // 创建应用状态
    let app_state = match AppState::new(map_proxy_config).await {
        Ok(app_state) => app_state,
        Err(e) => {
            eprintln!("❌ Failed to load configuration ___ :{e} ");
            std::process::exit(1);
        }
    };
    println!("🚀 Starting server at http://127.0.0.1:9090");

    let app_state_clone = app_state.clone();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state_clone.clone()))
            .app_data(Data::new(Arc::new(TileService::new())))
            .app_data(Data::new(Arc::new(MetadataService::new())))
            .configure(config)
            .wrap(middleware::Logger::new(
                "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %Dms",
            ))
            .wrap(
                middleware::DefaultHeaders::new(), // .add(("X-Content-Type-Options", "nosniff"))
                                                   // .add(("X-Frame-Options", "DENY"))
                                                   // .add(("X-XSS-Protection", "1; mode=block"))
                                                   // .add((
                                                   //     "Strict-Transport-Security",
                                                   //     "max-age=31536000; includeSubDomains",
                                                   // ))
                                                   // .add(("Content-Security-Policy", "default-src 'self'")),
            )
    })
    .bind(("0.0.0.0", app_state.config.port))?
    .run();
    let app_state_for_shutdown = app_state.clone();
    let shutdown = async {
        tokio::signal::ctrl_c().await.ok();
        println!("🛑 Received shutdown signal, gracefully shutting down...");

        // 可以安全使用 app_state
        app_state_for_shutdown.close_all().await;
        println!("✅ All connections closed");
    };

    tokio::select! {
        _ = server => {},
        _ = shutdown => {}
    }
    Ok(())
}
