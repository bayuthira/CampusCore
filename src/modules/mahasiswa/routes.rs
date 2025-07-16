use super::handler;
use crate::{
    modules::auth::middleware::require_role,
    db::DbPool,
};
use axum::{handler::Handler, middleware, routing::{get, post}, Router};

pub fn mahasiswa_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/mahasiswa/template-csv",
            get(handler::download_mahasiswa_csv_template_handler)
        )
        .route(
            "/mahasiswa/import-csv",
            post(handler::import_mahasiswa_from_csv_handler).layer(
                middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ])),
            ),
        )
        .route(
            "/mahasiswa",
            get(handler::get_all_mahasiswa_handler).layer(
                middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                    "DOSEN".to_string(),
                ])),
            )
            .post(handler::create_mahasiswa_handler.layer(
                middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ])),
            )),
        )
        .route(
            "/mahasiswa/{id}",
            get(handler::get_mahasiswa_by_id_handler)
                .put(handler::update_mahasiswa_handler)
                .delete(handler::delete_mahasiswa_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ]))),
        )
}