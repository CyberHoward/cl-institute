use inst_constraint::{
    AuthorityLevelCheck, Constraint, ConstraintEngine, EdgeNodeReference, GraphConnectivity,
    NoCyclicDependency, RequiredPolicyScope, Severity, UniqueNames, ValidationContext,
    WorkflowData,
};
use inst_model::*;

/// Helper: build a minimal valid organization.
fn test_org() -> Organization {
    Organization::new("Test University")
}

/// Helper: build a simple valid workflow with start -> intermediate -> end.
fn simple_valid_workflow(org: &Organization) -> WorkflowData {
    let wf = Workflow::new(org.id, "Admissions");
    let start = DecisionNode::new(wf.id, NodeType::Start, "Receive Application");
    let mid = DecisionNode::new(wf.id, NodeType::Intermediate, "Review");
    let end = DecisionNode::new(wf.id, NodeType::End, "Decision Made");

    let edge1 = Edge::new(wf.id, start.id, mid.id);
    let edge2 = Edge::new(wf.id, mid.id, end.id);

    WorkflowData {
        workflow: wf,
        nodes: vec![start, mid, end],
        edges: vec![edge1, edge2],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    }
}

/// Helper: build a ValidationContext from parts.
fn make_ctx(
    org: Organization,
    roles: Vec<OrganizationalRole>,
    workflows: Vec<WorkflowData>,
    policies: Vec<Policy>,
) -> ValidationContext {
    ValidationContext {
        organization: org,
        roles,
        workflows,
        policies,
        integrations: vec![],
    }
}

// -----------------------------------------------------------------------
// ConstraintEngine
// -----------------------------------------------------------------------

#[test]
fn engine_with_defaults_validates_a_clean_model() {
    let org = test_org();
    let wf_data = simple_valid_workflow(&org);
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let engine = ConstraintEngine::with_defaults();
    let result = engine.validate(&ctx);

    assert!(result.is_valid, "Expected valid but got: {:?}", result.violations);
}

#[test]
fn engine_aggregates_violations_from_multiple_constraints() {
    let org = test_org();
    // Empty workflow — no nodes at all.
    let wf = Workflow::new(org.id, "Empty");
    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![],
        edges: vec![],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let engine = ConstraintEngine::with_defaults();
    let result = engine.validate(&ctx);

    assert!(!result.is_valid);
    // Should have at least the "no start node" and "no end node" violations.
    assert!(result.violations.len() >= 2);
}

// -----------------------------------------------------------------------
// GraphConnectivity
// -----------------------------------------------------------------------

#[test]
fn graph_connectivity_passes_on_valid_workflow() {
    let org = test_org();
    let wf_data = simple_valid_workflow(&org);
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let constraint = GraphConnectivity;
    assert!(constraint.validate(&ctx).is_ok());
}

#[test]
fn graph_connectivity_fails_with_no_start_node() {
    let org = test_org();
    let wf = Workflow::new(org.id, "NoStart");
    let mid = DecisionNode::new(wf.id, NodeType::Intermediate, "Review");
    let end = DecisionNode::new(wf.id, NodeType::End, "Done");
    let edge = Edge::new(wf.id, mid.id, end.id);

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![mid, end],
        edges: vec![edge],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let result = GraphConnectivity.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations.iter().any(|v| v.message.contains("no start node")));
}

#[test]
fn graph_connectivity_fails_with_multiple_start_nodes() {
    let org = test_org();
    let wf = Workflow::new(org.id, "TwoStarts");
    let start1 = DecisionNode::new(wf.id, NodeType::Start, "Start 1");
    let start2 = DecisionNode::new(wf.id, NodeType::Start, "Start 2");
    let end = DecisionNode::new(wf.id, NodeType::End, "Done");
    let e1 = Edge::new(wf.id, start1.id, end.id);
    let e2 = Edge::new(wf.id, start2.id, end.id);

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start1, start2, end],
        edges: vec![e1, e2],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let result = GraphConnectivity.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations.iter().any(|v| v.message.contains("2 start nodes")));
}

#[test]
fn graph_connectivity_fails_with_no_end_node() {
    let org = test_org();
    let wf = Workflow::new(org.id, "NoEnd");
    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let mid = DecisionNode::new(wf.id, NodeType::Intermediate, "Review");
    let edge = Edge::new(wf.id, start.id, mid.id);

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, mid],
        edges: vec![edge],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let result = GraphConnectivity.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations.iter().any(|v| v.message.contains("no end node")));
}

#[test]
fn graph_connectivity_fails_with_unreachable_node() {
    let org = test_org();
    let wf = Workflow::new(org.id, "Unreachable");
    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let end = DecisionNode::new(wf.id, NodeType::End, "Done");
    let island = DecisionNode::new(wf.id, NodeType::Intermediate, "Island");
    let edge = Edge::new(wf.id, start.id, end.id);

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, end, island],
        edges: vec![edge],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let result = GraphConnectivity.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations.iter().any(|v| v.message.contains("Island")));
    assert!(violations.iter().any(|v| v.message.contains("not reachable")));
}

// -----------------------------------------------------------------------
// EdgeNodeReference
// -----------------------------------------------------------------------

#[test]
fn edge_node_reference_passes_on_valid_workflow() {
    let org = test_org();
    let wf_data = simple_valid_workflow(&org);
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    assert!(EdgeNodeReference.validate(&ctx).is_ok());
}

#[test]
fn edge_node_reference_fails_on_dangling_from_node() {
    let org = test_org();
    let wf = Workflow::new(org.id, "Dangling");
    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let end = DecisionNode::new(wf.id, NodeType::End, "End");
    let phantom = NodeId::new(); // does not exist in nodes list
    let bad_edge = Edge::new(wf.id, phantom, end.id);
    let good_edge = Edge::new(wf.id, start.id, end.id);

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, end],
        edges: vec![good_edge, bad_edge],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let result = EdgeNodeReference.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.message.contains("non-existent from_node_id")));
}

#[test]
fn edge_node_reference_fails_on_dangling_to_node() {
    let org = test_org();
    let wf = Workflow::new(org.id, "Dangling");
    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let end = DecisionNode::new(wf.id, NodeType::End, "End");
    let phantom = NodeId::new();
    let bad_edge = Edge::new(wf.id, start.id, phantom);
    let good_edge = Edge::new(wf.id, start.id, end.id);

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, end],
        edges: vec![good_edge, bad_edge],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let result = EdgeNodeReference.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.message.contains("non-existent to_node_id")));
}

// -----------------------------------------------------------------------
// AuthorityLevelCheck
// -----------------------------------------------------------------------

#[test]
fn authority_level_check_passes_when_no_authority_required() {
    let org = test_org();
    let wf_data = simple_valid_workflow(&org);
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    assert!(AuthorityLevelCheck.validate(&ctx).is_ok());
}

#[test]
fn authority_level_check_passes_with_sufficient_authority() {
    let org = test_org();
    let wf = Workflow::new(org.id, "WithAuthority");

    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let mut decision = DecisionNode::new(wf.id, NodeType::Intermediate, "High-Stakes Decision");
    decision.requires_authority = 5;
    let end = DecisionNode::new(wf.id, NodeType::End, "Done");

    let edge1 = Edge::new(wf.id, start.id, decision.id);
    let edge2 = Edge::new(wf.id, decision.id, end.id);

    let mut role = OrganizationalRole::new(org.id, "Director");
    role.authority_level = 10;

    let perm = EdgeRolePermission {
        edge_id: edge1.id,
        role_id: role.id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, decision, end],
        edges: vec![edge1, edge2],
        edge_requirements: vec![],
        edge_role_permissions: vec![perm],
    };
    let ctx = make_ctx(org, vec![role], vec![wf_data], vec![]);

    assert!(AuthorityLevelCheck.validate(&ctx).is_ok());
}

#[test]
fn authority_level_check_fails_with_insufficient_authority() {
    let org = test_org();
    let wf = Workflow::new(org.id, "InsufficientAuth");

    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let mut decision = DecisionNode::new(wf.id, NodeType::Intermediate, "High-Stakes Decision");
    decision.requires_authority = 10;
    let end = DecisionNode::new(wf.id, NodeType::End, "Done");

    let edge1 = Edge::new(wf.id, start.id, decision.id);
    let edge2 = Edge::new(wf.id, decision.id, end.id);

    let mut role = OrganizationalRole::new(org.id, "Intern");
    role.authority_level = 1;

    let perm = EdgeRolePermission {
        edge_id: edge1.id,
        role_id: role.id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, decision, end],
        edges: vec![edge1, edge2],
        edge_requirements: vec![],
        edge_role_permissions: vec![perm],
    };
    let ctx = make_ctx(org, vec![role], vec![wf_data], vec![]);

    let result = AuthorityLevelCheck.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.message.contains("requires authority_level 10")));
}

// -----------------------------------------------------------------------
// RequiredPolicyScope
// -----------------------------------------------------------------------

#[test]
fn required_policy_scope_passes_when_no_decision_type() {
    let org = test_org();
    let wf_data = simple_valid_workflow(&org);
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    assert!(RequiredPolicyScope.validate(&ctx).is_ok());
}

#[test]
fn required_policy_scope_passes_with_matching_policy() {
    let org = test_org();
    let wf = Workflow::new(org.id, "WithPolicy");

    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let mut decision = DecisionNode::new(wf.id, NodeType::Intermediate, "Approve Budget");
    decision.decision_type = Some(DecisionType::Approval);
    let end = DecisionNode::new(wf.id, NodeType::End, "Done");

    let edge1 = Edge::new(wf.id, start.id, decision.id);
    let edge2 = Edge::new(wf.id, decision.id, end.id);

    let policy = Policy::new(
        org.id,
        "procurement.approval",
        PolicyStrength::Constraint,
        "All purchases over $10k require VP approval",
    );

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, decision, end],
        edges: vec![edge1, edge2],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![policy]);

    assert!(RequiredPolicyScope.validate(&ctx).is_ok());
}

#[test]
fn required_policy_scope_fails_without_matching_policy() {
    let org = test_org();
    let wf = Workflow::new(org.id, "NoPolicy");

    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let mut decision = DecisionNode::new(wf.id, NodeType::Intermediate, "Classify Document");
    decision.decision_type = Some(DecisionType::Classification);
    let end = DecisionNode::new(wf.id, NodeType::End, "Done");

    let edge1 = Edge::new(wf.id, start.id, decision.id);
    let edge2 = Edge::new(wf.id, decision.id, end.id);

    // Policy exists but with wrong scope.
    let policy = Policy::new(
        org.id,
        "procurement.approval",
        PolicyStrength::Constraint,
        "Approval policy",
    );

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, decision, end],
        edges: vec![edge1, edge2],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![policy]);

    let result = RequiredPolicyScope.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.message.contains("classification")));
}

// -----------------------------------------------------------------------
// NoCyclicDependency
// -----------------------------------------------------------------------

#[test]
fn no_cyclic_dependency_passes_on_acyclic_graph() {
    let org = test_org();
    let wf_data = simple_valid_workflow(&org);
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    assert!(NoCyclicDependency.validate(&ctx).is_ok());
}

#[test]
fn no_cyclic_dependency_warns_on_cycle() {
    let org = test_org();
    let wf = Workflow::new(org.id, "CyclicReview");

    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let review = DecisionNode::new(wf.id, NodeType::Intermediate, "Review");
    let revise = DecisionNode::new(wf.id, NodeType::Intermediate, "Revise");
    let end = DecisionNode::new(wf.id, NodeType::End, "Done");

    let e1 = Edge::new(wf.id, start.id, review.id);
    let e2 = Edge::new(wf.id, review.id, revise.id);
    let e3 = Edge::new(wf.id, revise.id, review.id); // cycle!
    let e4 = Edge::new(wf.id, review.id, end.id);

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, review, revise, end],
        edges: vec![e1, e2, e3, e4],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let result = NoCyclicDependency.validate(&ctx);
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].severity, Severity::Warning);
    assert!(violations[0].message.contains("cycle"));
}

// -----------------------------------------------------------------------
// UniqueNames
// -----------------------------------------------------------------------

#[test]
fn unique_names_passes_with_distinct_names() {
    let org = test_org();
    let role1 = OrganizationalRole::new(org.id, "Admin");
    let role2 = OrganizationalRole::new(org.id, "Reviewer");

    let wf_data = simple_valid_workflow(&org);
    let ctx = make_ctx(org, vec![role1, role2], vec![wf_data], vec![]);

    assert!(UniqueNames.validate(&ctx).is_ok());
}

#[test]
fn unique_names_fails_with_duplicate_role_names() {
    let org = test_org();
    let role1 = OrganizationalRole::new(org.id, "Admin");
    let role2 = OrganizationalRole::new(org.id, "Admin");

    let wf_data = simple_valid_workflow(&org);
    let ctx = make_ctx(org, vec![role1, role2], vec![wf_data], vec![]);

    let result = UniqueNames.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.message.contains("Role name 'Admin' is duplicated")));
}

#[test]
fn unique_names_fails_with_duplicate_workflow_names_in_same_function() {
    let org = test_org();
    let func = Function::new(org.id, "Procurement");

    let mut wf1 = Workflow::new(org.id, "Vendor Selection");
    wf1.function_id = Some(func.id);
    let mut wf2 = Workflow::new(org.id, "Vendor Selection");
    wf2.function_id = Some(func.id);

    let make_minimal_wf = |wf: Workflow| -> WorkflowData {
        let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
        let end = DecisionNode::new(wf.id, NodeType::End, "End");
        let edge = Edge::new(wf.id, start.id, end.id);
        WorkflowData {
            workflow: wf,
            nodes: vec![start, end],
            edges: vec![edge],
            edge_requirements: vec![],
            edge_role_permissions: vec![],
        }
    };

    let ctx = make_ctx(
        org,
        vec![],
        vec![make_minimal_wf(wf1), make_minimal_wf(wf2)],
        vec![],
    );

    let result = UniqueNames.validate(&ctx);
    let violations = result.unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.message.contains("Workflow name 'Vendor Selection' is duplicated")));
}

#[test]
fn unique_names_allows_same_workflow_name_in_different_functions() {
    let org = test_org();
    let func1 = Function::new(org.id, "Procurement");
    let func2 = Function::new(org.id, "HR");

    let mut wf1 = Workflow::new(org.id, "Approval");
    wf1.function_id = Some(func1.id);
    let mut wf2 = Workflow::new(org.id, "Approval");
    wf2.function_id = Some(func2.id);

    let make_minimal_wf = |wf: Workflow| -> WorkflowData {
        let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
        let end = DecisionNode::new(wf.id, NodeType::End, "End");
        let edge = Edge::new(wf.id, start.id, end.id);
        WorkflowData {
            workflow: wf,
            nodes: vec![start, end],
            edges: vec![edge],
            edge_requirements: vec![],
            edge_role_permissions: vec![],
        }
    };

    let ctx = make_ctx(
        org,
        vec![],
        vec![make_minimal_wf(wf1), make_minimal_wf(wf2)],
        vec![],
    );

    assert!(UniqueNames.validate(&ctx).is_ok());
}

// -----------------------------------------------------------------------
// ValidationResult
// -----------------------------------------------------------------------

#[test]
fn validation_result_errors_and_warnings_filter_correctly() {
    let org = test_org();
    let wf = Workflow::new(org.id, "CyclicReview");

    // Build a workflow that has a cycle (warning) AND is missing an end node (error).
    let start = DecisionNode::new(wf.id, NodeType::Start, "Start");
    let review = DecisionNode::new(wf.id, NodeType::Intermediate, "Review");
    let revise = DecisionNode::new(wf.id, NodeType::Intermediate, "Revise");

    let e1 = Edge::new(wf.id, start.id, review.id);
    let e2 = Edge::new(wf.id, review.id, revise.id);
    let e3 = Edge::new(wf.id, revise.id, review.id); // cycle

    let wf_data = WorkflowData {
        workflow: wf,
        nodes: vec![start, review, revise],
        edges: vec![e1, e2, e3],
        edge_requirements: vec![],
        edge_role_permissions: vec![],
    };
    let ctx = make_ctx(org, vec![], vec![wf_data], vec![]);

    let engine = ConstraintEngine::with_defaults();
    let result = engine.validate(&ctx);

    assert!(!result.is_valid);
    assert!(!result.errors().is_empty());
    assert!(!result.warnings().is_empty());
}
