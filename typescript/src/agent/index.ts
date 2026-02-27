/**
 * Agent Runtime Module
 *
 * Provides the autonomous agent that can observe workflow state, interpret
 * policies, and either recommend or (when authorized) execute decisions
 * at decision nodes.
 *
 * The agent operates within strict institutional boundaries:
 *
 * - It can only act on nodes where its authority level is sufficient.
 * - All actions are audit-logged with full reasoning chains.
 * - Hard constraints (policy strength = "constraint") are NEVER overridden.
 * - The agent can be configured to require human approval before executing.
 *
 * The agent runtime depends on:
 * - cli-bridge/ for reading and mutating institutional state
 * - orchestration/ for LLM reasoning
 * - policy-interpreter/ for policy assembly and interpretation
 *
 * Architecture note: the agent is a TypeScript process that communicates
 * with the Rust core exclusively through CLI invocations. It does not
 * have direct database access.
 */

import type {
  Workflow,
  DecisionNode,
  WorkflowInstance,
  AuditEntry,
} from "../types/index.js";
import type { CliBridge } from "../cli-bridge/index.js";
import type { Orchestrator } from "../orchestration/index.js";
import type { PolicyInterpreter, PolicyRecommendation } from "../policy-interpreter/index.js";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Operating mode for the agent. */
export type AgentMode =
  /** Agent observes and recommends but never executes. */
  | "advisory"
  /** Agent executes decisions autonomously within its authority. */
  | "autonomous"
  /** Agent prepares decisions but requires human approval before executing. */
  | "supervised";

/** Configuration for the agent runtime. */
export interface AgentConfig {
  /** Unique identifier for this agent instance. */
  agentId: string;
  /** The institutional role the agent operates under. */
  roleName: string;
  /** Operating mode. */
  mode: AgentMode;
  /** Maximum authority level the agent is allowed to exercise. */
  maxAuthorityLevel: number;
  /**
   * Polling interval in milliseconds for watching workflow instances.
   * Only used when the agent is running in watch mode.
   */
  pollIntervalMs: number;
}

/** The result of the agent evaluating a decision point. */
export interface AgentDecision {
  /** The workflow instance being acted on. */
  instanceId: string;
  /** The decision node being evaluated. */
  nodeId: string;
  /** The policy recommendation that informed this decision. */
  recommendation: PolicyRecommendation;
  /** Whether the agent executed the decision or only recommended it. */
  executed: boolean;
  /** Full reasoning chain for audit purposes. */
  reasoning: string;
  /** Timestamp of the decision. */
  timestamp: string;
}

/** Events emitted by the agent during operation. */
export type AgentEvent =
  | { type: "started"; config: AgentConfig }
  | { type: "evaluating"; instanceId: string; nodeId: string }
  | { type: "decision"; decision: AgentDecision }
  | { type: "error"; error: string; instanceId?: string | undefined }
  | { type: "stopped"; reason: string };

/** Callback for agent events. */
export type AgentEventHandler = (event: AgentEvent) => void;

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/**
 * Autonomous agent runtime for institutional decision-making.
 *
 * The agent monitors workflow instances, evaluates pending decisions
 * using the policy interpreter, and either recommends or executes
 * actions depending on its operating mode and authority level.
 */
export class Agent {
  private running = false;
  private eventHandlers: AgentEventHandler[] = [];

  /**
   * @param config - Agent configuration.
   * @param bridge - CLI bridge for institutional state access.
   * @param orchestrator - LLM orchestrator for reasoning.
   * @param policyInterpreter - Policy interpreter for decision support.
   */
  constructor(
    private readonly config: AgentConfig,
    private readonly bridge: CliBridge,
    private readonly orchestrator: Orchestrator,
    private readonly policyInterpreter: PolicyInterpreter,
  ) {}

  /**
   * Register an event handler to receive agent lifecycle events.
   *
   * @param handler - Callback invoked for each agent event.
   */
  onEvent(handler: AgentEventHandler): void {
    this.eventHandlers.push(handler);
  }

  /**
   * Evaluate a single decision point and produce a recommendation
   * or execute the decision (depending on mode and authority).
   *
   * @param workflow - The workflow containing the decision.
   * @param node - The decision node to evaluate.
   * @param instance - The running workflow instance.
   * @returns The agent's decision, including reasoning.
   */
  async evaluate(
    workflow: Workflow,
    node: DecisionNode,
    instance: WorkflowInstance,
  ): Promise<AgentDecision> {
    throw new Error("Not yet implemented");
  }

  /**
   * Start the agent in watch mode, continuously monitoring for
   * workflow instances that need decisions.
   *
   * The agent polls for pending decisions at the configured interval
   * and evaluates them as they appear. It runs until `stop()` is called.
   */
  async start(): Promise<void> {
    throw new Error("Not yet implemented");
  }

  /**
   * Stop the agent's watch loop.
   *
   * @param reason - Human-readable reason for stopping.
   */
  stop(reason: string): void {
    throw new Error("Not yet implemented");
  }

  /**
   * Check whether the agent has sufficient authority to act on a node.
   *
   * @param node - The decision node to check.
   * @returns True if the agent's max authority level meets the node's requirement.
   */
  hasAuthority(node: DecisionNode): boolean {
    return this.config.maxAuthorityLevel >= node.requires_authority;
  }

  /** Whether the agent is currently running in watch mode. */
  get isRunning(): boolean {
    return this.running;
  }
}
