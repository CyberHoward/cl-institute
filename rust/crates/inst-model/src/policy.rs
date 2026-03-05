use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{OrganizationId, PolicyId};

/// The strength/formality level of a policy.
///
/// From the architecture's Formality Spectrum (Section 2.1):
/// - Constraint: Hard rules, legal/regulatory mandates (machine-enforced)
/// - Procedure: Defined steps with authorized deviation (executed deterministically)
/// - Preference: Intent-bearing guidance (LLM-interpreted at decision time)
/// - Context: Tacit knowledge, institutional culture (advisory only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStrength {
    Constraint,
    Procedure,
    Preference,
    Context,
}

/// A policy attached to a scope within the institutional model.
///
/// Policies are the bridge between formal rules and natural language guidance.
/// They are scoped to ontological structures (e.g., "procurement.*",
/// "procurement.vendor-selection") and retrieved by scope, not similarity search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: PolicyId,
    pub organization_id: OrganizationId,
    /// Dot-separated scope path (e.g., "procurement.vendor-selection").
    /// Policies are attached to scopes and retrieved hierarchically.
    pub scope: String,
    pub strength: PolicyStrength,
    /// The policy text — structured natural language with semantic intent.
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Policy {
    pub fn new(
        organization_id: OrganizationId,
        scope: impl Into<String>,
        strength: PolicyStrength,
        text: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: PolicyId::new(),
            organization_id,
            scope: scope.into(),
            strength,
            text: text.into(),
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns true if this policy's scope matches or is a parent of the given scope.
    pub fn matches_scope(&self, query_scope: &str) -> bool {
        query_scope == self.scope || query_scope.starts_with(&format!("{}.", self.scope))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_new_constructor() {
        let org_id = OrganizationId::new();
        let policy = Policy::new(org_id, "procurement", PolicyStrength::Constraint, "All purchases over $5000 need CFO approval");

        assert_eq!(policy.organization_id, org_id);
        assert_eq!(policy.scope, "procurement");
        assert_eq!(policy.strength, PolicyStrength::Constraint);
        assert_eq!(policy.text, "All purchases over $5000 need CFO approval");
        assert!(policy.metadata.is_none());
        assert_eq!(policy.created_at, policy.updated_at);
    }

    #[test]
    fn policy_unique_ids() {
        let org_id = OrganizationId::new();
        let p1 = Policy::new(org_id, "scope", PolicyStrength::Context, "text");
        let p2 = Policy::new(org_id, "scope", PolicyStrength::Context, "text");
        assert_ne!(p1.id, p2.id);
    }

    #[test]
    fn policy_strength_constraint_serializes() {
        let json = serde_json::to_string(&PolicyStrength::Constraint).unwrap();
        assert_eq!(json, "\"constraint\"");
        let ps: PolicyStrength = serde_json::from_str(&json).unwrap();
        assert_eq!(ps, PolicyStrength::Constraint);
    }

    #[test]
    fn policy_strength_procedure_serializes() {
        let json = serde_json::to_string(&PolicyStrength::Procedure).unwrap();
        assert_eq!(json, "\"procedure\"");
        let ps: PolicyStrength = serde_json::from_str(&json).unwrap();
        assert_eq!(ps, PolicyStrength::Procedure);
    }

    #[test]
    fn policy_strength_preference_serializes() {
        let json = serde_json::to_string(&PolicyStrength::Preference).unwrap();
        assert_eq!(json, "\"preference\"");
        let ps: PolicyStrength = serde_json::from_str(&json).unwrap();
        assert_eq!(ps, PolicyStrength::Preference);
    }

    #[test]
    fn policy_strength_context_serializes() {
        let json = serde_json::to_string(&PolicyStrength::Context).unwrap();
        assert_eq!(json, "\"context\"");
        let ps: PolicyStrength = serde_json::from_str(&json).unwrap();
        assert_eq!(ps, PolicyStrength::Context);
    }

    #[test]
    fn matches_scope_exact_match() {
        let org_id = OrganizationId::new();
        let policy = Policy::new(org_id, "procurement", PolicyStrength::Constraint, "text");
        assert!(policy.matches_scope("procurement"));
    }

    #[test]
    fn matches_scope_parent_scope_match() {
        let org_id = OrganizationId::new();
        let policy = Policy::new(org_id, "procurement", PolicyStrength::Constraint, "text");
        assert!(policy.matches_scope("procurement.vendor-selection"));
        assert!(policy.matches_scope("procurement.vendor-selection.evaluation"));
    }

    #[test]
    fn matches_scope_non_match() {
        let org_id = OrganizationId::new();
        let policy = Policy::new(org_id, "procurement", PolicyStrength::Constraint, "text");
        assert!(!policy.matches_scope("hr"));
        assert!(!policy.matches_scope("hr.onboarding"));
    }

    #[test]
    fn matches_scope_partial_name_does_not_match() {
        let org_id = OrganizationId::new();
        let policy = Policy::new(org_id, "proc", PolicyStrength::Constraint, "text");
        // "procurement" starts with "proc" but not "proc." so it should not match
        assert!(!policy.matches_scope("procurement"));
    }

    #[test]
    fn metadata_omitted_when_none() {
        let org_id = OrganizationId::new();
        let policy = Policy::new(org_id, "scope", PolicyStrength::Context, "text");
        let json = serde_json::to_string(&policy).unwrap();
        assert!(!json.contains("metadata"));
    }

    #[test]
    fn metadata_present_when_some() {
        let org_id = OrganizationId::new();
        let mut policy = Policy::new(org_id, "scope", PolicyStrength::Preference, "text");
        policy.metadata = Some(serde_json::json!({"source": "board_resolution", "date": "2024-01-15"}));

        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: Policy = serde_json::from_str(&json).unwrap();

        assert!(deserialized.metadata.is_some());
        assert_eq!(deserialized.metadata.unwrap()["source"], "board_resolution");
    }

    #[test]
    fn policy_json_roundtrip() {
        let org_id = OrganizationId::new();
        let mut policy = Policy::new(
            org_id,
            "procurement.vendor-selection",
            PolicyStrength::Procedure,
            "Vendors must be evaluated using the standard rubric",
        );
        policy.metadata = Some(serde_json::json!({"version": 2}));

        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: Policy = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, policy.id);
        assert_eq!(deserialized.scope, policy.scope);
        assert_eq!(deserialized.strength, policy.strength);
        assert_eq!(deserialized.text, policy.text);
        assert_eq!(deserialized.metadata, policy.metadata);
    }
}
