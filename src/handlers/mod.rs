use crate::handlers::metadata::get_service_json;
use crate::handlers::metadata::get_wmts_xml;
use crate::handlers::tile::{get_tianditu_swdx_tile, get_tianditu_tile, get_tile};
use actix_web::web;

pub mod metadata;
pub mod tile;
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/arcgis/rest/services")
            .route(
                "/{service_name}/MapServer/tile/{z}/{x}/{y}",
                web::get().to(get_tile),
            )
            //
            .route(
                "/{service_name}/MapServer/WMTS/tile/{version}/{service_n}/{Style}/{TileMatrixSet}/{z}/{x}/{y}",
                web::get().to(get_tile),
            )

            .route(
                "/{service_name}/MapServer/WMTS/1.0.0/WMTSCapabilities.xml",
                web::get().to(get_wmts_xml),
            )
            .route(
                "/{service_name}/MapServer/WMTS",
                web::get().to(get_wmts_xml),
            )
            .route("/{service_name}/MapServer", web::get().to(get_service_json))
            .route("/tianditu/{service_name}/wmts/tile/{z}/{x}/{y}", web::get().to(get_tianditu_tile)),
    )
        .service(web::scope("/sw")
            .route("/swdx", web::get().to(get_tianditu_swdx_tile))
        );
}
