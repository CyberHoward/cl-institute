/**
 * Integration Compiler Module
 *
 * Compiles edge specifications (natural language descriptions of what
 * must happen between decision points) into concrete automation code
 * for target platforms.
 *
 * The compilation pipeline:
 *
 * 1. Parse the edge's `rule` field (natural language intent).
 * 2. Match against available integration capabilities.
 * 3. Use the LLM to resolve ambiguity and select the best mapping.
 * 4. Generate target-specific automation code via a compilation target plugin.
 *
 * The compiler operates on intent, not implementation. An edge rule like
 * "route contract to legal for review via DocuSign" is compiled differently
 * depending on the target: an n8n workflow node, a direct API call sequence,
 * or a human checklist item.
 *
 * This module depends on:
 * - orchestration/ for LLM-assisted mapping
 * - targets/ for platform-specific code generation
 * - cli-bridge/ for reading integration capabilities from the model
 */

import type {
  Edge,
  Integration,
  Capability,
  Workflow,
  Policy,
} from "../types/index.js";
import type { Orchestrator } from "../orchestration/index.js";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** The result of compiling a single edge specification. */
export interface CompiledEdge {
  /** The original edge that was compiled. */
  edge: Edge;
  /** The compilation target that was used. */
  targetName: string;
  /** The generated automation code/config (target-specific). */
  output: string;
  /** Which integration capabilities were mapped to. */
  resolvedCapabilities: ResolvedCapability[];
  /** Warnings or notes from the compilation process. */
  warnings: string[];
}

/** A capability matched to part of an edge specification. */
export interface ResolvedCapability {
  /** The integration providing this capability. */
  integration: Integration;
  /** The specific capability being used. */
  capability: Capability;
  /** How this capability maps to the edge rule (LLM-generated explanation). */
  mapping: string;
  /** Confidence in the mapping (0.0 to 1.0). */
  confidence: number;
}

/**
 * Interface that compilation target plugins must implement.
 *
 * Each target (n8n, api-direct, human-checklist) provides a different
 * code generation strategy for the same edge specifications.
 */
export interface CompilationTarget {
  /** Unique name of this target (e.g., "n8n", "api-direct"). */
  readonly name: string;

  /**
   * Generate target-specific automation code from resolved capabilities.
   *
   * @param edge - The edge being compiled.
   * @param capabilities - The resolved capabilities to use.
   * @param policies - Applicable policies that constrain the automation.
   * @returns The generated automation code or configuration.
   */
  generate(
    edge: Edge,
    capabilities: ResolvedCapability[],
    policies: Policy[],
  ): Promise<string>;
}

// ---------------------------------------------------------------------------
// IntegrationCompiler
// ---------------------------------------------------------------------------

/**
 * Compiles edge specifications into platform-specific automations.
 *
 * Uses LLM reasoning to map natural language edge rules to concrete
 * integration capabilities, then delegates to target plugins for
 * code generation.
 */
export class IntegrationCompiler {
  private readonly targets: Map<string, CompilationTarget> = new Map();

  /**
   * @param orchestrator - The LLM orchestrator for capability matching.
   */
  constructor(private readonly orchestrator: Orchestrator) {}

  /**
   * Register a compilation target plugin.
   *
   * @param target - The target to register.
   */
  registerTarget(target: CompilationTarget): void {
    this.targets.set(target.name, target);
  }

  /**
   * Compile a single edge specification for the given target.
   *
   * 1. Reads the edge's rule text.
   * 2. Uses the LLM to match against available capabilities.
   * 3. Delegates to the target plugin for code generation.
   *
   * @param edge - The edge to compile.
   * @param availableIntegrations - Integrations available in the model.
   * @param targetName - Which compilation target to use.
   * @param applicablePolicies - Policies that may constrain the automation.
   * @returns The compiled edge with generated output.
   */
  async compileEdge(
    edge: Edge,
    availableIntegrations: Integration[],
    targetName: string,
    applicablePolicies: Policy[],
  ): Promise<CompiledEdge> {
    throw new Error("Not yet implemented");
  }

  /**
   * Compile all edges in a workflow for the given target.
   *
   * @param workflow - The workflow whose edges should be compiled.
   * @param edges - The edges to compile.
   * @param availableIntegrations - Integrations available in the model.
   * @param targetName - Which compilation target to use.
   * @param applicablePolicies - Policies that may constrain the automations.
   * @returns An array of compiled edges.
   */
  async compileWorkflow(
    workflow: Workflow,
    edges: Edge[],
    availableIntegrations: Integration[],
    targetName: string,
    applicablePolicies: Policy[],
  ): Promise<CompiledEdge[]> {
    throw new Error("Not yet implemented");
  }

  /**
   * List the names of all registered compilation targets.
   *
   * @returns Array of target names.
   */
  listTargets(): string[] {
    return Array.from(this.targets.keys());
  }
}
