use actix_web::{web::{Bytes, Data}, HttpRequest, HttpResponse, Responder};
use tokio_postgres::Client;
use tokio::{fs as async_fs, io::AsyncWriteExt};
use crate::handlers::video_stream::{transcode_to_multiple_resolutions, generate_thumbnail};
use tokio::fs;

// #[post("/upload-chunk")]
pub async fn upload_chunk(
    req: HttpRequest,
    body: Bytes,
) -> impl Responder {
    let video_id = match req.headers().get("X-Video-ID") {
        Some(v) => match v.to_str() {
            Ok(id) => id.to_string(),
            Err(_) => return HttpResponse::BadRequest().body("Invalid video ID"),
        },
        None => return HttpResponse::BadRequest().body("Missing video ID"),
    };

    let path = format!("uploads/{}.part", video_id);

    let mut file = match async_fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open file"),
    };

    if let Err(_) = file.write_all(&body).await {
        return HttpResponse::InternalServerError().body("Failed to write chunk");
    }

    HttpResponse::Accepted().finish()
}

// #[post("/upload-complete")]
pub async fn handle_upload_complete(
    db: Data<Client>,
    payload: actix_web::web::Json<crate::handlers::UploadCompletePayload>,
) -> impl Responder {
    let video_id = payload.video_id;
    let title = payload.title.clone();
    let part_path = format!("uploads/{}.part", video_id);
    let original_path = format!("uploads/{}_original.mp4", video_id);

    if let Err(_) = fs::rename(&part_path, &original_path).await {
        return HttpResponse::InternalServerError().body("File move error");
    }

    if let Err(_) = db.execute(
        "INSERT INTO videos (id, title, original_path) VALUES ($1, $2, $3)",
        &[&video_id, &title, &original_path],
    ).await {
        return HttpResponse::InternalServerError().body("DB insert error");
    }

    let original_path_clone = original_path.clone();
    let db_clone = db.get_ref().clone();

    tokio::spawn(async move {
        let _ = transcode_to_multiple_resolutions(&original_path_clone, &format!("uploads/{}", video_id)).await;
        let _ = generate_thumbnail(&original_path_clone, &format!("uploads/{}_thumb.jpg", video_id)).await;

        let _ = db_clone.execute(
            "UPDATE videos SET thumbnail_path = $1, path_1080p = $2, path_720p = $3, path_480p = $4 WHERE id = $5",
            &[&format!("uploads/{}_thumb.jpg", video_id),
              &format!("uploads/{}_1080p.mp4", video_id),
              &format!("uploads/{}_720p.mp4", video_id),
              &format!("uploads/{}_480p.mp4", video_id),
              &video_id],
        ).await;
    });

    HttpResponse::Ok().body("Upload completed")
}
