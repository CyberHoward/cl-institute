/**
 * @clinstitute/typescript
 *
 * Core workflow engine for the Intelligent Institution Initiative.
 *
 * Provides a Coloured Petri Net execution engine with institutional semantics:
 * authority-gated transitions, policy-scoped governance, cryptographic audit
 * trails, and structured context assembly for agent execution.
 *
 * Modules:
 * - core/engine   — Engine class: definition, runtime, and query operations
 * - core/audit    — Hash-chained audit log
 * - core/context  — Work order assembly for agent goal construction
 * - core/validate — Net structural validation
 * - core/db       — SQLite schema and connection management
 * - core/types    — Canonical type definitions (CPN + institutional model)
 */

export {
  Engine,
  AuditLog,
  buildWorkOrder,
  validateNet,
  DB,
} from "./core/index.js";

export type {
  TransitionDef,
  WorkOrder,
  Institution,
  Role,
  Actor,
  ActorType,
  ActorRoleAssignment,
  PolicyStrength,
  Policy,
  Net,
  Place,
  JsonSchema,
  TransitionMode,
  DecisionType,
  Postconditions,
  EvidenceType,
  EvidenceRequirement,
  Transition,
  ArcDirection,
  Arc,
  Token,
  InstanceStatus,
  WorkflowInstance,
  AuditAction,
  Evidence,
  AuditEntry,
  Severity,
  ConstraintViolation,
  ValidationResult,
  FiringResult,
  PendingJudgment,
} from "./core/index.js";
