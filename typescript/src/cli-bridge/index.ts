/**
 * CLI Bridge -- institutional model access layer.
 *
 * Access layer for the TypeScript-native institutional model.
 *
 * All other modules (agent, policy-interpreter, orchestration) consume
 * data through the typed interfaces this layer provides.
 *
 * The model, constraint engine, audit log, and store are all implemented
 * in TypeScript (see src/spike/ for the Petri net execution engine).
 */

import type {
  Organization,
  OrganizationalRole,
  Workflow,
  DecisionNode,
  NodeType,
  DecisionType,
  Edge,
  Policy,
  PolicyStrength,
  Integration,
  AuditEntry,
  ValidationResult,
  ChainVerification,
} from "../types/index.js";

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/**
 * Error thrown when an operation on the institutional model fails.
 */
export class ModelError extends Error {
  constructor(
    public readonly operation: string,
    message: string,
  ) {
    super(`Model operation '${operation}' failed: ${message}`);
    this.name = "ModelError";
  }
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/** Configuration for the CLI bridge. */
export interface CliBridgeOptions {
  /**
   * Working directory / project path for the institutional model.
   */
  projectPath?: string | undefined;
}

// ---------------------------------------------------------------------------
// CliBridge
// ---------------------------------------------------------------------------

/**
 * Typed access layer for the institutional model.
 *
 * Every public method corresponds to a model operation. This class
 * is intentionally stateless -- it holds only configuration.
 *
 * TODO: Wire these methods to the TypeScript-native model/store
 * implementation.
 */
export class CliBridge {
  private readonly projectPath: string | undefined;

  constructor(options: CliBridgeOptions = {}) {
    this.projectPath = options.projectPath;
  }

  // -----------------------------------------------------------------------
  // Organization
  // -----------------------------------------------------------------------

  async orgShow(): Promise<Organization> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Roles
  // -----------------------------------------------------------------------

  async roleCreate(opts: {
    name: string;
    authorityLevel?: number | undefined;
    description?: string | undefined;
  }): Promise<OrganizationalRole> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  async roleList(): Promise<OrganizationalRole[]> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  async roleShow(name: string): Promise<OrganizationalRole> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Workflows
  // -----------------------------------------------------------------------

  async workflowCreate(opts: {
    name: string;
    function?: string | undefined;
    description?: string | undefined;
  }): Promise<Workflow> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  async workflowList(): Promise<Workflow[]> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  async workflowValidate(name?: string | undefined): Promise<ValidationResult> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Nodes
  // -----------------------------------------------------------------------

  async nodeCreate(opts: {
    workflow: string;
    type: NodeType;
    label: string;
    decisionType?: DecisionType | undefined;
    requiresAuthority?: number | undefined;
  }): Promise<DecisionNode> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  async nodeList(workflow: string): Promise<DecisionNode[]> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Edges
  // -----------------------------------------------------------------------

  async edgeCreate(opts: {
    workflow: string;
    from: string;
    to: string;
    label?: string | undefined;
    rule?: string | undefined;
  }): Promise<Edge> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  async edgeList(workflow: string): Promise<Edge[]> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Policies
  // -----------------------------------------------------------------------

  async policyAttach(opts: {
    scope: string;
    strength: PolicyStrength;
    text: string;
  }): Promise<Policy> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  async policyList(scope?: string | undefined): Promise<Policy[]> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Integrations
  // -----------------------------------------------------------------------

  async integrationList(): Promise<Integration[]> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Audit
  // -----------------------------------------------------------------------

  async auditLog(opts?: { last?: number | undefined } | undefined): Promise<AuditEntry[]> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  async auditVerify(): Promise<ChainVerification> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Graph export
  // -----------------------------------------------------------------------

  async graphExport(opts: {
    workflow: string;
    format: "dot" | "json";
  }): Promise<string> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }

  // -----------------------------------------------------------------------
  // Init
  // -----------------------------------------------------------------------

  async init(opts: { name: string }): Promise<void> {
    throw new Error("Not yet implemented — pending TypeScript model layer");
  }
}
