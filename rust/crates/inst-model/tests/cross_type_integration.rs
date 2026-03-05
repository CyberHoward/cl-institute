//! Cross-type integration tests for the inst-model crate.
//!
//! These tests verify that all core types work together correctly,
//! that a complete mini institution can be constructed, serialized,
//! and deserialized with all ID references remaining consistent.

use inst_model::*;
use serde::{Deserialize, Serialize};

/// A complete mini institution bundling all core types for serialization testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MiniInstitution {
    organization: Organization,
    roles: Vec<OrganizationalRole>,
    workflow: Workflow,
    nodes: Vec<DecisionNode>,
    edges: Vec<Edge>,
    policies: Vec<Policy>,
    integration: Integration,
}

fn build_mini_institution() -> MiniInstitution {
    // 1. Organization
    let mut org = Organization::new("Acme University");
    org.description = Some("A test institution for integration testing".to_string());
    org.rules.insert(
        "procurement_limit".to_string(),
        "All purchases over $10,000 require board approval".to_string(),
    );

    let org_id = org.id;

    // 2. Roles
    let mut compliance_officer = OrganizationalRole::new(org_id, "compliance-officer");
    compliance_officer.description = Some("Ensures regulatory compliance".to_string());
    compliance_officer.authority_level = 3;

    let mut budget_approver = OrganizationalRole::new(org_id, "budget-approver");
    budget_approver.authority_level = 2;

    // 3. Workflow
    let mut workflow = Workflow::new(org_id, "Procurement Process");
    workflow.description = Some("Standard procurement workflow".to_string());
    let wf_id = workflow.id;

    // 4. Nodes
    let mut start_node = DecisionNode::new(wf_id, NodeType::Start, "Submit Request");
    start_node.x = 100.0;
    start_node.y = 50.0;

    let mut review_node = DecisionNode::new(wf_id, NodeType::Intermediate, "Budget Review");
    review_node.decision_type = Some(DecisionType::Approval);
    review_node.requires_authority = 2;
    review_node.x = 300.0;
    review_node.y = 50.0;
    review_node.index = 1;

    let mut compliance_node =
        DecisionNode::new(wf_id, NodeType::Intermediate, "Compliance Check");
    compliance_node.decision_type = Some(DecisionType::Classification);
    compliance_node.requires_authority = 3;
    compliance_node.x = 500.0;
    compliance_node.y = 50.0;
    compliance_node.index = 2;

    let mut end_node = DecisionNode::new(wf_id, NodeType::End, "Order Placed");
    end_node.x = 700.0;
    end_node.y = 50.0;
    end_node.index = 3;

    // 5. Edges
    let mut edge1 = Edge::new(wf_id, start_node.id, review_node.id);
    edge1.label = Some("Submit for review".to_string());

    let mut edge2 = Edge::new(wf_id, review_node.id, compliance_node.id);
    edge2.label = Some("Budget approved".to_string());
    edge2.rule = Some("Budget approval from authorized approver".to_string());

    let mut edge3 = Edge::new(wf_id, compliance_node.id, end_node.id);
    edge3.label = Some("Compliance verified".to_string());

    // 6. Policies
    let policy1 = Policy::new(
        org_id,
        "procurement",
        PolicyStrength::Constraint,
        "All vendor contracts must be reviewed by legal before signing",
    );
    let mut policy2 = Policy::new(
        org_id,
        "procurement.vendor-selection",
        PolicyStrength::Preference,
        "Prefer vendors with sustainability certifications",
    );
    policy2.metadata = Some(serde_json::json!({"source": "sustainability_committee", "year": 2024}));

    // 7. Integration
    let mut integration = Integration::new(org_id, "SAP");
    integration.description = Some("SAP ERP integration".to_string());
    integration.capabilities.push(Capability {
        id: CapabilityId::new(),
        name: "create_purchase_order".to_string(),
        description: Some("Creates a purchase order in SAP".to_string()),
        input_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "vendor_id": {"type": "string"},
                "line_items": {"type": "array"}
            }
        })),
        output_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "po_number": {"type": "string"}
            }
        })),
    });

    MiniInstitution {
        organization: org,
        roles: vec![compliance_officer, budget_approver],
        workflow,
        nodes: vec![start_node, review_node, compliance_node, end_node],
        edges: vec![edge1, edge2, edge3],
        policies: vec![policy1, policy2],
        integration,
    }
}

#[test]
fn create_complete_mini_institution() {
    let inst = build_mini_institution();

    // Verify structure
    assert_eq!(inst.organization.name, "Acme University");
    assert_eq!(inst.roles.len(), 2);
    assert_eq!(inst.nodes.len(), 4);
    assert_eq!(inst.edges.len(), 3);
    assert_eq!(inst.policies.len(), 2);
    assert_eq!(inst.integration.capabilities.len(), 1);
}

#[test]
fn serialize_and_deserialize_entire_institution_json() {
    let inst = build_mini_institution();

    let json = serde_json::to_string_pretty(&inst).unwrap();
    let deserialized: MiniInstitution = serde_json::from_str(&json).unwrap();

    // Verify organization
    assert_eq!(deserialized.organization.id, inst.organization.id);
    assert_eq!(deserialized.organization.name, inst.organization.name);
    assert_eq!(
        deserialized.organization.description,
        inst.organization.description
    );
    assert_eq!(deserialized.organization.rules, inst.organization.rules);

    // Verify roles
    assert_eq!(deserialized.roles.len(), 2);
    assert_eq!(deserialized.roles[0].name, "compliance-officer");
    assert_eq!(deserialized.roles[0].authority_level, 3);
    assert_eq!(deserialized.roles[1].name, "budget-approver");
    assert_eq!(deserialized.roles[1].authority_level, 2);

    // Verify workflow
    assert_eq!(deserialized.workflow.id, inst.workflow.id);
    assert_eq!(deserialized.workflow.name, "Procurement Process");

    // Verify nodes
    assert_eq!(deserialized.nodes.len(), 4);
    assert_eq!(deserialized.nodes[0].node_type, NodeType::Start);
    assert_eq!(deserialized.nodes[3].node_type, NodeType::End);

    // Verify edges
    assert_eq!(deserialized.edges.len(), 3);

    // Verify policies
    assert_eq!(deserialized.policies.len(), 2);
    assert_eq!(deserialized.policies[0].strength, PolicyStrength::Constraint);
    assert_eq!(deserialized.policies[1].strength, PolicyStrength::Preference);

    // Verify integration
    assert_eq!(deserialized.integration.name, "SAP");
    assert_eq!(deserialized.integration.capabilities.len(), 1);
}

#[test]
fn id_references_are_consistent_after_roundtrip() {
    let inst = build_mini_institution();
    let org_id = inst.organization.id;

    let json = serde_json::to_string(&inst).unwrap();
    let deserialized: MiniInstitution = serde_json::from_str(&json).unwrap();

    // All roles reference the same organization
    for role in &deserialized.roles {
        assert_eq!(
            role.organization_id, org_id,
            "Role '{}' has wrong organization_id",
            role.name
        );
    }

    // Workflow references the organization
    assert_eq!(deserialized.workflow.organization_id, org_id);

    let wf_id = deserialized.workflow.id;

    // All nodes reference the workflow
    for node in &deserialized.nodes {
        assert_eq!(
            node.workflow_id, wf_id,
            "Node '{}' has wrong workflow_id",
            node.label
        );
    }

    // All edges reference the workflow
    for edge in &deserialized.edges {
        assert_eq!(edge.workflow_id, wf_id);
    }

    // All policies reference the organization
    for policy in &deserialized.policies {
        assert_eq!(policy.organization_id, org_id);
    }

    // Integration references the organization
    assert_eq!(deserialized.integration.organization_id, org_id);

    // Edge references point to valid nodes
    let node_ids: Vec<_> = deserialized.nodes.iter().map(|n| n.id).collect();
    for edge in &deserialized.edges {
        assert!(
            node_ids.contains(&edge.from_node_id),
            "Edge from_node_id {:?} not found in nodes",
            edge.from_node_id
        );
        assert!(
            node_ids.contains(&edge.to_node_id),
            "Edge to_node_id {:?} not found in nodes",
            edge.to_node_id
        );
    }
}

#[test]
fn edge_connectivity_matches_node_graph() {
    let inst = build_mini_institution();

    // Verify the graph structure: Start -> Review -> Compliance -> End
    let start = &inst.nodes[0];
    let review = &inst.nodes[1];
    let compliance = &inst.nodes[2];
    let end = &inst.nodes[3];

    assert_eq!(inst.edges[0].from_node_id, start.id);
    assert_eq!(inst.edges[0].to_node_id, review.id);

    assert_eq!(inst.edges[1].from_node_id, review.id);
    assert_eq!(inst.edges[1].to_node_id, compliance.id);

    assert_eq!(inst.edges[2].from_node_id, compliance.id);
    assert_eq!(inst.edges[2].to_node_id, end.id);
}

#[test]
fn policies_match_expected_scopes() {
    let inst = build_mini_institution();

    let procurement_policy = &inst.policies[0];
    let vendor_policy = &inst.policies[1];

    // The broad procurement policy should match both scopes
    assert!(procurement_policy.matches_scope("procurement"));
    assert!(procurement_policy.matches_scope("procurement.vendor-selection"));

    // The vendor-selection policy should only match its own scope and children
    assert!(vendor_policy.matches_scope("procurement.vendor-selection"));
    assert!(vendor_policy.matches_scope("procurement.vendor-selection.evaluation"));
    assert!(!vendor_policy.matches_scope("procurement"));
}

#[test]
fn workflow_instance_can_reference_institution_entities() {
    let inst = build_mini_institution();

    // Create an instance of the workflow
    let mut instance = WorkflowInstance::new(inst.workflow.id, "Test Applicant");

    // Set current node to the first node (Start)
    instance.current_node_id = Some(inst.nodes[0].id);

    // Add a trace entry referencing an edge transition
    instance.trace.push(serde_json::json!({
        "action": "started",
        "node_id": inst.nodes[0].id.to_string(),
    }));

    let json = serde_json::to_string(&instance).unwrap();
    let deserialized: WorkflowInstance = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.workflow_id, inst.workflow.id);
    assert_eq!(deserialized.current_node_id, Some(inst.nodes[0].id));
    assert_eq!(deserialized.status, InstanceStatus::InProgress);
    assert_eq!(deserialized.trace.len(), 1);
}

#[test]
fn all_optional_fields_omitted_in_minimal_institution() {
    // Create a minimal institution with no optional fields set
    let org = Organization::new("Minimal Org");
    let org_id = org.id;
    let role = OrganizationalRole::new(org_id, "basic-role");
    let workflow = Workflow::new(org_id, "Basic Workflow");
    let node = DecisionNode::new(workflow.id, NodeType::Start, "Start");
    let edge = Edge::new(workflow.id, node.id, node.id);
    let policy = Policy::new(org_id, "general", PolicyStrength::Context, "Be nice");
    let integration = Integration::new(org_id, "Minimal Integration");

    let inst = MiniInstitution {
        organization: org,
        roles: vec![role],
        workflow,
        nodes: vec![node],
        edges: vec![edge],
        policies: vec![policy],
        integration,
    };

    let json = serde_json::to_string(&inst).unwrap();

    // Verify that optional fields are not present in the JSON
    // Organization description
    assert!(!json.contains("\"description\":null"));
    // function_id on workflow
    assert!(!json.contains("\"function_id\":null"));
    // metadata on policy
    assert!(!json.contains("\"metadata\":null"));

    // But the JSON should still be deserializable
    let deserialized: MiniInstitution = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.organization.name, "Minimal Org");
    assert!(deserialized.organization.description.is_none());
    assert!(deserialized.workflow.function_id.is_none());
    assert!(deserialized.policies[0].metadata.is_none());
}

#[test]
fn large_institution_with_many_entities() {
    let org = Organization::new("Large Org");
    let org_id = org.id;
    let wf = Workflow::new(org_id, "Complex Workflow");
    let wf_id = wf.id;

    // Create 10 nodes
    let mut nodes = Vec::new();
    nodes.push(DecisionNode::new(wf_id, NodeType::Start, "Start"));
    for i in 1..9 {
        let mut node = DecisionNode::new(
            wf_id,
            NodeType::Intermediate,
            format!("Step {}", i),
        );
        node.index = i as i32;
        nodes.push(node);
    }
    nodes.push(DecisionNode::new(wf_id, NodeType::End, "End"));

    // Create edges connecting them linearly
    let mut edges = Vec::new();
    for i in 0..nodes.len() - 1 {
        let mut edge = Edge::new(wf_id, nodes[i].id, nodes[i + 1].id);
        edge.label = Some(format!("Step {} to {}", i, i + 1));
        edges.push(edge);
    }

    // Create 5 policies
    let policies: Vec<Policy> = (0..5)
        .map(|i| {
            Policy::new(
                org_id,
                format!("scope.level{}", i),
                PolicyStrength::Preference,
                format!("Policy text {}", i),
            )
        })
        .collect();

    let inst = MiniInstitution {
        organization: org,
        roles: vec![],
        workflow: wf,
        nodes,
        edges,
        policies,
        integration: Integration::new(org_id, "None"),
    };

    let json = serde_json::to_string(&inst).unwrap();
    let deserialized: MiniInstitution = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.nodes.len(), 10);
    assert_eq!(deserialized.edges.len(), 9);
    assert_eq!(deserialized.policies.len(), 5);

    // Verify all edge connections are preserved
    for i in 0..deserialized.edges.len() {
        assert_eq!(deserialized.edges[i].from_node_id, inst.nodes[i].id);
        assert_eq!(deserialized.edges[i].to_node_id, inst.nodes[i + 1].id);
    }
}
