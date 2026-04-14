use crate::platform_events::PlatformEvent;
use anyhow::Result;
use async_nats::jetstream;
use std::time::Duration;
use tracing::{error, info};

#[derive(Clone)]
pub struct EventPublisher {
    jetstream: Option<jetstream::Context>,
    stream_name: String,
}

impl EventPublisher {
    pub async fn new(
        nats_url: Option<&str>,
        stream_name: String,
        max_age_days: u64,
    ) -> Result<Self> {
        let jetstream = match nats_url {
            Some(url) => {
                let client = async_nats::connect(url).await?;
                let js = jetstream::new(client);

                let stream_config = jetstream::stream::Config {
                    name: stream_name.clone(),
                    subjects: vec!["adeptus.>".to_string()],
                    retention: jetstream::stream::RetentionPolicy::Limits,
                    max_age: Duration::from_secs(max_age_days * 24 * 60 * 60),
                    storage: jetstream::stream::StorageType::File,
                    ..Default::default()
                };

                match js.get_or_create_stream(stream_config).await {
                    Ok(stream) => {
                        info!(
                            stream = %stream_name,
                            state = ?stream.cached_info().state,
                            "NATS JetStream stream ready"
                        );
                    }
                    Err(e) => {
                        error!(stream = %stream_name, "Failed to create NATS stream: {e}");
                        return Err(e.into());
                    }
                }

                Some(js)
            }
            None => {
                info!("NATS not configured — event publishing disabled");
                None
            }
        };

        Ok(Self {
            jetstream,
            stream_name,
        })
    }

    pub async fn publish(&self, event: &PlatformEvent) {
        let Some(ref js) = self.jetstream else {
            tracing::debug!(
                event_type = %event.event_type,
                event_id = %event.event_id,
                "Event publishing disabled (no NATS), skipping"
            );
            return;
        };

        let payload = match serde_json::to_vec(event) {
            Ok(p) => p,
            Err(e) => {
                error!(
                    event_type = %event.event_type,
                    event_id = %event.event_id,
                    "Failed to serialize event: {e}"
                );
                return;
            }
        };

        match js.publish(event.event_type.clone(), payload.into()).await {
            Ok(ack) => match ack.await {
                Ok(_) => {
                    tracing::debug!(
                        event_type = %event.event_type,
                        event_id = %event.event_id,
                        "Event published"
                    );
                }
                Err(e) => {
                    error!(
                        event_type = %event.event_type,
                        event_id = %event.event_id,
                        "Event publish ack failed: {e}"
                    );
                }
            },
            Err(e) => {
                error!(
                    event_type = %event.event_type,
                    event_id = %event.event_id,
                    "Failed to publish event: {e}"
                );
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.jetstream.is_some()
    }

    pub fn is_configured(&self) -> bool {
        self.jetstream.is_some()
    }

    pub fn stream_name(&self) -> &str {
        &self.stream_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_without_nats() {
        let publisher = EventPublisher::new(None, "TEST_STREAM".to_string(), 7)
            .await
            .unwrap();
        assert!(!publisher.is_connected());
        assert!(!publisher.is_configured());
    }

    #[tokio::test]
    async fn test_stream_name() {
        let publisher = EventPublisher::new(None, "ADEPTUS_EVENTS".to_string(), 30)
            .await
            .unwrap();
        assert_eq!(publisher.stream_name(), "ADEPTUS_EVENTS");
    }

    #[tokio::test]
    async fn test_publish_without_nats_does_not_panic() {
        let publisher = EventPublisher::new(None, "TEST_STREAM".to_string(), 7)
            .await
            .unwrap();

        let event = PlatformEvent::new(
            "adeptus.test",
            "test-subject",
            crate::platform_events::EventResource {
                resource_type: "test".to_string(),
                resource_id: uuid::Uuid::new_v4().to_string(),
                resource_name: None,
                resource_url: None,
            },
            serde_json::json!({}),
            None,
        );

        publisher.publish(&event).await;
    }
}
