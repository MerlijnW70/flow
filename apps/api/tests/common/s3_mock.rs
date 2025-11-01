// S3 Mock Infrastructure for Storage Tests
// Provides in-memory S3 simulation for testing file operations

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use uuid::Uuid;

/// Mock S3 file entry
#[derive(Clone, Debug)]
pub struct MockS3File {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub data: Bytes,
    pub size: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// In-memory mock S3 storage
#[derive(Clone)]
pub struct MockS3Storage {
    files: Arc<Mutex<HashMap<String, MockS3File>>>,
    bucket: String,
}

impl MockS3Storage {
    /// Create a new mock S3 storage instance
    pub fn new(bucket: &str) -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            bucket: bucket.to_string(),
        }
    }

    /// Upload a file to mock S3
    pub fn upload(
        &self,
        filename: &str,
        content_type: &str,
        data: Bytes,
    ) -> Result<String, String> {
        let file_id = Uuid::new_v4().to_string();
        let size = data.len();

        let file = MockS3File {
            id: file_id.clone(),
            filename: filename.to_string(),
            content_type: content_type.to_string(),
            data,
            size,
            created_at: chrono::Utc::now(),
        };

        self.files
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?
            .insert(file_id.clone(), file);

        Ok(file_id)
    }

    /// Download a file from mock S3
    pub fn download(&self, file_id: &str) -> Result<MockS3File, String> {
        self.files
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?
            .get(file_id)
            .cloned()
            .ok_or_else(|| "File not found".to_string())
    }

    /// Delete a file from mock S3
    pub fn delete(&self, file_id: &str) -> Result<(), String> {
        self.files
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?
            .remove(file_id)
            .ok_or_else(|| "File not found".to_string())?;

        Ok(())
    }

    /// Check if a file exists
    pub fn exists(&self, file_id: &str) -> bool {
        self.files
            .lock()
            .map(|files| files.contains_key(file_id))
            .unwrap_or(false)
    }

    /// Get file metadata without downloading
    pub fn metadata(&self, file_id: &str) -> Result<MockS3FileMetadata, String> {
        let file = self.download(file_id)?;
        Ok(MockS3FileMetadata {
            id: file.id,
            filename: file.filename,
            content_type: file.content_type,
            size: file.size,
            created_at: file.created_at,
        })
    }

    /// List all files in bucket
    pub fn list_files(&self) -> Vec<String> {
        self.files
            .lock()
            .map(|files| files.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Clear all files (for cleanup between tests)
    pub fn clear(&self) {
        if let Ok(mut files) = self.files.lock() {
            files.clear();
        }
    }

    /// Get total storage size
    pub fn total_size(&self) -> usize {
        self.files
            .lock()
            .map(|files| files.values().map(|f| f.size).sum())
            .unwrap_or(0)
    }

    /// Get bucket name
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

/// File metadata without data payload
#[derive(Clone, Debug)]
pub struct MockS3FileMetadata {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Generate a mock presigned URL
pub fn generate_presigned_url(
    bucket: &str,
    file_id: &str,
    operation: &str,
    expiry_secs: u64,
) -> String {
    let expiry = chrono::Utc::now().timestamp() + expiry_secs as i64;
    format!(
        "https://s3.mock.amazonaws.com/{}/{}?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Expires={}&operation={}",
        bucket, file_id, expiry, operation
    )
}

/// Create a test file with random content
pub fn create_test_file(size_kb: usize) -> Bytes {
    let size_bytes = size_kb * 1024;
    let data = vec![b'A'; size_bytes];
    Bytes::from(data)
}

/// Create a test image file (PNG)
pub fn create_test_image() -> Bytes {
    // Minimal valid PNG header
    let png_header = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    ];
    Bytes::from(png_header)
}

/// Create a test PDF file
pub fn create_test_pdf() -> Bytes {
    // Minimal valid PDF
    let pdf = b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n>>\nendobj\n%%EOF";
    Bytes::from(pdf.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_storage_upload_and_download() {
        let storage = MockS3Storage::new("test-bucket");
        let data = Bytes::from("test content");

        let file_id = storage
            .upload("test.txt", "text/plain", data.clone())
            .expect("Upload failed");

        assert!(storage.exists(&file_id));

        let downloaded = storage.download(&file_id).expect("Download failed");
        assert_eq!(downloaded.filename, "test.txt");
        assert_eq!(downloaded.data, data);
    }

    #[test]
    fn test_mock_storage_delete() {
        let storage = MockS3Storage::new("test-bucket");
        let data = Bytes::from("test content");

        let file_id = storage
            .upload("test.txt", "text/plain", data)
            .expect("Upload failed");

        assert!(storage.exists(&file_id));

        storage.delete(&file_id).expect("Delete failed");
        assert!(!storage.exists(&file_id));
    }

    #[test]
    fn test_mock_storage_metadata() {
        let storage = MockS3Storage::new("test-bucket");
        let data = Bytes::from("test content");

        let file_id = storage
            .upload("test.txt", "text/plain", data.clone())
            .expect("Upload failed");

        let metadata = storage.metadata(&file_id).expect("Metadata failed");
        assert_eq!(metadata.filename, "test.txt");
        assert_eq!(metadata.content_type, "text/plain");
        assert_eq!(metadata.size, data.len());
    }

    #[test]
    fn test_generate_presigned_url() {
        let url = generate_presigned_url("test-bucket", "file123", "getObject", 3600);
        assert!(url.contains("test-bucket"));
        assert!(url.contains("file123"));
        assert!(url.contains("X-Amz-Algorithm"));
        assert!(url.contains("operation=getObject"));
    }

    #[test]
    fn test_create_test_files() {
        let file = create_test_file(10); // 10 KB
        assert_eq!(file.len(), 10 * 1024);

        let image = create_test_image();
        assert!(image.len() > 0);
        assert_eq!(&image[0..4], &[0x89, 0x50, 0x4E, 0x47]); // PNG signature

        let pdf = create_test_pdf();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_mock_storage_list_and_clear() {
        let storage = MockS3Storage::new("test-bucket");

        storage
            .upload("file1.txt", "text/plain", Bytes::from("data1"))
            .unwrap();
        storage
            .upload("file2.txt", "text/plain", Bytes::from("data2"))
            .unwrap();

        let files = storage.list_files();
        assert_eq!(files.len(), 2);

        storage.clear();
        assert_eq!(storage.list_files().len(), 0);
    }

    #[test]
    fn test_mock_storage_total_size() {
        let storage = MockS3Storage::new("test-bucket");

        storage
            .upload("file1.txt", "text/plain", Bytes::from("12345"))
            .unwrap();
        storage
            .upload("file2.txt", "text/plain", Bytes::from("67890"))
            .unwrap();

        assert_eq!(storage.total_size(), 10);
    }
}
