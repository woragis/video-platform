// src/routes/video.rs
use actix_web::web::{self, ServiceConfig};

use crate::handlers::video_upload::{upload_chunk, handle_upload_complete};
use crate::handlers::video_stream::stream_video;

pub fn configure_video_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/videos")
            .route("/upload-chunk", web::post().to(upload_chunk))
            .route("/upload-complete", web::post().to(handle_upload_complete))
            .route("/{video_id}/stream/{quality}", web::get().to(stream_video))
    );
}
