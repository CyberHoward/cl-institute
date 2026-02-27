use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{CapabilityId, IntegrationId, OrganizationId};

/// An external integration available to the institution.
///
/// Maps to the integration registry in the architecture (Section 3.2.4).
/// Each integration exposes capabilities with defined input/output schemas.
/// The registry does NOT contain credentials — those are environment config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: IntegrationId,
    pub organization_id: OrganizationId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Integration {
    pub fn new(organization_id: OrganizationId, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: IntegrationId::new(),
            organization_id,
            name: name.into(),
            description: None,
            capabilities: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A capability exposed by an integration.
///
/// Examples: DocuSign -> `route_for_signature`, SAP -> `create_purchase_order`.
/// The integration compiler references these when compiling edge specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: CapabilityId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for the capability's input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    /// JSON Schema for the capability's output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_new_with_empty_capabilities() {
        let org_id = OrganizationId::new();
        let integration = Integration::new(org_id, "DocuSign");

        assert_eq!(integration.organization_id, org_id);
        assert_eq!(integration.name, "DocuSign");
        assert!(integration.description.is_none());
        assert!(integration.capabilities.is_empty());
        assert_eq!(integration.created_at, integration.updated_at);
    }

    #[test]
    fn integration_unique_ids() {
        let org_id = OrganizationId::new();
        let i1 = Integration::new(org_id, "A");
        let i2 = Integration::new(org_id, "B");
        assert_ne!(i1.id, i2.id);
    }

    #[test]
    fn adding_capabilities() {
        let org_id = OrganizationId::new();
        let mut integration = Integration::new(org_id, "SAP");

        let cap = Capability {
            id: CapabilityId::new(),
            name: "create_purchase_order".to_string(),
            description: Some("Creates a PO in SAP".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "vendor_id": {"type": "string"},
                    "amount": {"type": "number"}
                }
            })),
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "po_number": {"type": "string"}
                }
            })),
        };

        integration.capabilities.push(cap);
        assert_eq!(integration.capabilities.len(), 1);
        assert_eq!(integration.capabilities[0].name, "create_purchase_order");
    }

    #[test]
    fn capability_schemas_as_json_values() {
        let cap = Capability {
            id: CapabilityId::new(),
            name: "route_for_signature".to_string(),
            description: None,
            input_schema: Some(serde_json::json!({
                "type": "object",
                "required": ["document_url", "signers"],
                "properties": {
                    "document_url": {"type": "string", "format": "uri"},
                    "signers": {
                        "type": "array",
                        "items": {"type": "string", "format": "email"}
                    }
                }
            })),
            output_schema: None,
        };

        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: Capability = serde_json::from_str(&json).unwrap();

        assert!(deserialized.input_schema.is_some());
        assert!(deserialized.output_schema.is_none());
        let schema = deserialized.input_schema.unwrap();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"][0], "document_url");
    }

    #[test]
    fn capability_optional_fields_omitted_when_none() {
        let cap = Capability {
            id: CapabilityId::new(),
            name: "minimal_cap".to_string(),
            description: None,
            input_schema: None,
            output_schema: None,
        };

        let json = serde_json::to_string(&cap).unwrap();
        assert!(!json.contains("\"description\""));
        assert!(!json.contains("\"input_schema\""));
        assert!(!json.contains("\"output_schema\""));
    }

    #[test]
    fn integration_json_roundtrip_with_capabilities() {
        let org_id = OrganizationId::new();
        let mut integration = Integration::new(org_id, "Slack");
        integration.description = Some("Slack integration".to_string());

        integration.capabilities.push(Capability {
            id: CapabilityId::new(),
            name: "send_message".to_string(),
            description: Some("Send a Slack message".to_string()),
            input_schema: Some(serde_json::json!({"type": "object"})),
            output_schema: Some(serde_json::json!({"type": "object"})),
        });
        integration.capabilities.push(Capability {
            id: CapabilityId::new(),
            name: "create_channel".to_string(),
            description: None,
            input_schema: None,
            output_schema: None,
        });

        let json = serde_json::to_string(&integration).unwrap();
        let deserialized: Integration = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, integration.id);
        assert_eq!(deserialized.name, "Slack");
        assert_eq!(deserialized.description, Some("Slack integration".to_string()));
        assert_eq!(deserialized.capabilities.len(), 2);
        assert_eq!(deserialized.capabilities[0].name, "send_message");
        assert_eq!(deserialized.capabilities[1].name, "create_channel");
    }

    #[test]
    fn integration_description_omitted_when_none() {
        let org_id = OrganizationId::new();
        let integration = Integration::new(org_id, "Test");
        let json = serde_json::to_string(&integration).unwrap();
        assert!(!json.contains("\"description\""));
    }
}
