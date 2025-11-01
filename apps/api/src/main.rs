use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use vibe_api::{metrics, modules};

#[derive(OpenApi)]
#[openapi(
    paths(hello),
    components(schemas())
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/hello",
    responses(
        (status = 200, description = "Hello World")
    )
)]
async fn hello() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() {
    // Initialize metrics
    let _prometheus_handle = metrics::init_metrics();

    // For now, create routes without database
    // In production, you would:
    // 1. Load config from .env
    // 2. Create database pool
    // 3. Pass pool to routes that need it

    let app = Router::new()
        .route("/hello", get(hello))
        .merge(metrics::routes())
        // Health routes require database - placeholder for now
        // .merge(modules::health::routes(db_pool.clone()))
        // GraphQL requires database - placeholder for now
        // .merge(modules::graphql::routes(modules::graphql::build_schema(db_pool)))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Server running at http://localhost:3000/swagger-ui");
    println!("📊 Metrics available at http://localhost:3000/metrics");
    println!("💚 Health check at http://localhost:3000/health");
    println!("✅ Readiness check at http://localhost:3000/ready");
    println!("\nNote: GraphQL and database-dependent endpoints require database setup");

    axum::serve(listener, app)
        .await
        .unwrap();
}
