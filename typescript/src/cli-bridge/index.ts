/**
 * CLI Bridge -- the boundary between the TypeScript intelligence layer and
 * the Rust `inst` CLI.
 *
 * This module is the ONLY place that spawns the `inst` binary. All other
 * TypeScript modules consume data through the typed interfaces this bridge
 * provides. Communication is strictly:
 *
 *   TypeScript -> spawns `inst <subcommand> --json` -> parses stdout JSON
 *
 * No IPC, no sockets, no shared memory. Just CLI invocations and JSON.
 */

import { execaNode, execa, type ResultPromise } from "execa";
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
 * Error thrown when the `inst` CLI exits with a non-zero status code.
 * Captures stderr and the exit code for diagnostic purposes.
 */
export class CliError extends Error {
  constructor(
    public readonly command: string,
    public readonly args: readonly string[],
    public readonly exitCode: number | undefined,
    public readonly stderr: string,
  ) {
    super(
      `inst CLI failed (exit ${exitCode ?? "unknown"}): ${stderr.trim() || "(no stderr)"}`,
    );
    this.name = "CliError";
  }
}

/**
 * Error thrown when the CLI produces output that cannot be parsed as
 * valid JSON or does not match the expected shape.
 */
export class CliParseError extends Error {
  constructor(
    public readonly command: string,
    public readonly rawOutput: string,
    cause?: unknown,
  ) {
    super(`Failed to parse JSON output from '${command}'`);
    this.name = "CliParseError";
    if (cause !== undefined) {
      this.cause = cause;
    }
  }
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/** Configuration for the CLI bridge. */
export interface CliBridgeOptions {
  /**
   * Absolute path to the `inst` binary.
   * Defaults to "inst" (resolved via $PATH).
   */
  binaryPath?: string | undefined;

  /**
   * Working directory / project path to pass to the CLI.
   * If provided, `--project-path <path>` is prepended to every invocation.
   */
  projectPath?: string | undefined;
}

// ---------------------------------------------------------------------------
// CliBridge
// ---------------------------------------------------------------------------

/**
 * Typed wrapper around the `inst` CLI binary.
 *
 * Every public method corresponds to a CLI subcommand. The bridge:
 * 1. Constructs the argument list (including `--json` for machine-readable output).
 * 2. Spawns the `inst` process via `execa`.
 * 3. Asserts a zero exit code (throws `CliError` otherwise).
 * 4. Parses the JSON stdout into the corresponding TypeScript type.
 *
 * This class is intentionally stateless -- it holds only configuration.
 * Each call is an independent subprocess invocation.
 */
export class CliBridge {
  private readonly binaryPath: string;
  private readonly projectPath: string | undefined;

  constructor(options: CliBridgeOptions = {}) {
    this.binaryPath = options.binaryPath ?? "inst";
    this.projectPath = options.projectPath;
  }

  // -----------------------------------------------------------------------
  // Internal helpers
  // -----------------------------------------------------------------------

  /**
   * Execute the CLI with the given arguments and return raw stdout.
   * Throws `CliError` on non-zero exit.
   */
  private async exec(args: string[]): Promise<string> {
    const fullArgs: string[] = [];

    if (this.projectPath !== undefined) {
      fullArgs.push("--project-path", this.projectPath);
    }

    fullArgs.push(...args, "--json");

    try {
      const result = await execa(this.binaryPath, fullArgs);
      return result.stdout;
    } catch (error: unknown) {
      if (isExecaError(error)) {
        throw new CliError(
          this.binaryPath,
          fullArgs,
          error.exitCode,
          error.stderr,
        );
      }
      throw error;
    }
  }

  /**
   * Execute the CLI and parse the JSON output.
   * Throws `CliParseError` if stdout is not valid JSON.
   */
  private async execJson<T>(args: string[]): Promise<T> {
    const stdout = await this.exec(args);

    try {
      return JSON.parse(stdout) as T;
    } catch (cause: unknown) {
      throw new CliParseError(
        `${this.binaryPath} ${args.join(" ")}`,
        stdout,
        cause,
      );
    }
  }

  // -----------------------------------------------------------------------
  // Organization
  // -----------------------------------------------------------------------

  /** Retrieve the current organization's details. */
  async orgShow(): Promise<Organization> {
    return this.execJson<Organization>(["org", "show"]);
  }

  // -----------------------------------------------------------------------
  // Roles
  // -----------------------------------------------------------------------

  /** Create a new organizational role. */
  async roleCreate(opts: {
    name: string;
    authorityLevel?: number | undefined;
    description?: string | undefined;
  }): Promise<OrganizationalRole> {
    const args = ["role", "create", opts.name];
    if (opts.authorityLevel !== undefined) {
      args.push("--authority-level", String(opts.authorityLevel));
    }
    if (opts.description !== undefined) {
      args.push("--description", opts.description);
    }
    return this.execJson<OrganizationalRole>(args);
  }

  /** List all organizational roles. */
  async roleList(): Promise<OrganizationalRole[]> {
    return this.execJson<OrganizationalRole[]>(["role", "list"]);
  }

  /** Show details of a specific role by name. */
  async roleShow(name: string): Promise<OrganizationalRole> {
    return this.execJson<OrganizationalRole>(["role", "show", name]);
  }

  // -----------------------------------------------------------------------
  // Workflows
  // -----------------------------------------------------------------------

  /** Create a new workflow definition. */
  async workflowCreate(opts: {
    name: string;
    function?: string | undefined;
    description?: string | undefined;
  }): Promise<Workflow> {
    const args = ["workflow", "create", opts.name];
    if (opts.function !== undefined) {
      args.push("--function", opts.function);
    }
    if (opts.description !== undefined) {
      args.push("--description", opts.description);
    }
    return this.execJson<Workflow>(args);
  }

  /** List all workflow definitions. */
  async workflowList(): Promise<Workflow[]> {
    return this.execJson<Workflow[]>(["workflow", "list"]);
  }

  /**
   * Run the constraint engine against one or all workflows.
   * Returns the validation result with any violations.
   */
  async workflowValidate(name?: string | undefined): Promise<ValidationResult> {
    const args = ["workflow", "validate"];
    if (name !== undefined) {
      args.push(name);
    }
    return this.execJson<ValidationResult>(args);
  }

  // -----------------------------------------------------------------------
  // Nodes
  // -----------------------------------------------------------------------

  /** Create a new decision node in a workflow. */
  async nodeCreate(opts: {
    workflow: string;
    type: NodeType;
    label: string;
    decisionType?: DecisionType | undefined;
    requiresAuthority?: number | undefined;
  }): Promise<DecisionNode> {
    const args = [
      "node",
      "create",
      "--workflow",
      opts.workflow,
      "--type",
      opts.type,
      opts.label,
    ];
    if (opts.decisionType !== undefined) {
      args.push("--decision-type", opts.decisionType);
    }
    if (opts.requiresAuthority !== undefined) {
      args.push("--requires-authority", String(opts.requiresAuthority));
    }
    return this.execJson<DecisionNode>(args);
  }

  /** List all nodes in a workflow. */
  async nodeList(workflow: string): Promise<DecisionNode[]> {
    return this.execJson<DecisionNode[]>(["node", "list", "--workflow", workflow]);
  }

  // -----------------------------------------------------------------------
  // Edges
  // -----------------------------------------------------------------------

  /** Create a new edge between two nodes in a workflow. */
  async edgeCreate(opts: {
    workflow: string;
    from: string;
    to: string;
    label?: string | undefined;
    rule?: string | undefined;
  }): Promise<Edge> {
    const args = [
      "edge",
      "create",
      "--workflow",
      opts.workflow,
      "--from",
      opts.from,
      "--to",
      opts.to,
    ];
    if (opts.label !== undefined) {
      args.push("--label", opts.label);
    }
    if (opts.rule !== undefined) {
      args.push("--rule", opts.rule);
    }
    return this.execJson<Edge>(args);
  }

  /** List all edges in a workflow. */
  async edgeList(workflow: string): Promise<Edge[]> {
    return this.execJson<Edge[]>(["edge", "list", "--workflow", workflow]);
  }

  // -----------------------------------------------------------------------
  // Policies
  // -----------------------------------------------------------------------

  /** Attach a new policy to a scope. */
  async policyAttach(opts: {
    scope: string;
    strength: PolicyStrength;
    text: string;
  }): Promise<Policy> {
    return this.execJson<Policy>([
      "policy",
      "attach",
      "--scope",
      opts.scope,
      "--strength",
      opts.strength,
      "--text",
      opts.text,
    ]);
  }

  /** List policies, optionally filtered by scope. */
  async policyList(scope?: string | undefined): Promise<Policy[]> {
    const args = ["policy", "list"];
    if (scope !== undefined) {
      args.push("--scope", scope);
    }
    return this.execJson<Policy[]>(args);
  }

  // -----------------------------------------------------------------------
  // Integrations
  // -----------------------------------------------------------------------

  /** List all registered integrations. */
  async integrationList(): Promise<Integration[]> {
    return this.execJson<Integration[]>(["integration", "list"]);
  }

  // -----------------------------------------------------------------------
  // Audit
  // -----------------------------------------------------------------------

  /** Retrieve audit log entries, optionally limited to the most recent N. */
  async auditLog(opts?: { last?: number | undefined } | undefined): Promise<AuditEntry[]> {
    const args = ["audit", "log"];
    if (opts?.last !== undefined) {
      args.push("--last", String(opts.last));
    }
    return this.execJson<AuditEntry[]>(args);
  }

  /** Verify the integrity of the audit log's hash chain. */
  async auditVerify(): Promise<ChainVerification> {
    return this.execJson<ChainVerification>(["audit", "verify"]);
  }

  // -----------------------------------------------------------------------
  // Graph export
  // -----------------------------------------------------------------------

  /**
   * Export a workflow graph in the specified format.
   * Returns the raw string output (DOT source or JSON document).
   */
  async graphExport(opts: {
    workflow: string;
    format: "dot" | "json";
  }): Promise<string> {
    // For graph export we return the raw string rather than parsed JSON,
    // since DOT format is not JSON. For JSON format, callers can parse
    // the returned string themselves if needed.
    return this.exec([
      "graph",
      "export",
      "--workflow",
      opts.workflow,
      "--format",
      opts.format,
    ]);
  }

  // -----------------------------------------------------------------------
  // Init
  // -----------------------------------------------------------------------

  /** Initialize a new institutional project. */
  async init(opts: { name: string }): Promise<void> {
    await this.exec(["init", opts.name]);
  }
}

// ---------------------------------------------------------------------------
// Internal utilities
// ---------------------------------------------------------------------------

/** Type guard for execa error objects. */
function isExecaError(
  error: unknown,
): error is { exitCode: number | undefined; stderr: string } {
  return (
    typeof error === "object" &&
    error !== null &&
    "exitCode" in error &&
    "stderr" in error
  );
}
