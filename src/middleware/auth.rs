use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

#[derive(Debug, Clone)]
pub struct SubjectContext {
    pub subject_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub async fn extract_subject(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let subject_id = request
        .headers()
        .get("X-Subject-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let ip_address = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string());

    let user_agent = request
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(subject_id) = subject_id {
        request.extensions_mut().insert(SubjectContext {
            subject_id,
            ip_address,
            user_agent,
        });
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request as HttpRequest, middleware, routing::get};
    use tower::ServiceExt;

    async fn echo_subject(request: Request) -> String {
        match request.extensions().get::<SubjectContext>() {
            Some(ctx) => format!(
                "{}|{}|{}",
                ctx.subject_id,
                ctx.ip_address.as_deref().unwrap_or("none"),
                ctx.user_agent.as_deref().unwrap_or("none"),
            ),
            None => "no-subject".to_string(),
        }
    }

    fn app() -> Router {
        Router::new()
            .route("/test", get(echo_subject))
            .layer(middleware::from_fn(extract_subject))
    }

    #[tokio::test]
    async fn test_extract_subject_with_all_headers() {
        let response = app()
            .oneshot(
                HttpRequest::builder()
                    .uri("/test")
                    .header("X-Subject-Id", "user-123")
                    .header("X-Forwarded-For", "1.2.3.4, 5.6.7.8")
                    .header("User-Agent", "TestClient/1.0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "user-123|1.2.3.4|TestClient/1.0");
    }

    #[tokio::test]
    async fn test_extract_subject_without_subject_id() {
        let response = app()
            .oneshot(
                HttpRequest::builder()
                    .uri("/test")
                    .header("X-Forwarded-For", "10.0.0.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "no-subject");
    }

    #[tokio::test]
    async fn test_extract_subject_no_optional_headers() {
        let response = app()
            .oneshot(
                HttpRequest::builder()
                    .uri("/test")
                    .header("X-Subject-Id", "user-bare")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, "user-bare|none|none");
    }
}
