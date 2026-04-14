pub mod mutation;
pub mod query;

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse},
};

use crate::middleware::SubjectContext;

pub type AdeptusSchema = Schema<query::QueryRoot, mutation::MutationRoot, EmptySubscription>;

pub fn build_schema(state: crate::AppState) -> AdeptusSchema {
    Schema::build(query::QueryRoot, mutation::MutationRoot, EmptySubscription)
        .data(state)
        .finish()
}

pub async fn graphql_handler(
    State(schema): State<AdeptusSchema>,
    subject: Option<Extension<SubjectContext>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();
    if let Some(Extension(ctx)) = subject {
        request = request.data(ctx);
    }
    schema.execute(request).await.into()
}

pub async fn graphql_playground() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}
