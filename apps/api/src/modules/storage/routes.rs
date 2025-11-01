use axum::{
    extract::{Multipart, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::config::StorageConfig;
use crate::utils::{
    error::{AppError, AppResult},
    response::{no_content, ApiResponse},
};

use super::service::StorageService;

#[derive(Clone)]
struct StorageState {
    service: Arc<StorageService>,
}

#[derive(Deserialize)]
struct PresignedUrlQuery {
    file_name: String,
    content_type: String,
    #[serde(default = "default_expires_in")]
    expires_in: u64,
}

#[derive(Deserialize)]
struct DownloadQuery {
    file_name: String,
    #[serde(default = "default_expires_in")]
    expires_in: u64,
}

fn default_expires_in() -> u64 {
    3600 // 1 hour
}

pub fn routes(config: StorageConfig) -> Router {
    // Create service asynchronously - we'll need to handle this in main.rs
    // For now, create a placeholder router that will be initialized properly
    let rt = tokio::runtime::Handle::current();
    let service = rt.block_on(async {
        Arc::new(StorageService::new(config).await.expect("Failed to create storage service"))
    });

    let state = StorageState { service };

    Router::new()
        .route("/storage/upload", post(upload_file))
        .route("/storage/presigned-upload", get(get_presigned_upload_url))
        .route("/storage/presigned-download/:file_id", get(get_presigned_download_url))
        .route("/storage/:file_id", get(get_file_metadata))
        .route("/storage/:file_id", delete(delete_file))
        .with_state(state)
}

async fn upload_file(
    State(state): State<StorageState>,
    mut multipart: Multipart,
) -> AppResult<impl axum::response::IntoResponse> {
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart data: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());

                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read file: {}", e)))?;

                file_data = Some(data.to_vec());
            }
            _ => {}
        }
    }

    let file_name = file_name.ok_or_else(|| AppError::BadRequest("File name is required".to_string()))?;
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let file_data = file_data.ok_or_else(|| AppError::BadRequest("File data is required".to_string()))?;

    let response = state.service.upload_file(file_name, content_type, file_data).await?;

    Ok(ApiResponse::success(response))
}

async fn get_presigned_upload_url(
    State(state): State<StorageState>,
    Query(query): Query<PresignedUrlQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let response = state
        .service
        .generate_presigned_upload_url(query.file_name, query.content_type, query.expires_in)
        .await?;

    Ok(ApiResponse::success(response))
}

async fn get_presigned_download_url(
    State(state): State<StorageState>,
    Path(file_id): Path<String>,
    Query(query): Query<DownloadQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let response = state
        .service
        .generate_presigned_download_url(file_id, query.file_name, query.expires_in)
        .await?;

    Ok(ApiResponse::success(response))
}

async fn get_file_metadata(
    State(state): State<StorageState>,
    Path(file_id): Path<String>,
    Query(query): Query<DownloadQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let metadata = state.service.get_file_metadata(file_id, query.file_name).await?;

    Ok(ApiResponse::success(metadata))
}

async fn delete_file(
    State(state): State<StorageState>,
    Path(file_id): Path<String>,
    Query(query): Query<DownloadQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    state.service.delete_file(file_id, query.file_name).await?;

    Ok(no_content())
}
