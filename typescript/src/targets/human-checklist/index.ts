/**
 * Human Checklist Compilation Target
 *
 * Generates human-readable checklists from compiled edge specifications.
 *
 * This is the "fallback" target for edge specifications that cannot (or
 * should not) be fully automated. Instead of generating machine-executable
 * automation code, this target produces structured checklists that a human
 * operator follows manually.
 *
 * Use cases:
 * - Edges involving judgment that exceeds current automation confidence
 * - Compliance-critical transitions that require human verification
 * - Organizations that prefer manual processes with AI guidance
 * - Gradual automation: start with checklists, upgrade to n8n/API later
 *
 * The generated checklists include:
 * - Step-by-step instructions derived from the edge specification
 * - Policy reminders for applicable constraints and procedures
 * - Required documents and approvals from edge requirements
 * - Verification checkpoints
 *
 * Output format is Markdown, suitable for rendering in a UI or printing.
 */

import type { Edge, Policy } from "../../types/index.js";
import type {
  CompilationTarget,
  ResolvedCapability,
} from "../../integration-compiler/index.js";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A single step in a human checklist. */
export interface ChecklistStep {
  /** Step number (1-indexed). */
  stepNumber: number;
  /** Instruction text for the human operator. */
  instruction: string;
  /** Whether this step requires explicit sign-off. */
  requiresSignoff: boolean;
  /** Policies relevant to this step. */
  policyReminders: string[];
  /** Expected artifacts or documents from this step. */
  expectedOutputs: string[];
}

/** A complete human checklist for an edge transition. */
export interface Checklist {
  /** Title of the checklist (derived from the edge label or rule). */
  title: string;
  /** Brief description of the transition this checklist covers. */
  description: string;
  /** Ordered steps the human operator must follow. */
  steps: ChecklistStep[];
  /** Policies that apply to the entire transition. */
  applicablePolicies: string[];
  /** Required authority level for executing this transition. */
  requiredAuthorityLevel: number;
}

// ---------------------------------------------------------------------------
// HumanChecklistTarget
// ---------------------------------------------------------------------------

/**
 * Compilation target that generates human-readable checklists.
 *
 * Implements the CompilationTarget interface from the integration-compiler
 * module. Produces Markdown-formatted checklists rather than executable code.
 */
export class HumanChecklistTarget implements CompilationTarget {
  readonly name = "human-checklist";

  /**
   * Generate a Markdown checklist from resolved capabilities.
   *
   * Translates each resolved capability into a human-actionable step,
   * incorporates policy constraints as reminders, and formats the
   * result as a Markdown document.
   *
   * @param edge - The edge being compiled.
   * @param capabilities - The resolved capabilities to describe as steps.
   * @param policies - Applicable policies to include as reminders.
   * @returns Markdown string representing the checklist.
   */
  async generate(
    edge: Edge,
    capabilities: ResolvedCapability[],
    policies: Policy[],
  ): Promise<string> {
    throw new Error("Not yet implemented");
  }

  /**
   * Build the structured checklist object from an edge and its context.
   *
   * This intermediate representation can be used programmatically
   * before formatting to Markdown.
   *
   * @param edge - The edge being compiled.
   * @param capabilities - The resolved capabilities.
   * @param policies - Applicable policies.
   * @returns A structured Checklist object.
   */
  buildChecklist(
    edge: Edge,
    capabilities: ResolvedCapability[],
    policies: Policy[],
  ): Checklist {
    throw new Error("Not yet implemented");
  }

  /**
   * Format a Checklist object as a Markdown document.
   *
   * @param checklist - The structured checklist to format.
   * @returns Markdown string.
   */
  formatAsMarkdown(checklist: Checklist): string {
    throw new Error("Not yet implemented");
  }
}
