use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::Region,
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use std::time::Duration;
use uuid::Uuid;

use crate::config::StorageConfig;
use crate::utils::error::{AppError, AppResult};

use super::model::{FileMetadata, PresignedUrlResponse, UploadResponse};

pub struct StorageService {
    client: Client,
    bucket: String,
    max_file_size_bytes: u64,
}

impl StorageService {
    pub async fn new(config: StorageConfig) -> AppResult<Self> {
        let mut aws_config_builder = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.s3_region.clone()));

        // If custom endpoint is provided (for MinIO, LocalStack, etc.)
        if let Some(endpoint) = config.s3_endpoint {
            aws_config_builder = aws_config_builder.endpoint_url(endpoint);
        }

        let aws_config = aws_config_builder.load().await;
        let client = Client::new(&aws_config);

        let max_file_size_bytes = config.max_file_size_mb * 1024 * 1024;

        Ok(Self {
            client,
            bucket: config.s3_bucket,
            max_file_size_bytes,
        })
    }

    /// Upload a file to S3
    pub async fn upload_file(
        &self,
        file_name: String,
        content_type: String,
        data: Vec<u8>,
    ) -> AppResult<UploadResponse> {
        // Validate file size
        if data.len() as u64 > self.max_file_size_bytes {
            return Err(AppError::FileTooLarge);
        }

        let file_id = Uuid::new_v4().to_string();
        let key = format!("uploads/{}/{}", file_id, file_name);

        // Upload to S3
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(data.clone()))
            .content_type(&content_type)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("S3 upload error: {}", e)))?;

        // Generate public URL (adjust based on your S3 configuration)
        let url = format!(
            "https://{}.s3.amazonaws.com/{}",
            self.bucket, key
        );

        Ok(UploadResponse {
            file_id,
            file_name,
            file_size: data.len() as u64,
            content_type,
            url,
        })
    }

    /// Generate a presigned URL for direct upload
    pub async fn generate_presigned_upload_url(
        &self,
        file_name: String,
        content_type: String,
        expires_in_seconds: u64,
    ) -> AppResult<PresignedUrlResponse> {
        let file_id = Uuid::new_v4().to_string();
        let key = format!("uploads/{}/{}", file_id, file_name);

        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_in_seconds))
            .map_err(|e| AppError::InternalServer(format!("Presigning config error: {}", e)))?;

        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(&content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::ExternalService(format!("Presigning error: {}", e)))?;

        Ok(PresignedUrlResponse {
            url: presigned_request.uri().to_string(),
            expires_in_seconds,
        })
    }

    /// Generate a presigned URL for download
    pub async fn generate_presigned_download_url(
        &self,
        file_id: String,
        file_name: String,
        expires_in_seconds: u64,
    ) -> AppResult<PresignedUrlResponse> {
        let key = format!("uploads/{}/{}", file_id, file_name);

        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_in_seconds))
            .map_err(|e| AppError::InternalServer(format!("Presigning config error: {}", e)))?;

        let presigned_request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::ExternalService(format!("Presigning error: {}", e)))?;

        Ok(PresignedUrlResponse {
            url: presigned_request.uri().to_string(),
            expires_in_seconds,
        })
    }

    /// Delete a file from S3
    pub async fn delete_file(&self, file_id: String, file_name: String) -> AppResult<()> {
        let key = format!("uploads/{}/{}", file_id, file_name);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("S3 delete error: {}", e)))?;

        Ok(())
    }

    /// Get file metadata
    pub async fn get_file_metadata(
        &self,
        file_id: String,
        file_name: String,
    ) -> AppResult<FileMetadata> {
        let key = format!("uploads/{}/{}", file_id, file_name);

        let head_object = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| AppError::NotFound(format!("File not found: {}", e)))?;

        Ok(FileMetadata {
            file_id,
            file_name,
            file_size: head_object.content_length().unwrap_or(0) as u64,
            content_type: head_object.content_type().unwrap_or("").to_string(),
            uploaded_at: head_object
                .last_modified()
                .map(|t| t.to_string())
                .unwrap_or_default(),
        })
    }
}
