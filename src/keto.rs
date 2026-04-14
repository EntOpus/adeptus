use crate::error::AdeptusError;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct KetoClient {
    client: Option<reqwest::Client>,
    read_url: Option<String>,
}

#[derive(Serialize)]
struct CheckRequest {
    namespace: String,
    object: String,
    relation: String,
    subject_id: String,
}

#[derive(Deserialize)]
struct CheckResponse {
    allowed: bool,
}

impl KetoClient {
    pub fn new(read_url: Option<String>) -> Self {
        if let Some(ref url) = read_url {
            info!(keto_url = %url, "Keto authorization enabled");
            Self {
                client: Some(reqwest::Client::new()),
                read_url: Some(url.clone()),
            }
        } else {
            info!("Keto not configured — all permission checks will pass (dev mode)");
            Self {
                client: None,
                read_url: None,
            }
        }
    }

    pub async fn check_permission(
        &self,
        namespace: &str,
        object: &str,
        relation: &str,
        subject_id: &str,
    ) -> Result<bool, AdeptusError> {
        let (Some(client), Some(url)) = (&self.client, &self.read_url) else {
            debug!(
                namespace,
                object, relation, subject_id, "Keto not configured, allowing"
            );
            return Ok(true);
        };

        let check_url = format!("{}/relation-tuples/check", url);

        let resp = client
            .post(&check_url)
            .json(&CheckRequest {
                namespace: namespace.to_string(),
                object: object.to_string(),
                relation: relation.to_string(),
                subject_id: subject_id.to_string(),
            })
            .send()
            .await
            .map_err(|e| {
                error!("Keto request failed: {e}");
                AdeptusError::Internal {
                    message: format!("Permission check failed: {e}"),
                }
            })?;

        if resp.status() == reqwest::StatusCode::FORBIDDEN {
            return Ok(false);
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Keto returned unexpected status");
            return Err(AdeptusError::Internal {
                message: format!("Permission check returned {status}"),
            });
        }

        let check: CheckResponse = resp.json().await.map_err(|e| {
            error!("Failed to parse Keto response: {e}");
            AdeptusError::Internal {
                message: format!("Failed to parse permission check response: {e}"),
            }
        })?;

        debug!(
            namespace,
            object,
            relation,
            subject_id,
            allowed = check.allowed,
            "Keto permission check"
        );

        Ok(check.allowed)
    }

    pub async fn require_permission(
        &self,
        namespace: &str,
        object: &str,
        relation: &str,
        subject_id: &str,
    ) -> Result<(), AdeptusError> {
        let allowed = self
            .check_permission(namespace, object, relation, subject_id)
            .await?;
        if !allowed {
            return Err(AdeptusError::InsufficientPermissions {
                required: format!("{namespace}:{object}#{relation}"),
            });
        }
        Ok(())
    }

    pub fn is_configured(&self) -> bool {
        self.read_url.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_without_keto() {
        let client = KetoClient::new(None);
        assert!(!client.is_configured());
    }

    #[test]
    fn test_new_with_keto() {
        let client = KetoClient::new(Some("http://keto:4466".to_string()));
        assert!(client.is_configured());
    }

    #[tokio::test]
    async fn test_check_permission_without_keto() {
        let client = KetoClient::new(None);
        let result = client
            .check_permission("document", "org-1", "read", "user-1")
            .await
            .unwrap();
        assert!(result, "unconfigured Keto should always allow");
    }

    #[tokio::test]
    async fn test_require_permission_without_keto() {
        let client = KetoClient::new(None);
        let result = client
            .require_permission("document", "*", "manage", "user-1")
            .await;
        assert!(result.is_ok(), "unconfigured Keto should always allow");
    }
}
