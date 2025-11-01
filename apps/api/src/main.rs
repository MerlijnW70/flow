use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
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

    // Get database URL from environment
    // Try DATABASE_PUBLIC_URL first (Railway proxy), then fall back to DATABASE_URL
    let database_url = std::env::var("DATABASE_PUBLIC_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL or DATABASE_PUBLIC_URL must be set");

    println!("🔗 Connecting to database...");

    // Create database pool with increased timeout and retry settings
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(300))
        .connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            eprintln!("❌ Failed to connect to database: {}", e);
            eprintln!("Database URL format: postgresql://user:pass@host:port/db");
            panic!("Database connection failed");
        });

    println!("✅ Connected to database");

    // Run migrations
    println!("🔄 Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    println!("✅ Migrations completed");

    let app = Router::new()
        .route("/hello", get(hello))
        .merge(metrics::routes())
        .merge(modules::health::routes(db_pool.clone()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    // Use PORT from environment (Railway provides this) or default to 3000
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string());

    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap();

    println!("🚀 Server running on port {}", port);
    println!("📊 Metrics available at /metrics");
    println!("💚 Health check at /health");
    println!("✅ Readiness check at /ready");
    println!("📖 Swagger UI at /swagger-ui");

    axum::serve(listener, app)
        .await
        .unwrap();
}
