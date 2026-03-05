import { z } from "zod";

// ---------------------------------------------------------------------------
// Institution
// ---------------------------------------------------------------------------

export interface Institution {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Role
// ---------------------------------------------------------------------------

export interface Role {
  id: string;
  institution_id: string;
  name: string;
  description?: string;
  authority_level: number;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Actor
// ---------------------------------------------------------------------------

export type ActorType = "human" | "agent";

export interface Actor {
  id: string;
  institution_id: string;
  name: string;
  type: ActorType;
  created_at: string;
  updated_at: string;
}

export interface ActorRoleAssignment {
  actor_id: string;
  role_id: string;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Policy
// ---------------------------------------------------------------------------

export type PolicyStrength = "constraint" | "procedure" | "preference" | "context";

export interface Policy {
  id: string;
  institution_id: string;
  scope: string;
  strength: PolicyStrength;
  text: string;
  metadata?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Net (workflow)
// ---------------------------------------------------------------------------

export interface Net {
  id: string;
  institution_id: string;
  domain?: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Place
// ---------------------------------------------------------------------------

export const JsonSchemaValue = z.record(z.unknown());
export type JsonSchema = z.infer<typeof JsonSchemaValue>;

export interface Place {
  id: string;
  net_id: string;
  description: string;
  schema?: JsonSchema;
}

// ---------------------------------------------------------------------------
// Transition
// ---------------------------------------------------------------------------

export type TransitionMode = "deterministic" | "judgment" | "agentic";

export type DecisionType =
  | "approval"
  | "classification"
  | "prioritization"
  | "allocation"
  | "exception_handling";

export interface Postconditions {
  required: string[];
  desired?: string[];
  escalation?: string[];
}

export type EvidenceType = "artifact" | "reference" | "attestation";

export interface EvidenceRequirement {
  id: string;
  description: string;
  type: EvidenceType;
  required: boolean;
}

export interface Transition {
  id: string;
  net_id: string;

  // CPN core
  consumes: string[];
  produces: string[];
  guard?: string;

  // Institutional semantics
  intent: string;
  mode: TransitionMode;
  decision_type?: DecisionType;
  requires_authority: number;
  authorized_roles?: string[];

  // Data flow
  input_schema?: JsonSchema;
  output_schema?: JsonSchema;
  context_sources: string[];

  // Execution contract
  postconditions: Postconditions;
  evidence_requirements: EvidenceRequirement[];
  available_tools: string[];
  timeout?: number;
}

// ---------------------------------------------------------------------------
// Arc (connects places to transitions)
// ---------------------------------------------------------------------------

export type ArcDirection = "place_to_transition" | "transition_to_place";

export interface Arc {
  id: string;
  net_id: string;
  place_id: string;
  transition_id: string;
  direction: ArcDirection;
}

// ---------------------------------------------------------------------------
// Token and Marking (runtime)
// ---------------------------------------------------------------------------

export interface Token {
  id: string;
  instance_id: string;
  place_id: string;
  payload: Record<string, unknown>;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Workflow Instance
// ---------------------------------------------------------------------------

export type InstanceStatus = "running" | "completed" | "stuck" | "suspended";

export interface WorkflowInstance {
  id: string;
  net_id: string;
  status: InstanceStatus;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

export type AuditAction =
  | "instance_created"
  | "transition_fired"
  | "judgment_pending"
  | "judgment_resolved"
  | "postcondition_failed"
  | "escalation_triggered"
  | "policy_consulted"
  | "override_applied";

export interface Evidence {
  requirement_id: string;
  type: EvidenceType;
  content: unknown;
  captured_at: string;
}

export interface AuditEntry {
  id: string;
  instance_id: string;
  timestamp: string;
  sequence: number;
  action: AuditAction;
  actor: { actor_id: string; role_id: string; authority_level: number };
  transition_id?: string;
  marking_before?: Record<string, unknown>;
  marking_after?: Record<string, unknown>;
  evidence?: Evidence[];
  reasoning?: string;
  prev_hash: string;
  entry_hash: string;
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

export type Severity = "error" | "warning";

export interface ConstraintViolation {
  constraint_name: string;
  severity: Severity;
  message: string;
  location: string;
  suggestion?: string;
}

export interface ValidationResult {
  violations: ConstraintViolation[];
  is_valid: boolean;
}

// ---------------------------------------------------------------------------
// Firing result (returned by fireTransition)
// ---------------------------------------------------------------------------

export interface FiringResult {
  success: boolean;
  transition_id: string;
  instance_id: string;
  tokens_consumed: Token[];
  tokens_produced: Token[];
  postcondition_results: Record<string, boolean>;
  evidence: Evidence[];
  audit_entry_id: string;
  error?: string;
}

// ---------------------------------------------------------------------------
// Pending judgment (returned by getPendingJudgments)
// ---------------------------------------------------------------------------

export interface PendingJudgment {
  instance_id: string;
  transition_id: string;
  transition_intent: string;
  transition_mode: "judgment";
  requires_authority: number;
  token_payloads: Record<string, unknown>[];
  policies: Policy[];
}
