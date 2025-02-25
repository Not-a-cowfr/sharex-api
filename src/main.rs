use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use rand::distr::Alphanumeric;
use rand::Rng;
use dotenv::dotenv;
use std::env;
use bytes::{BytesMut, BufMut};

const BUFFER_SIZE: usize = 262_144;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let bind_address = env::var("BIND_ADDRESS").unwrap_or("0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or("8080".to_string()).parse::<u16>().unwrap();
    println!("Starting server on {}:{}", bind_address, port);

    fs::create_dir_all("./uploads").await.expect("Failed to create uploads directory");

    let max_file_size = env::var("MAX_FILE_SIZE")
        .unwrap_or("100".to_string())
        .parse::<usize>()
        .unwrap() * 1024 * 1024;

    HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::new(max_file_size))
            .service(web::resource("/upload").route(web::post().to(handle_upload)))
            .service(web::resource("/{file_id}").route(web::get().to(view_file)))
            .service(actix_files::Files::new("/files", "./uploads")
                .use_last_modified(true))
    })
    .bind(format!("{}:{}", bind_address, port))?
    .run()
    .await
}

async fn handle_upload(
    mut payload: Multipart
) -> Result<HttpResponse, Error> {
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

        let random_id: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();
        let filename = format!("{}.{}", random_id, extension);
        let filepath = format!("./uploads/{}", filename);

        let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
        let mut f = fs::File::create(&filepath).await?;

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            total_size += data.len() as u64;

            if total_size > max_file_size {
                fs::remove_file(&filepath).await?;
                return Ok(HttpResponse::PayloadTooLarge()
                    .body(format!("File size exceeds {} bytes", max_file_size)));
            }

            buffer.put_slice(&data);
            
            if buffer.len() >= BUFFER_SIZE {
                f.write_all(&buffer).await?;
                buffer.clear();
            }
        }
        
        if !buffer.is_empty() {
            f.write_all(&buffer).await?;
        }
        
        f.flush().await?;

        let base_url = env::var("URL").unwrap_or(format!("http://localhost:{}", port));
        let view_url = format!("{}/{}", base_url, filename);
        return Ok(HttpResponse::Ok()
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .json(serde_json::json!({
                "status": 200,
                "data": {
                    "link": view_url
                }
            })));
    }
    Ok(HttpResponse::BadRequest().body("No file uploaded"))
}

async fn view_file(
    path: web::Path<String>
) -> impl Responder {
    let filepath = format!("./uploads/{}", path);

    if let Ok(file_content) = fs::read(&filepath).await {
        let content_type = match path.as_str().split('.').last() {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            _ => "application/octet-stream",
        };
        return HttpResponse::Ok()
            .content_type(content_type)
            .append_header(("Content-Disposition", "inline"))
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .body(file_content);
    }

    HttpResponse::NotFound().body("File not found")
}