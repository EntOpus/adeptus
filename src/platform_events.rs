use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub subject_id: String,
    pub resource: EventResource,
    pub metadata: serde_json::Value,
    pub access: Option<EventAccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResource {
    pub resource_type: String,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub resource_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAccess {
    pub namespace: String,
    pub object: String,
    pub relation: String,
}

impl PlatformEvent {
    pub fn new(
        event_type: &str,
        subject_id: &str,
        resource: EventResource,
        metadata: serde_json::Value,
        access: Option<EventAccess>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            source: "adeptus".to_string(),
            timestamp: Utc::now(),
            subject_id: subject_id.to_string(),
            resource,
            metadata,
            access,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_resource() -> EventResource {
        EventResource {
            resource_type: "document".to_string(),
            resource_id: "abc-123".to_string(),
            resource_name: Some("Test Doc".to_string()),
            resource_url: None,
        }
    }

    #[test]
    fn test_platform_event_new() {
        let event = PlatformEvent::new(
            "adeptus.document.created",
            "user-1",
            sample_resource(),
            serde_json::json!({"key": "value"}),
            None,
        );
        assert_eq!(event.event_type, "adeptus.document.created");
        assert_eq!(event.source, "adeptus");
        assert_eq!(event.subject_id, "user-1");
        assert!(!event.event_id.is_nil());
    }

    #[test]
    fn test_platform_event_with_access() {
        let access = EventAccess {
            namespace: "document".to_string(),
            object: "*".to_string(),
            relation: "manage".to_string(),
        };
        let event = PlatformEvent::new(
            "adeptus.document.deleted",
            "user-1",
            sample_resource(),
            serde_json::json!({}),
            Some(access),
        );
        assert!(event.access.is_some());
        let a = event.access.unwrap();
        assert_eq!(a.namespace, "document");
        assert_eq!(a.relation, "manage");
    }

    #[test]
    fn test_unique_event_ids() {
        let e1 = PlatformEvent::new("t", "s", sample_resource(), serde_json::json!({}), None);
        let e2 = PlatformEvent::new("t", "s", sample_resource(), serde_json::json!({}), None);
        assert_ne!(e1.event_id, e2.event_id);
    }

    #[test]
    fn test_platform_event_serde_roundtrip() {
        let event = PlatformEvent::new(
            "adeptus.document.created",
            "user-42",
            sample_resource(),
            serde_json::json!({"title": "Test"}),
            None,
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: PlatformEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_id, event.event_id);
        assert_eq!(deserialized.source, "adeptus");
    }
}
