use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed, keyed::DashMapStateStore},
};
use std::{num::NonZeroU32, sync::Arc};

use crate::config::RateLimitConfig;
use crate::middleware::auth::SubjectContext;

type GlobalLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;
type KeyedLimiter = RateLimiter<String, DashMapStateStore<String>, DefaultClock>;

#[derive(Clone)]
pub struct RateLimitState {
    global: Option<Arc<GlobalLimiter>>,
    per_user: Option<Arc<KeyedLimiter>>,
    per_ip: Option<Arc<KeyedLimiter>>,
    enabled: bool,
}

impl RateLimitState {
    pub fn new(config: &RateLimitConfig) -> Self {
        if !config.enabled {
            return Self {
                global: None,
                per_user: None,
                per_ip: None,
                enabled: false,
            };
        }

        let burst = NonZeroU32::new(config.burst_size).unwrap_or(NonZeroU32::new(10).unwrap());

        let global = NonZeroU32::new(config.global_limit).map(|limit| {
            let quota = Quota::per_minute(limit).allow_burst(burst);
            Arc::new(RateLimiter::direct(quota))
        });

        let per_user = NonZeroU32::new(config.per_user_limit).map(|limit| {
            let quota = Quota::per_minute(limit).allow_burst(burst);
            Arc::new(RateLimiter::dashmap(quota))
        });

        let per_ip = NonZeroU32::new(config.per_ip_limit).map(|limit| {
            let quota = Quota::per_minute(limit).allow_burst(burst);
            Arc::new(RateLimiter::dashmap(quota))
        });

        Self {
            global,
            per_user,
            per_ip,
            enabled: true,
        }
    }
}

pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<RateLimitState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !state.enabled {
        return next.run(request).await;
    }

    if let Some(ref limiter) = state.global
        && limiter.check().is_err()
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("Retry-After", "60")],
            "Rate limit exceeded",
        )
            .into_response();
    }

    if let Some(ctx) = request.extensions().get::<SubjectContext>() {
        if let Some(ref limiter) = state.per_user
            && limiter.check_key(&ctx.subject_id).is_err()
        {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", "60")],
                "Per-user rate limit exceeded",
            )
                .into_response();
        }

        if let Some(ref ip) = ctx.ip_address
            && let Some(ref limiter) = state.per_ip
            && limiter.check_key(ip).is_err()
        {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", "60")],
                "Per-IP rate limit exceeded",
            )
                .into_response();
        }
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_disabled() {
        let config = RateLimitConfig {
            enabled: false,
            global_limit: 100,
            per_user_limit: 10,
            per_ip_limit: 5,
            burst_size: 2,
        };
        let state = RateLimitState::new(&config);
        assert!(!state.enabled);
        assert!(state.global.is_none());
    }

    #[test]
    fn test_new_enabled() {
        let config = RateLimitConfig {
            enabled: true,
            global_limit: 100,
            per_user_limit: 10,
            per_ip_limit: 5,
            burst_size: 2,
        };
        let state = RateLimitState::new(&config);
        assert!(state.enabled);
        assert!(state.global.is_some());
        assert!(state.per_user.is_some());
        assert!(state.per_ip.is_some());
    }
}
