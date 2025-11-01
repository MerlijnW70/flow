use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMetadata,
}

#[derive(Serialize)]
pub struct PaginationMetadata {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: None,
        }
    }

    pub fn with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message: Some(message),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

        Self {
            success: true,
            data,
            pagination: PaginationMetadata {
                page,
                per_page,
                total,
                total_pages,
            },
        }
    }
}

impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::CREATED, Json(ApiResponse::success(data)))
}

pub fn no_content() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
