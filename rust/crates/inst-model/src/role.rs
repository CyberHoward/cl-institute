use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{MemberId, OrganizationId, RoleId, UserId};

/// Permission level for organization membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionLevel {
    Owner,
    Admin,
    Member,
    Viewer,
}

/// A member of an organization (user + permission level).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMember {
    pub id: MemberId,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub permission_level: PermissionLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An institutional role within an organization (e.g., "compliance-officer").
///
/// These are distinct from permission levels — they represent domain-specific
/// authority (who can approve procurement, who reviews compliance, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalRole {
    pub id: RoleId,
    pub organization_id: OrganizationId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Authority level (0 = lowest). Used by the constraint engine
    /// to validate that decision makers have sufficient authority.
    #[serde(default)]
    pub authority_level: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OrganizationalRole {
    pub fn new(organization_id: OrganizationId, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: RoleId::new(),
            organization_id,
            name: name.into(),
            description: None,
            authority_level: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Assignment of an organizational role to a member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRoleAssignment {
    pub member_id: MemberId,
    pub role_id: RoleId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn organizational_role_new_defaults() {
        let org_id = OrganizationId::new();
        let role = OrganizationalRole::new(org_id, "compliance-officer");

        assert_eq!(role.organization_id, org_id);
        assert_eq!(role.name, "compliance-officer");
        assert!(role.description.is_none());
        assert_eq!(role.authority_level, 0);
        assert_eq!(role.created_at, role.updated_at);
    }

    #[test]
    fn organizational_role_unique_ids() {
        let org_id = OrganizationId::new();
        let role1 = OrganizationalRole::new(org_id, "Role A");
        let role2 = OrganizationalRole::new(org_id, "Role B");
        assert_ne!(role1.id, role2.id);
    }

    #[test]
    fn permission_level_serializes_as_snake_case() {
        assert_eq!(serde_json::to_string(&PermissionLevel::Owner).unwrap(), "\"owner\"");
        assert_eq!(serde_json::to_string(&PermissionLevel::Admin).unwrap(), "\"admin\"");
        assert_eq!(serde_json::to_string(&PermissionLevel::Member).unwrap(), "\"member\"");
        assert_eq!(serde_json::to_string(&PermissionLevel::Viewer).unwrap(), "\"viewer\"");
    }

    #[test]
    fn permission_level_roundtrip_owner() {
        let json = serde_json::to_string(&PermissionLevel::Owner).unwrap();
        let deserialized: PermissionLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, PermissionLevel::Owner);
    }

    #[test]
    fn permission_level_roundtrip_admin() {
        let json = serde_json::to_string(&PermissionLevel::Admin).unwrap();
        let deserialized: PermissionLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, PermissionLevel::Admin);
    }

    #[test]
    fn permission_level_roundtrip_member() {
        let json = serde_json::to_string(&PermissionLevel::Member).unwrap();
        let deserialized: PermissionLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, PermissionLevel::Member);
    }

    #[test]
    fn permission_level_roundtrip_viewer() {
        let json = serde_json::to_string(&PermissionLevel::Viewer).unwrap();
        let deserialized: PermissionLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, PermissionLevel::Viewer);
    }

    #[test]
    fn authority_level_defaults_to_zero() {
        let json = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "organization_id": "550e8400-e29b-41d4-a716-446655440001",
            "name": "test-role",
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T00:00:00Z"
        });
        let role: OrganizationalRole = serde_json::from_value(json).unwrap();
        assert_eq!(role.authority_level, 0);
    }

    #[test]
    fn role_description_omitted_when_none() {
        let org_id = OrganizationId::new();
        let role = OrganizationalRole::new(org_id, "test-role");
        let json = serde_json::to_string(&role).unwrap();
        assert!(!json.contains("description"));
    }

    #[test]
    fn role_json_roundtrip() {
        let org_id = OrganizationId::new();
        let mut role = OrganizationalRole::new(org_id, "budget-approver");
        role.description = Some("Approves budget items".to_string());
        role.authority_level = 3;

        let json = serde_json::to_string(&role).unwrap();
        let deserialized: OrganizationalRole = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, role.id);
        assert_eq!(deserialized.organization_id, role.organization_id);
        assert_eq!(deserialized.name, role.name);
        assert_eq!(deserialized.description, role.description);
        assert_eq!(deserialized.authority_level, role.authority_level);
    }
}
