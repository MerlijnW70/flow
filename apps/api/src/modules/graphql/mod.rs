mod schema;

pub use schema::{build_schema, GraphQLSchema};

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

/// GraphQL query handler
async fn graphql_handler(
    State(schema): State<GraphQLSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// GraphQL Playground UI
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn routes(schema: GraphQLSchema) -> Router {
    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .with_state(schema)
}
