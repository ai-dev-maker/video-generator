mod request;

use request::{
    runpod_request,
    save_prompt,
};

use axum::{
    response::Html,
    routing::{get, post},
    Router,
    Json,
};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GenerateRequest {
    prompt: String,
}

#[derive(Serialize)]
struct GenerateResponse {
    image: String
}

#[tokio::main]
async fn main() -> Result<()>{
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/generate", post(generate_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn read_html_from_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

async fn index_handler() -> Html<String> {
    let html = read_html_from_file("templates/index.html")
        .await
        .unwrap_or_else(|_| "<h1>Error Loading HTML</h1>".to_string());

    Html(html)
}

async fn generate_handler(
    Json(payload): Json<GenerateRequest>,
) -> Json<GenerateResponse> {
    let prompt = payload.prompt;
    println!("Prompt: {}", prompt);

    save_prompt(&prompt)
        .await
        .expect("Cannot Write User's Prompt to workflow.json");

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let base64 = runpod_request()
        .await
        .unwrap();

    Json(GenerateResponse { image: base64 })
}
