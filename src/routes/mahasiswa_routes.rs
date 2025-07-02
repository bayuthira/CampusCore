use crate::{auth::require_role, db::DbPool, handlers};
use axum::{handler::Handler, middleware, routing::{get, post}, Router};

pub fn mahasiswa_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/api/mahasiswa/import-csv",
            post(handlers::mahasiswa_handler::import_mahasiswa_from_csv_handler).layer(
                middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ])),
            ),
        )
        .route(
            "/api/mahasiswa",
            get(handlers::mahasiswa_handler::get_all_mahasiswa_handler).layer(
                middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                    "DOSEN".to_string(),
                ])),
            )
            .post(handlers::mahasiswa_handler::create_mahasiswa_handler.layer(
                middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ])),
            )),
        )
        .route(
            "/api/mahasiswa/{id}",
            get(handlers::mahasiswa_handler::get_mahasiswa_by_id_handler)
                .put(handlers::mahasiswa_handler::update_mahasiswa_handler)
                .delete(handlers::mahasiswa_handler::delete_mahasiswa_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ]))),
        )
}