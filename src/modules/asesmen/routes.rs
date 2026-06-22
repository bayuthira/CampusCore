use super::handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn asesmen_router() -> Router<DbPool> {
    let management = Router::new()
        .route(
            "/asesmen/skala-nilai/global",
            get(handler::global_scale_list_handler).put(handler::global_scale_save_handler),
        )
        .route(
            "/asesmen/skala-nilai/{prodi_id}",
            get(handler::scale_list_handler).put(handler::scale_save_handler),
        )
        .route(
            "/asesmen/nilai-akhir",
            get(handler::final_grade_classes_handler),
        )
        .route(
            "/asesmen/nilai-akhir/{jadwal_id}",
            get(handler::final_grade_detail_handler),
        )
        .route(
            "/asesmen/nilai-akhir/{jadwal_id}/ajukan",
            post(handler::final_grade_submit_handler),
        )
        .route(
            "/asesmen/nilai-akhir/{jadwal_id}/review",
            post(handler::final_grade_review_handler),
        )
        .route(
            "/asesmen/nilai-akhir/{jadwal_id}/publikasikan",
            post(handler::final_grade_publish_handler),
        )
        .route(
            "/asesmen",
            get(handler::list_handler).post(handler::create_handler),
        )
        .route("/asesmen/jadwal", get(handler::schedule_options_handler))
        .route(
            "/asesmen/{id}",
            get(handler::detail_handler).put(handler::update_handler),
        )
        .route("/asesmen/{id}/submit", post(handler::submit_handler))
        .route("/asesmen/{id}/review", post(handler::review_handler))
        .route(
            "/asesmen/{id}/dokumen/{jenis}",
            post(handler::upload_handler),
        )
        .route(
            "/asesmen/{id}/dokumen/{document_id}/download",
            get(handler::download_handler),
        )
        .route(
            "/asesmen/{id}/penggandaan",
            put(handler::production_handler),
        )
        .route("/asesmen/{id}/mulai", post(handler::start_handler))
        .route("/asesmen/{id}/selesai", post(handler::finish_handler))
        .route(
            "/asesmen/{id}/presensi/{enrollment_id}",
            put(handler::attendance_handler),
        )
        .route(
            "/asesmen/{id}/nilai/{enrollment_id}",
            put(handler::grade_handler),
        )
        .route("/asesmen/{id}/kunci", post(handler::lock_handler))
        .route(
            "/asesmen/{id}/buka-nilai",
            post(handler::reopen_grade_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "DOSEN".to_string(),
            "KAPRODI".to_string(),
            "SUPER_ADMIN".to_string(),
            "STAF_AKADEMIK".to_string(),
            "STAF_BAUM".to_string(),
        ])));

    let student = Router::new()
        .route("/asesmen-saya", get(handler::student_list_handler))
        .route("/nilai-saya", get(handler::student_grades_handler))
        .route(
            "/asesmen-saya/check-in",
            post(handler::student_check_in_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "MAHASISWA".to_string()
        ])));

    Router::new().merge(management).merge(student)
}
