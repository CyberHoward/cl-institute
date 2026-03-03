use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::OrganizationId;

/// Top-level institutional entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Named rules: key = rule name, value = rule description text.
    #[serde(default)]
    pub rules: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Organization {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: OrganizationId::new(),
            name: name.into(),
            description: None,
            rules: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_with_correct_defaults() {
        let org = Organization::new("Acme Corp");
        assert_eq!(org.name, "Acme Corp");
        assert!(org.description.is_none());
        assert!(org.rules.is_empty());
        assert!(org.created_at <= Utc::now());
        assert_eq!(org.created_at, org.updated_at);
    }

    #[test]
    fn new_generates_unique_ids() {
        let org1 = Organization::new("Org A");
        let org2 = Organization::new("Org B");
        assert_ne!(org1.id, org2.id);
    }

    #[test]
    fn json_roundtrip() {
        let mut org = Organization::new("Test Org");
        org.description = Some("A test organization".to_string());
        org.rules.insert("rule1".to_string(), "All purchases need approval".to_string());
        org.rules.insert("rule2".to_string(), "No weekend work".to_string());

        let json = serde_json::to_string(&org).unwrap();
        let deserialized: Organization = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, org.id);
        assert_eq!(deserialized.name, org.name);
        assert_eq!(deserialized.description, org.description);
        assert_eq!(deserialized.rules, org.rules);
    }

    #[test]
    fn toml_roundtrip() {
        let mut org = Organization::new("TOML Org");
        org.description = Some("Testing TOML".to_string());
        org.rules.insert("key".to_string(), "value".to_string());

        let toml_str = toml::to_string(&org).unwrap();
        let deserialized: Organization = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.id, org.id);
        assert_eq!(deserialized.name, org.name);
        assert_eq!(deserialized.description, org.description);
        assert_eq!(deserialized.rules, org.rules);
    }

    #[test]
    fn rules_hashmap_serializes_in_json() {
        let mut org = Organization::new("Rules Org");
        org.rules.insert("expense_limit".to_string(), "Max $5000 without approval".to_string());

        let json = serde_json::to_string(&org).unwrap();
        assert!(json.contains("expense_limit"));
        assert!(json.contains("Max $5000 without approval"));
    }

    #[test]
    fn optional_description_omitted_when_none_in_json() {
        let org = Organization::new("No Description Org");
        let json = serde_json::to_string(&org).unwrap();
        assert!(!json.contains("description"));
    }

    #[test]
    fn optional_description_present_when_some_in_json() {
        let mut org = Organization::new("Has Description Org");
        org.description = Some("Present".to_string());
        let json = serde_json::to_string(&org).unwrap();
        assert!(json.contains("\"description\""));
        assert!(json.contains("Present"));
    }

    #[test]
    fn deserialize_without_optional_fields() {
        // description is missing, rules is missing (both have defaults)
        let json = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Minimal Org",
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T00:00:00Z"
        });
        let org: Organization = serde_json::from_value(json).unwrap();
        assert_eq!(org.name, "Minimal Org");
        assert!(org.description.is_none());
        assert!(org.rules.is_empty());
    }
}
