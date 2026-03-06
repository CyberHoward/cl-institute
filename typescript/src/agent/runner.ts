import type { Engine } from "../core/engine.js";
import type { StepResult, RunResult, Transition } from "../core/types.js";
import { buildWorkOrder } from "../core/context.js";
import type { InstitutionalContextStore } from "../core/context-store.js";
import { PostconditionVerifier, type ExecutionEvidence } from "./postconditions.js";
import { buildSystemPrompt, buildContextPrompt } from "./prompt.js";

/** The function signature for executing a transition via LLM. */
export type TransitionExecutor = (
  transitionId: string,
  systemPrompt: string,
  contextPrompt: string,
  tools: unknown[],
) => Promise<ExecutionEvidence>;

export interface AgentRunnerOptions {
  /** Function that executes a transition (LLM agent call). */
  executor: TransitionExecutor;
  /** Institutional context store for standing facts. Optional. */
  contextStore?: InstitutionalContextStore | undefined;
  /** Tool registry. Maps tool name to tool object. Optional — tools are passed through to executor. */
  toolRegistry?: Map<string, unknown> | undefined;
  /** Max retries for runtime errors. Default: 3. */
  maxRetries?: number | undefined;
  /** Max steps for run(). Default: 20. */
  maxSteps?: number | undefined;
}

/**
 * AgentRunner drives agentic transition execution.
 * Sits above the Engine — observes enabled transitions, builds work orders,
 * invokes the executor (LLM), verifies postconditions, and fires transitions.
 */
export class AgentRunner {
  private readonly maxRetries: number;
  private readonly maxSteps: number;

  constructor(
    private readonly engine: Engine,
    private readonly verifier: PostconditionVerifier,
    private readonly options: AgentRunnerOptions,
  ) {
    this.maxRetries = options.maxRetries ?? 3;
    this.maxSteps = options.maxSteps ?? 20;
  }

  /**
   * Execute one enabled agentic transition.
   * Returns the step result (fired, escalated, no_enabled_transitions, or error).
   */
  async step(instanceId: string, actorId: string): Promise<StepResult> {
    // 1. Find enabled agentic transitions
    const enabled = this.engine.getEnabledTransitions(instanceId, actorId);
    const agenticTransitions = enabled.filter((t) => t.mode === "agentic");

    if (agenticTransitions.length === 0) {
      return { outcome: "no_enabled_transitions", instance_id: instanceId };
    }

    const transition = agenticTransitions[0]!;
    return this.executeTransition(instanceId, actorId, transition);
  }

  /**
   * Loop step() until no agentic transitions remain or limits are hit.
   */
  async run(instanceId: string, actorId: string): Promise<RunResult> {
    const steps: StepResult[] = [];

    for (let i = 0; i < this.maxSteps; i++) {
      const result = await this.step(instanceId, actorId);
      steps.push(result);

      if (result.outcome === "no_enabled_transitions") {
        // Remove the last "no transitions" step from results — it's not a real step
        steps.pop();
        return { instance_id: instanceId, steps, final_outcome: "completed" };
      }

      if (result.outcome === "escalated") {
        return { instance_id: instanceId, steps, final_outcome: "escalated" };
      }

      if (result.outcome === "error") {
        return { instance_id: instanceId, steps, final_outcome: "error" };
      }
    }

    return { instance_id: instanceId, steps, final_outcome: "max_steps" };
  }

  private async executeTransition(
    instanceId: string,
    actorId: string,
    transition: Transition,
  ): Promise<StepResult> {
    // 2. Build work order
    const workOrder = buildWorkOrder(this.engine, instanceId, transition.id);

    // 3. Resolve institutional context
    const institutionalContext = this.options.contextStore
      ? this.options.contextStore.resolve(workOrder.context_sources)
      : {};

    // 4. Build prompts
    const systemPrompt = buildSystemPrompt(workOrder);
    const contextPrompt = buildContextPrompt(workOrder, institutionalContext);

    // 5. Resolve tools
    const tools: unknown[] = [];
    if (this.options.toolRegistry) {
      for (const toolName of workOrder.available_tools) {
        const tool = this.options.toolRegistry.get(toolName);
        if (tool) tools.push(tool);
      }
    }

    // 6. Execute with retry on runtime errors
    let evidence: ExecutionEvidence | undefined;
    let retries = 0;
    let lastError: string | undefined;

    while (retries <= this.maxRetries) {
      try {
        evidence = await this.options.executor(
          transition.id,
          systemPrompt,
          contextPrompt,
          tools,
        );
        break; // success — exit retry loop
      } catch (err) {
        lastError = err instanceof Error ? err.message : String(err);
        retries++;
        if (retries > this.maxRetries) {
          return {
            outcome: "error",
            transition_id: transition.id,
            instance_id: instanceId,
            retries_used: this.maxRetries,
            error: lastError,
          };
        }
      }
    }

    // 7. Verify postconditions
    const pcResults = await this.verifier.verify(
      workOrder.postconditions,
      evidence!,
    );

    const allMet = PostconditionVerifier.allRequiredMet(
      workOrder.postconditions,
      pcResults,
    );

    if (!allMet) {
      return {
        outcome: "escalated",
        transition_id: transition.id,
        instance_id: instanceId,
        postcondition_results: pcResults,
      };
    }

    // 8. Fire transition through the engine
    const firingResult = this.engine.fireTransition(
      instanceId,
      transition.id,
      actorId,
      evidence!.payload,
      undefined, // evidence — TODO: map from execution evidence
      `Agent executed transition. Postconditions verified: ${pcResults.map((r) => `${r.postcondition}=${r.satisfied}(${r.method})`).join(", ")}`,
    );

    return {
      outcome: firingResult.success ? "fired" : "error",
      transition_id: transition.id,
      instance_id: instanceId,
      firing_result: firingResult,
      postcondition_results: pcResults,
      retries_used: retries > 0 ? retries : undefined,
      error: firingResult.success ? undefined : firingResult.error,
    };
  }
}
