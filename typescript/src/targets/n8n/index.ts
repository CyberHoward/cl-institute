/**
 * n8n Compilation Target
 *
 * Generates n8n workflow JSON from compiled edge specifications.
 *
 * n8n is a workflow automation platform that uses a node-based visual editor.
 * This target maps institutional edge specifications to n8n workflow nodes,
 * producing JSON that can be imported directly into an n8n instance.
 *
 * The generated workflows include:
 * - Trigger nodes that fire when a transition is initiated
 * - Integration nodes mapped to the resolved capabilities
 * - Conditional logic derived from policy constraints
 * - Error handling nodes for failure cases
 *
 * This target depends on the integration-compiler module's CompilationTarget
 * interface and is registered as a plugin at runtime.
 */

import type { Edge, Policy } from "../../types/index.js";
import type {
  CompilationTarget,
  ResolvedCapability,
} from "../../integration-compiler/index.js";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** An n8n workflow node definition. */
export interface N8nNode {
  /** Unique name within the workflow. */
  name: string;
  /** n8n node type (e.g., "n8n-nodes-base.httpRequest"). */
  type: string;
  /** Position in the visual editor [x, y]. */
  position: [number, number];
  /** Node-specific parameters. */
  parameters: Record<string, unknown>;
}

/** An n8n workflow connection between nodes. */
export interface N8nConnection {
  /** Source node name. */
  from: string;
  /** Target node name. */
  to: string;
  /** Output index on the source node (default 0). */
  fromOutput?: number | undefined;
  /** Input index on the target node (default 0). */
  toInput?: number | undefined;
}

/** A complete n8n workflow definition. */
export interface N8nWorkflow {
  name: string;
  nodes: N8nNode[];
  connections: Record<string, {
    main: Array<Array<{ node: string; type: string; index: number }>>;
  }>;
  settings: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// N8nTarget
// ---------------------------------------------------------------------------

/**
 * Compilation target that generates n8n workflow JSON.
 *
 * Implements the CompilationTarget interface from the integration-compiler
 * module. Registered as a plugin so the compiler can delegate to it.
 */
export class N8nTarget implements CompilationTarget {
  readonly name = "n8n";

  /**
   * Generate an n8n workflow JSON string from resolved capabilities.
   *
   * Maps each resolved capability to an n8n node, connects them in
   * sequence, and wraps the result in a valid n8n workflow document.
   *
   * @param edge - The edge being compiled.
   * @param capabilities - The resolved capabilities to map to n8n nodes.
   * @param policies - Applicable policies that may add conditional logic.
   * @returns JSON string representing an n8n workflow.
   */
  async generate(
    edge: Edge,
    capabilities: ResolvedCapability[],
    policies: Policy[],
  ): Promise<string> {
    throw new Error("Not yet implemented");
  }

  /**
   * Map a single integration capability to an n8n node definition.
   *
   * @param capability - The capability to map.
   * @param position - Visual position for the node.
   * @returns An n8n node definition.
   */
  mapCapabilityToNode(
    capability: ResolvedCapability,
    position: [number, number],
  ): N8nNode {
    throw new Error("Not yet implemented");
  }
}
