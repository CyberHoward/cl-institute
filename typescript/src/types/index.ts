/**
 * Type definitions for the Intelligent Institution model.
 *
 * These interfaces mirror the Rust model types from `inst-model` and related crates.
 * They represent the JSON serialization format produced by the `inst` CLI.
 * All UUIDs are represented as strings in JSON output.
 * All timestamps are ISO-8601 strings.
 */

// ---------------------------------------------------------------------------
// Organization
// ---------------------------------------------------------------------------

/** Top-level institutional entity. */
export interface Organization {
  id: string;
  name: string;
  description?: string | undefined;
  /** Named rules: key = rule name, value = rule description text. */
  rules: Record<string, string>;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Roles
// ---------------------------------------------------------------------------

/** Permission level for organization membership. */
export type PermissionLevel = "owner" | "admin" | "member" | "viewer";

/** A member of an organization (user + permission level). */
export interface OrganizationMember {
  id: string;
  organization_id: string;
  user_id: string;
  permission_level: PermissionLevel;
  created_at: string;
  updated_at: string;
}

/**
 * An institutional role within an organization (e.g., "compliance-officer").
 *
 * These are distinct from permission levels -- they represent domain-specific
 * authority (who can approve procurement, who reviews compliance, etc.).
 */
export interface OrganizationalRole {
  id: string;
  organization_id: string;
  name: string;
  description?: string | undefined;
  /** Authority level (0 = lowest). Used by the constraint engine. */
  authority_level: number;
  created_at: string;
  updated_at: string;
}

/** Assignment of an organizational role to a member. */
export interface MemberRoleAssignment {
  member_id: string;
  role_id: string;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Functions & Workflows
// ---------------------------------------------------------------------------

/**
 * A functional grouping of related workflows within an organization.
 * Example: "Procurement", "Compliance", "HR Onboarding".
 */
export interface Function {
  id: string;
  organization_id: string;
  name: string;
  description?: string | undefined;
  created_at: string;
  updated_at: string;
}

/**
 * A workflow definition -- a graph of decision nodes connected by edges.
 *
 * The workflow is the central organizational unit: it contains nodes (decision points),
 * edges (transitions), and attached policies.
 */
export interface Workflow {
  id: string;
  organization_id: string;
  function_id?: string | undefined;
  name: string;
  description?: string | undefined;
  created_at: string;
  updated_at: string;
}

/** A snapshot of a workflow at a point in time, for version history. */
export interface WorkflowVersion {
  id: string;
  workflow_id: string;
  version: number;
  change_reason: string;
  /** JSON snapshot of the full workflow graph (nodes + edges + policies). */
  snapshot: unknown;
  created_by: string;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Decision Nodes
// ---------------------------------------------------------------------------

/** The type of a node in the decision graph. */
export type NodeType = "start" | "intermediate" | "end";

/**
 * The category of judgment required at a decision point.
 *
 * From the architecture doc's Decision Point Ontology (Section 2.2).
 */
export type DecisionType =
  | "approval"
  | "classification"
  | "prioritization"
  | "allocation"
  | "exception_handling";

/**
 * A node in the workflow decision graph -- the atomic unit of the system.
 *
 * In the architecture, this is the "judgment point" where institutional
 * decisions require context, policy, precedent, and discretion.
 */
export interface DecisionNode {
  id: string;
  workflow_id: string;
  node_type: NodeType;
  label: string;
  /** Ordering index within the workflow. */
  index: number;
  /** Visual position X coordinate. */
  x: number;
  /** Visual position Y coordinate. */
  y: number;
  /** The category of decision at this node (if applicable). */
  decision_type?: DecisionType | undefined;
  /** Minimum authority level required to make this decision. */
  requires_authority: number;
  /** The schema for what this decision produces. */
  output_schema?: string | undefined;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Edges
// ---------------------------------------------------------------------------

/**
 * An edge connecting two decision nodes in the workflow graph.
 *
 * Edges describe *what* needs to happen between decisions (intent-level),
 * not *how* (implementation). The `rule` field is the edge specification
 * that gets compiled to automations by the integration compiler.
 */
export interface Edge {
  id: string;
  workflow_id: string;
  from_node_id: string;
  to_node_id: string;
  label?: string | undefined;
  /**
   * The edge specification -- natural language description of what
   * must happen for this transition. Compiled to automation by the
   * integration compiler.
   */
  rule?: string | undefined;
  created_at: string;
  updated_at: string;
}

/** The type of requirement attached to an edge transition. */
export type RequirementType = "document" | "approval";

/**
 * A requirement that must be satisfied for an edge transition.
 * Examples: "Upload signed contract", "Budget approval from CFO".
 */
export interface EdgeRequirement {
  id: string;
  edge_id: string;
  type: RequirementType;
  label: string;
  description?: string | undefined;
  config?: unknown | undefined;
  is_optional: boolean;
  created_at: string;
  updated_at: string;
}

/** Which organizational roles are authorized to execute a transition. */
export interface EdgeRolePermission {
  edge_id: string;
  role_id: string;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Policies
// ---------------------------------------------------------------------------

/**
 * The strength/formality level of a policy.
 *
 * From the architecture's Formality Spectrum (Section 2.1):
 * - constraint: Hard rules, legal/regulatory mandates (machine-enforced)
 * - procedure: Defined steps with authorized deviation (executed deterministically)
 * - preference: Intent-bearing guidance (LLM-interpreted at decision time)
 * - context: Tacit knowledge, institutional culture (advisory only)
 */
export type PolicyStrength = "constraint" | "procedure" | "preference" | "context";

/**
 * A policy attached to a scope within the institutional model.
 *
 * Policies are the bridge between formal rules and natural language guidance.
 * They are scoped to ontological structures (e.g., "procurement.*",
 * "procurement.vendor-selection") and retrieved by scope, not similarity search.
 */
export interface Policy {
  id: string;
  organization_id: string;
  /** Dot-separated scope path (e.g., "procurement.vendor-selection"). */
  scope: string;
  strength: PolicyStrength;
  /** The policy text -- structured natural language with semantic intent. */
  text: string;
  metadata?: unknown | undefined;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Integrations
// ---------------------------------------------------------------------------

/**
 * An external integration available to the institution.
 *
 * Each integration exposes capabilities with defined input/output schemas.
 * The registry does NOT contain credentials -- those are environment config.
 */
export interface Integration {
  id: string;
  organization_id: string;
  name: string;
  description?: string | undefined;
  capabilities: Capability[];
  created_at: string;
  updated_at: string;
}

/**
 * A capability exposed by an integration.
 * Examples: DocuSign -> `route_for_signature`, SAP -> `create_purchase_order`.
 */
export interface Capability {
  id: string;
  name: string;
  description?: string | undefined;
  /** JSON Schema for the capability's input. */
  input_schema?: unknown | undefined;
  /** JSON Schema for the capability's output. */
  output_schema?: unknown | undefined;
}

// ---------------------------------------------------------------------------
// Workflow Instances (runtime)
// ---------------------------------------------------------------------------

/** Runtime status of a workflow instance. */
export type InstanceStatus = "IN_PROGRESS" | "COMPLETED" | "REJECTED" | "CANCELLED";

/**
 * A running instance of a workflow -- the execution state.
 *
 * Tracks the current position in the decision graph and
 * the trace of decisions made so far.
 */
export interface WorkflowInstance {
  id: string;
  workflow_id: string;
  applicant_name: string;
  current_node_id?: string | undefined;
  status: InstanceStatus;
  /** Ordered trace of transitions and decisions made during execution. */
  trace: unknown[];
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

/** Categories of auditable events in the institutional model. */
export type AuditAction =
  // Model mutations
  | "organization_created"
  | "organization_updated"
  | "role_created"
  | "role_updated"
  | "role_deleted"
  | "workflow_created"
  | "workflow_updated"
  | "node_created"
  | "node_updated"
  | "node_deleted"
  | "edge_created"
  | "edge_updated"
  | "edge_deleted"
  | "policy_attached"
  | "policy_updated"
  | "policy_detached"
  | "integration_registered"
  | "integration_updated"
  // Runtime events
  | "instance_created"
  | "decision_made"
  | "transition_executed"
  | "document_submitted"
  | "instance_completed"
  | "instance_cancelled"
  // Agent events
  | "agent_recommendation"
  | "agent_decision"
  | "override_applied";

/** The actor who performed an auditable action. */
export type Actor =
  | { type: "user"; user_id: string; display_name: string }
  | { type: "agent"; agent_id: string; role: string }
  | { type: "system" };

/**
 * A single entry in the audit log.
 *
 * Each entry is cryptographically chained to the previous entry via `prev_hash`
 * and `entry_hash`, forming a tamper-evident append-only log.
 */
export interface AuditEntry {
  id: string;
  timestamp: string;
  sequence: number;
  action: AuditAction;
  actor: Actor;
  prior_state?: unknown | undefined;
  new_state?: unknown | undefined;
  reasoning?: string | undefined;
  prev_hash: string;
  entry_hash: string;
}

// ---------------------------------------------------------------------------
// Validation / Constraints
// ---------------------------------------------------------------------------

/** Severity of a constraint violation. */
export type Severity = "error" | "warning";

/** Where in the institutional model a violation was detected. */
export type ViolationLocation =
  | { kind: "organization"; organization_id: string }
  | { kind: "workflow"; workflow_id: string }
  | { kind: "node"; workflow_id: string; node_id: string }
  | { kind: "edge"; workflow_id: string; edge_id: string }
  | { kind: "role"; role_id: string }
  | { kind: "policy"; policy_id: string }
  | { kind: "global" };

/** A single constraint violation with full diagnostic context. */
export interface ConstraintViolation {
  /** Name of the constraint that produced this violation. */
  constraint_name: string;
  /** How severe the violation is. */
  severity: Severity;
  /** Human-readable description of what went wrong. */
  message: string;
  /** Which entity in the model is responsible. */
  location: ViolationLocation;
  /** Optional suggestion for how to fix the issue. */
  suggestion?: string | undefined;
}

/**
 * Aggregated result of running all constraints.
 *
 * `is_valid` is true only if there are zero error-severity violations.
 * Warnings alone do not make the result invalid.
 */
export interface ValidationResult {
  violations: ConstraintViolation[];
  is_valid: boolean;
}

// ---------------------------------------------------------------------------
// Chain Verification (audit chain integrity check)
// ---------------------------------------------------------------------------

/** Result of verifying the audit log's hash chain integrity. */
export interface ChainVerification {
  /** Whether the entire chain is intact. */
  valid: boolean;
  /** Total number of entries checked. */
  entries_checked: number;
  /** Details of any broken links in the chain. */
  errors: ChainVerificationError[];
}

/** A single broken link in the audit chain. */
export interface ChainVerificationError {
  entry_id: string;
  sequence: number;
  message: string;
}
