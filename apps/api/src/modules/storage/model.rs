use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct PresignedUrlResponse {
    pub url: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct GetFileRequest {
    pub file_id: String,
}

#[derive(Debug, Serialize)]
pub struct FileMetadata {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub uploaded_at: String,
}
