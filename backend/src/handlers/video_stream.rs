use actix_web::{HttpRequest, HttpResponse, Responder, web::Path, http::header};
use std::{fs::File, io::{Seek, SeekFrom, Read}, path::PathBuf};
use uuid::Uuid;
use tokio::process::Command;

// #[get("/videos/{video_id}/stream/{quality}")]
pub async fn stream_video(
    path: Path<(Uuid, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (video_id, quality) = path.into_inner();
    let file_path = format!("uploads/{}_{}.mp4", video_id, quality);
    let path_buf = PathBuf::from(&file_path);

    if !path_buf.exists() {
        return HttpResponse::NotFound().body("Video not found");
    }

    let mut file = File::open(&path_buf).unwrap();
    let metadata = file.metadata().unwrap();
    let total_size = metadata.len();

    let range = req.headers().get(header::RANGE)
        .and_then(|h| h.to_str().ok())
        .and_then(|r| {
            if r.starts_with("bytes=") {
                let parts: Vec<&str> = r[6..].split('-').collect();
                let start = parts.get(0).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                let end = parts.get(1).and_then(|e| e.parse::<u64>().ok()).unwrap_or(total_size - 1);
                Some((start, end))
            } else {
                None
            }
        });

    if let Some((start, end)) = range {
        let chunk_size = end - start + 1;
        let mut buffer = vec![0u8; chunk_size as usize];
        file.seek(SeekFrom::Start(start)).unwrap();
        file.read_exact(&mut buffer).unwrap();

        HttpResponse::PartialContent()
            .insert_header((header::CONTENT_TYPE, "video/mp4"))
            .insert_header((header::ACCEPT_RANGES, "bytes"))
            .insert_header((
                header::CONTENT_RANGE,
                format!("bytes {}-{}/{}", start, end, total_size),
            ))
            .body(buffer)
    } else {
        HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "video/mp4"))
            .body(actix_files::NamedFile::open(file_path).unwrap())
    }
}

pub async fn transcode_to_multiple_resolutions(input: &str, output_base: &str) -> std::io::Result<()> {
    let resolutions = [("1080p", "1080"), ("720p", "720"), ("480p", "480")];

    for (label, height) in resolutions {
        let output = format!("{}_{}.mp4", output_base, label);
        Command::new("ffmpeg")
            .args(["-i", input, "-vf", &format!("scale=-2:{}", height), "-c:v", "libx264", "-crf", "23", "-preset", "fast", &output])
            .output()
            .await?;
    }
    Ok(())
}

pub async fn generate_thumbnail(input: &str, output: &str) -> std::io::Result<()> {
    Command::new("ffmpeg")
        .args(["-i", input, "-ss", "00:00:02", "-vframes", "1", output])
        .output()
        .await?;
    Ok(())
}
