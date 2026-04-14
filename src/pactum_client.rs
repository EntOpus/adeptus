use serde::Deserialize;
use tracing::{debug, error, info};

use crate::config::PactumConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentResult {
    Active,
    NotGiven { license_slug: String },
    Withdrawn { license_slug: String },
}

#[derive(Clone)]
pub struct PactumClient {
    client: Option<reqwest::Client>,
    graphql_url: Option<String>,
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<ConsentStatusData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConsentStatusData {
    consent_status: ConsentStatusVariant,
}

#[derive(Deserialize)]
#[serde(tag = "status")]
enum ConsentStatusVariant {
    Active {},
    NotGiven {},
    Withdrawn {},
}

impl PactumClient {
    pub fn new(config: &PactumConfig) -> Self {
        if let Some(ref url) = config.graphql_url {
            info!(pactum_url = %url, "Pactum consent checking enabled");
            Self {
                client: Some(reqwest::Client::new()),
                graphql_url: Some(url.clone()),
            }
        } else {
            info!("Pactum not configured — consent checks disabled (all content accessible)");
            Self {
                client: None,
                graphql_url: None,
            }
        }
    }

    pub async fn check_consent(&self, subject_id: &str, license_slug: &str) -> ConsentResult {
        let (Some(client), Some(url)) = (&self.client, &self.graphql_url) else {
            debug!(
                subject_id,
                license_slug, "Pactum not configured, granting access"
            );
            return ConsentResult::Active;
        };

        let query = serde_json::json!({
            "query": r#"
                query ConsentCheck($subjectId: String!, $licenseSlug: String!) {
                    consentStatus(subjectId: $subjectId, licenseSlug: $licenseSlug) {
                        ... on Active { status: __typename }
                        ... on Withdrawn { status: __typename }
                        ... on NotGiven { status: __typename }
                    }
                }
            "#,
            "variables": {
                "subjectId": subject_id,
                "licenseSlug": license_slug,
            }
        });

        let resp = match client.post(url).json(&query).send().await {
            Ok(r) => r,
            Err(e) => {
                error!("Pactum request failed: {e} — granting access (fail open)");
                return ConsentResult::Active;
            }
        };

        let body: GraphQLResponse = match resp.json().await {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to parse Pactum response: {e} — granting access (fail open)");
                return ConsentResult::Active;
            }
        };

        match body.data {
            Some(data) => match data.consent_status {
                ConsentStatusVariant::Active {} => ConsentResult::Active,
                ConsentStatusVariant::NotGiven {} => ConsentResult::NotGiven {
                    license_slug: license_slug.to_string(),
                },
                ConsentStatusVariant::Withdrawn {} => ConsentResult::Withdrawn {
                    license_slug: license_slug.to_string(),
                },
            },
            None => {
                error!("Pactum returned no data — granting access (fail open)");
                ConsentResult::Active
            }
        }
    }

    pub fn is_configured(&self) -> bool {
        self.graphql_url.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_without_pactum() {
        let config = PactumConfig { graphql_url: None };
        let client = PactumClient::new(&config);
        assert!(!client.is_configured());
    }

    #[test]
    fn test_new_with_pactum() {
        let config = PactumConfig {
            graphql_url: Some("http://pactum:3004/graphql".to_string()),
        };
        let client = PactumClient::new(&config);
        assert!(client.is_configured());
    }

    #[tokio::test]
    async fn test_check_consent_without_pactum() {
        let config = PactumConfig { graphql_url: None };
        let client = PactumClient::new(&config);
        let result = client.check_consent("user-1", "doc-eula").await;
        assert_eq!(result, ConsentResult::Active);
    }
}
