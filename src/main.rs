use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use std::fs;
use std::io::Write;
use uuid::Uuid;
use dotenv::dotenv;
use std::env;
use mime_guess;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let bind_address = env::var("BIND_ADDRESS").unwrap_or("0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or("8080".to_string()).parse::<u16>().unwrap();
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/upload").route(web::post().to(handle_upload)))
            .service(web::resource("/view/{file_id}").route(web::get().to(view_file)))
            .service(actix_files::Files::new("/files", "./uploads"))
    })
    .bind(format!("{}:{}", bind_address, port))?
    .run()
    .await
}

async fn handle_upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let max_file_size = env::var("MAX_FILE_SIZE")
        .unwrap_or("100".to_string())
        .parse::<u64>()
        .unwrap() * 1024 * 1024;
    let mut total_size: u64 = 0;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_type = field.content_type();

        let extension = content_type
            .map(|mime| mime.subtype().as_str())
            .unwrap_or("bin");

        let filename = format!("{}.{}", Uuid::new_v4(), extension);
        let filepath = format!("./uploads/{}", filename);

        let mut f = fs::File::create(&filepath)?;

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            total_size += data.len() as u64;

            if total_size > max_file_size {
                fs::remove_file(&filepath)?;
                return Ok(HttpResponse::PayloadTooLarge()
                    .body(format!("File size exceeds {} bytes", max_file_size)));
            }

            f.write_all(&data)?;
        }

        let base_url = env::var("BASE_URL").unwrap_or(format!("http://localhost:{}", port));
        let view_url = format!("{}/view/{}", base_url, filename);
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": 200,
            "data": {
                "link": view_url
            }
        })));
    }
    Ok(HttpResponse::BadRequest().body("No file uploaded"))
}

async fn view_file(path: web::Path<String>) -> impl Responder {
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let base_url = env::var("BASE_URL").unwrap_or(format!("http://localhost:{}", port));
    let filepath = format!("./uploads/{}", path.as_str());
    let file_url = format!("{}/files/{}", base_url, path.as_str());

    if let Ok(metadata) = fs::metadata(&filepath) {
        if metadata.is_file() {
            let content_type = mime_guess::from_path(&filepath).first_or_octet_stream();
            let mime_str = content_type.as_ref();
            let (og_type, embed_html) = match content_type.as_ref() {
                "image/png" | "image/jpeg" | "image/gif" => (
                    "image",
                    format!(r#"<img src="{}" alt="Uploaded content" style="max-width: 100%;" />"#, file_url)
                ),
                "video/mp4" | "video/webm" => (
                    "video",
                    format!(r#"<video controls><source src="{}" type="{}">Your browser does not support the video tag.</video>"#, file_url, mime_str)
                ),
                "text/plain" => (
                    "article",
                    format!(r#"<pre style="white-space: pre-wrap;">Content not directly viewable here. Download: <a href="{}">{}</a></pre>"#, file_url, path.as_str())
                ),
                _ => (
                    "file",
                    format!(r#"<p>Content not directly viewable. Download: <a href="{}">{}</a></p>"#, file_url, path.as_str())
                ),
            };

            let html = format!(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <meta property="og:image" content="{}">
                    <meta property="og:type" content="{}">
                    <meta property="og:title" content="Uploaded File">
                    <meta charset="utf-8" />
                </head>
                <body>
                    {}
                </body>
                </html>
                "#,
                file_url, og_type, embed_html
            );
            return HttpResponse::Ok().content_type("text/html").body(html);
        }
    }
    HttpResponse::NotFound().body("File not found")
}