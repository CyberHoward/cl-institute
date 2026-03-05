import type { AgentTool } from "@mariozechner/pi-agent-core";
import type {
  Net,
  Transition,
  Marking,
  Token,
  ExecutionLog,
} from "./types.js";
import { ContextStore } from "../context/store.js";
import { executeTransition } from "../agent/executor.js";
import { verifyPostconditions } from "../agent/postconditions.js";

function markingToRecord(marking: Marking): Record<string, Token> {
  return Object.fromEntries(marking);
}

function isEnabled(transition: Transition, marking: Marking): boolean {
  return transition.consumes.every((placeId) => {
    const token = marking.get(placeId);
    return token != null && token.count > 0;
  });
}

function findEnabledTransitions(
  net: Net,
  marking: Marking,
): Transition[] {
  return net.transitions.filter((t) => isEnabled(t, marking));
}

export interface EngineOptions {
  /** All tools available to agents */
  tools: AgentTool[];
  /** Maximum number of transitions to fire before stopping */
  maxSteps?: number;
  /** Verbose logging */
  verbose?: boolean;
}

export interface EngineResult {
  logs: ExecutionLog[];
  finalMarking: Record<string, Token>;
  status: "completed" | "stuck" | "max-steps";
}

/**
 * Minimal Petri net engine. Finds enabled transitions, hands them to the
 * agent executor, verifies postconditions, and updates the marking.
 */
export async function runNet(
  net: Net,
  initialMarking: Marking,
  contextStore: ContextStore,
  options: EngineOptions,
): Promise<EngineResult> {
  const marking = new Map(initialMarking);
  const logs: ExecutionLog[] = [];
  const maxSteps = options.maxSteps ?? 20;
  let steps = 0;

  console.log("\n========================================");
  console.log(`Starting net: ${net.id}`);
  console.log(`Initial marking: ${JSON.stringify(markingToRecord(marking))}`);
  console.log("========================================\n");

  while (steps < maxSteps) {
    const enabled = findEnabledTransitions(net, marking);

    if (enabled.length === 0) {
      console.log("\nNo enabled transitions. Engine stopping.");
      // Check if we reached the final state
      const finalPlace = net.places.at(-1);
      const finalToken = finalPlace ? marking.get(finalPlace.id) : undefined;
      const status =
        finalToken && finalToken.count > 0 ? "completed" : "stuck";
      return { logs, finalMarking: markingToRecord(marking), status };
    }

    // Pick the first enabled transition (FIFO for now)
    const transition = enabled[0]!;
    steps++;

    console.log(`\n[Step ${steps}] Firing: ${transition.id}`);
    console.log(`  Enabled transitions: ${enabled.map((t) => t.id).join(", ")}`);

    const markingBefore = markingToRecord(marking);
    const startTime = Date.now();

    // Gather context from the store
    const context = contextStore.gather(transition.context_sources);

    // Filter tools to only those available for this transition
    const availableTools = options.tools.filter((t) =>
      transition.available_tools.includes(t.name),
    );
    for (const toolName of transition.available_tools) {
      if (!availableTools.some((t) => t.name === toolName)) {
        console.warn(`  Warning: tool "${toolName}" not found in registry`);
      }
    }

    // Execute the transition via the agent
    const result = await executeTransition(transition, context, availableTools);

    // Verify postconditions
    console.log("\n  Verifying postconditions...");
    const verification = await verifyPostconditions(
      transition.postconditions,
      result,
    );

    const durationMs = Date.now() - startTime;

    if (verification.allRequiredMet) {
      // Consume input tokens
      for (const placeId of transition.consumes) {
        const token = marking.get(placeId)!;
        if (token.count <= 1) {
          marking.delete(placeId);
        } else {
          marking.set(placeId, { ...token, count: token.count - 1 });
        }
      }

      // Produce output tokens with payload from execution
      for (const placeId of transition.produces) {
        const existing = marking.get(placeId);
        marking.set(placeId, {
          count: (existing?.count ?? 0) + 1,
          payload: result.payload,
        });
      }

      // Update context store with payload data
      contextStore.merge(result.payload);

      console.log(`\n  ✓ Transition ${transition.id} FIRED successfully`);
      console.log(`    Duration: ${durationMs}ms`);
      console.log(`    Marking: ${JSON.stringify(markingToRecord(marking))}`);

      logs.push({
        transitionId: transition.id,
        markingBefore,
        agentActions: result.actions,
        postconditionResults: verification.results,
        markingAfter: markingToRecord(marking),
        status: "fired",
        durationMs,
      });
    } else {
      console.log(`\n  ✗ Transition ${transition.id} FAILED`);
      console.log(`    Failed postconditions:`);
      for (const [pc, met] of Object.entries(verification.results)) {
        if (!met) console.log(`      - ${pc}`);
      }

      const hasEscalation =
        transition.postconditions.escalation &&
        transition.postconditions.escalation.length > 0;

      logs.push({
        transitionId: transition.id,
        markingBefore,
        agentActions: result.actions,
        postconditionResults: verification.results,
        markingAfter: markingToRecord(marking),
        status: hasEscalation ? "escalated" : "failed",
        durationMs,
      });

      if (hasEscalation) {
        console.log(
          `    Escalation paths: ${transition.postconditions.escalation!.join(", ")}`,
        );
      }

      // Don't modify marking on failure — transition didn't fire
      break;
    }
  }

  if (steps >= maxSteps) {
    console.log(`\nMax steps (${maxSteps}) reached. Engine stopping.`);
    return { logs, finalMarking: markingToRecord(marking), status: "max-steps" };
  }

  return { logs, finalMarking: markingToRecord(marking), status: "completed" };
}
