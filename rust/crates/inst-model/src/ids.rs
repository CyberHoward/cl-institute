use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }
    };
}

define_id!(OrganizationId);
define_id!(RoleId);
define_id!(MemberId);
define_id!(FunctionId);
define_id!(WorkflowId);
define_id!(NodeId);
define_id!(EdgeId);
define_id!(RequirementId);
define_id!(PolicyId);
define_id!(IntegrationId);
define_id!(CapabilityId);
define_id!(InstanceId);
define_id!(VersionId);
define_id!(SubmissionId);
define_id!(UserId);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn new_ids_are_unique() {
        let id1 = OrganizationId::new();
        let id2 = OrganizationId::new();
        let id3 = OrganizationId::new();
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn id_from_existing_uuid() {
        let uuid = Uuid::new_v4();
        let id = OrganizationId::from_uuid(uuid);
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn id_from_uuid_via_into() {
        let uuid = Uuid::new_v4();
        let id: WorkflowId = uuid.into();
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn id_serializes_as_uuid_string_in_json() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let id = NodeId::from_uuid(uuid);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"550e8400-e29b-41d4-a716-446655440000\"");
    }

    #[test]
    fn id_deserializes_from_uuid_string() {
        let json = "\"550e8400-e29b-41d4-a716-446655440000\"";
        let id: NodeId = serde_json::from_str(json).unwrap();
        assert_eq!(
            id.0,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn id_roundtrips_through_json() {
        let id = PolicyId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: PolicyId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn id_display_matches_uuid_display() {
        let uuid = Uuid::new_v4();
        let id = EdgeId::from_uuid(uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn id_implements_hash_correctly() {
        let uuid = Uuid::new_v4();
        let id1 = RoleId::from_uuid(uuid);
        let id2 = RoleId::from_uuid(uuid);

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2));
    }

    #[test]
    fn id_eq_same_uuid() {
        let uuid = Uuid::new_v4();
        let id1 = MemberId::from_uuid(uuid);
        let id2 = MemberId::from_uuid(uuid);
        assert_eq!(id1, id2);
    }

    #[test]
    fn id_ne_different_uuid() {
        let id1 = MemberId::new();
        let id2 = MemberId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn id_default_creates_new() {
        let id1 = FunctionId::default();
        let id2 = FunctionId::default();
        // Default should create new unique IDs each time
        assert_ne!(id1, id2);
    }

    #[test]
    fn id_copy_semantics() {
        let id = InstanceId::new();
        let copied = id;
        // Both should be usable (Copy trait)
        assert_eq!(id, copied);
    }

    #[test]
    fn all_id_types_serialize_consistently() {
        // Ensure all defined ID types serialize as plain UUID strings
        let uuid = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8").unwrap();

        let expected = "\"a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8\"";

        assert_eq!(serde_json::to_string(&OrganizationId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&RoleId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&MemberId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&FunctionId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&WorkflowId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&NodeId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&EdgeId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&RequirementId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&PolicyId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&IntegrationId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&CapabilityId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&InstanceId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&VersionId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&SubmissionId::from_uuid(uuid)).unwrap(), expected);
        assert_eq!(serde_json::to_string(&UserId::from_uuid(uuid)).unwrap(), expected);
    }
}
