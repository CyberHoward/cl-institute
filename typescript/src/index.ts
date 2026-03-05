/**
 * @clinstitute/typescript
 *
 * TypeScript system for the Intelligent Institution Initiative.
 *
 * This package provides the institutional model, Petri net execution engine,
 * AI-driven policy interpretation, integration compilation, and autonomous
 * agent operation.
 *
 * Modules:
 * - types/          -- Core type definitions
 * - spike/          -- Petri net execution engine (spike)
 * - cli-bridge/     -- Model access layer
 * - orchestration/  -- LLM prompt management, conversation state
 * - policy-interpreter/ -- Policy assembly, LLM reasoning for decisions
 * - integration-compiler/ -- Edge specification -> automation compilation
 * - agent/          -- Autonomous agent runtime
 * - targets/        -- Compilation target plugins (n8n, human-checklist)
 */

// Types (re-export all interfaces and type aliases)
export type {
  Organization,
  PermissionLevel,
  OrganizationMember,
  OrganizationalRole,
  MemberRoleAssignment,
  Function,
  Workflow,
  WorkflowVersion,
  NodeType,
  DecisionType,
  DecisionNode,
  Edge,
  RequirementType,
  EdgeRequirement,
  EdgeRolePermission,
  PolicyStrength,
  Policy,
  Integration,
  Capability,
  InstanceStatus,
  WorkflowInstance,
  AuditAction,
  Actor,
  AuditEntry,
  Severity,
  ViolationLocation,
  ConstraintViolation,
  ValidationResult,
  ChainVerification,
  ChainVerificationError,
} from "./types/index.js";

// Model Access Layer
export { CliBridge, ModelError } from "./cli-bridge/index.js";
export type { CliBridgeOptions } from "./cli-bridge/index.js";

// Orchestration
export { Orchestrator } from "./orchestration/index.js";
export type {
  ConversationMessage,
  ConversationState,
  LlmProviderConfig,
  LlmResponse,
} from "./orchestration/index.js";

// Policy Interpreter
export { PolicyInterpreter } from "./policy-interpreter/index.js";
export type {
  ResolvedPolicy,
  PolicyRecommendation,
} from "./policy-interpreter/index.js";

// Integration Compiler
export { IntegrationCompiler } from "./integration-compiler/index.js";
export type {
  CompiledEdge,
  ResolvedCapability,
  CompilationTarget,
} from "./integration-compiler/index.js";

// Agent
export { Agent } from "./agent/index.js";
export type {
  AgentMode,
  AgentConfig,
  AgentDecision,
  AgentEvent,
  AgentEventHandler,
} from "./agent/index.js";

// Targets
export { N8nTarget } from "./targets/n8n/index.js";
export type {
  N8nNode,
  N8nConnection,
  N8nWorkflow,
} from "./targets/n8n/index.js";

export { HumanChecklistTarget } from "./targets/human-checklist/index.js";
export type {
  ChecklistStep,
  Checklist,
} from "./targets/human-checklist/index.js";
