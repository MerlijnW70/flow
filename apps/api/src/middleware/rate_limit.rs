use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{num::NonZeroU32, sync::Arc, time::Duration};

pub type RateLimitLayer = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Create a rate limiter with specified requests per second
pub fn create_rate_limiter(requests_per_second: u32) -> RateLimitLayer {
    let quota = Quota::per_second(
        NonZeroU32::new(requests_per_second).expect("Invalid rate limit value"),
    );
    Arc::new(RateLimiter::direct(quota))
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    limiter: axum::extract::State<RateLimitLayer>,
    request: Request,
    next: Next,
) -> Response {
    match limiter.check() {
        Ok(_) => next.run(request).await,
        Err(_) => (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rate_limiter() {
        let limiter = create_rate_limiter(100);
        assert!(limiter.check().is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_enforcement() {
        let limiter = create_rate_limiter(2);

        // First two requests should succeed
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());

        // Third request should fail
        assert!(limiter.check().is_err());

        // Wait a bit and try again
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(limiter.check().is_ok());
    }
}
