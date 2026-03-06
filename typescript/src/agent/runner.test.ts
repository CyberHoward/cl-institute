import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "../core/engine.js";
import { InstitutionalContextStore } from "../core/context-store.js";
import { PostconditionVerifier } from "./postconditions.js";
import { AgentRunner } from "./runner.js";
import type { StepResult } from "../core/types.js";

/**
 * Mock agent executor — replaces actual LLM calls.
 * Returns canned responses keyed by transition ID.
 */
function mockExecutor(responses: Record<string, {
  text: string;
  toolResults: Array<{ toolName: string; result: Record<string, unknown> }>;
  payload: Record<string, unknown>;
}>) {
  return async (transitionId: string, _systemPrompt: string, _contextPrompt: string, _tools: unknown[]) => {
    const response = responses[transitionId];
    if (!response) throw new Error(`No mock response for transition: ${transitionId}`);
    return response;
  };
}

/** Mock executor that throws a runtime error */
function throwingExecutor(errorMessage: string) {
  return async () => {
    throw new Error(errorMessage);
  };
}

describe("AgentRunner", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let actorId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    const inst = engine.createInstitution("Test Org");
    instId = inst.id;

    const role = engine.createRole(instId, "operator", 5);
    const actor = engine.createActor(instId, "Agent Bot", "agent");
    engine.assignRole(actor.id, role.id);
    actorId = actor.id;

    const net = engine.createNet(instId, "Test Flow", "test");
    netId = net.id;

    engine.addPlace(netId, "start", "Starting place");
    engine.addPlace(netId, "middle", "Middle place");
    engine.addPlace(netId, "end", "End place");

    engine.addTransition(netId, {
      id: "step-one",
      consumes: ["start"],
      produces: ["middle"],
      intent: "Do the first thing",
      mode: "agentic",
      requires_authority: 1,
      context_sources: [],
      postconditions: { required: ["thing-done"] },
      evidence_requirements: [],
      available_tools: ["tool-a"],
    });

    engine.addTransition(netId, {
      id: "step-two",
      consumes: ["middle"],
      produces: ["end"],
      intent: "Do the second thing",
      mode: "agentic",
      requires_authority: 1,
      context_sources: [],
      postconditions: { required: ["second-done"] },
      evidence_requirements: [],
      available_tools: ["tool-b"],
    });
  });

  afterEach(() => {
    engine.close();
  });

  it("step() fires one agentic transition and returns result", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([["thing-done", (): boolean => true]]),
    );

    const executor = mockExecutor({
      "step-one": {
        text: "Done",
        toolResults: [{ toolName: "tool-a", result: { ok: true } }],
        payload: { step_one_output: true },
      },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.step(instance.id, actorId);

    expect(result.outcome).toBe("fired");
    expect(result.transition_id).toBe("step-one");
    expect(result.firing_result?.success).toBe(true);
    expect(result.postcondition_results?.[0]?.satisfied).toBe(true);

    // Token should now be in "middle"
    const marking = engine.getMarking(instance.id);
    expect(marking.has("middle")).toBe(true);
    expect(marking.has("start")).toBe(false);
  });

  it("step() returns no_enabled_transitions when none available", async () => {
    const instance = engine.instantiate(netId, "end", { done: true });

    const verifier = new PostconditionVerifier(new Map());
    const runner = new AgentRunner(engine, verifier, { executor: mockExecutor({}) });

    const result = await runner.step(instance.id, actorId);
    expect(result.outcome).toBe("no_enabled_transitions");
  });

  it("step() skips judgment transitions", async () => {
    // Add a judgment transition from start
    engine.addPlace(netId, "judgment-out", "After judgment");
    engine.addTransition(netId, {
      id: "judge-it",
      consumes: ["start"],
      produces: ["judgment-out"],
      intent: "Make a judgment call",
      mode: "judgment",
      decision_type: "approval",
      requires_authority: 1,
      context_sources: [],
      postconditions: { required: ["decided"] },
      evidence_requirements: [],
      available_tools: [],
    });

    // Both judge-it and step-one consume from "start", but runner should only fire agentic
    const instance = engine.instantiate(netId, "start", { data: "test" });

    const verifier = new PostconditionVerifier(
      new Map([["thing-done", (): boolean => true]]),
    );
    const executor = mockExecutor({
      "step-one": {
        text: "Done",
        toolResults: [],
        payload: { done: true },
      },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.step(instance.id, actorId);

    // Should fire step-one (agentic), not judge-it (judgment)
    expect(result.outcome).toBe("fired");
    expect(result.transition_id).toBe("step-one");
  });

  it("run() executes all agentic transitions until none remain", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([
        ["thing-done", (): boolean => true],
        ["second-done", (): boolean => true],
      ]),
    );

    const executor = mockExecutor({
      "step-one": { text: "Done 1", toolResults: [], payload: { step: 1 } },
      "step-two": { text: "Done 2", toolResults: [], payload: { step: 2 } },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.run(instance.id, actorId);

    expect(result.final_outcome).toBe("completed");
    expect(result.steps).toHaveLength(2);
    expect(result.steps[0]!.transition_id).toBe("step-one");
    expect(result.steps[1]!.transition_id).toBe("step-two");

    // Final marking should be at "end"
    const marking = engine.getMarking(instance.id);
    expect(marking.has("end")).toBe(true);
  });

  it("escalates when postconditions fail", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([["thing-done", (): boolean => false]]), // always fails
    );

    const executor = mockExecutor({
      "step-one": { text: "Tried", toolResults: [], payload: {} },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.step(instance.id, actorId);

    expect(result.outcome).toBe("escalated");
    expect(result.transition_id).toBe("step-one");
    // Token should NOT have moved
    const marking = engine.getMarking(instance.id);
    expect(marking.has("start")).toBe(true);
    expect(marking.has("middle")).toBe(false);
  });

  it("retries on runtime error then reports error after max retries", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(new Map());
    const executor = throwingExecutor("Network timeout");

    const runner = new AgentRunner(engine, verifier, { executor, maxRetries: 2 });
    const result = await runner.step(instance.id, actorId);

    expect(result.outcome).toBe("error");
    expect(result.retries_used).toBe(2);
    expect(result.error).toContain("Network timeout");

    // Token should NOT have moved
    const marking = engine.getMarking(instance.id);
    expect(marking.has("start")).toBe(true);
  });

  it("run() stops at max steps", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([
        ["thing-done", (): boolean => true],
        ["second-done", (): boolean => true],
      ]),
    );

    const executor = mockExecutor({
      "step-one": { text: "Done 1", toolResults: [], payload: { step: 1 } },
      "step-two": { text: "Done 2", toolResults: [], payload: { step: 2 } },
    });

    const runner = new AgentRunner(engine, verifier, { executor, maxSteps: 1 });
    const result = await runner.run(instance.id, actorId);

    expect(result.final_outcome).toBe("max_steps");
    expect(result.steps).toHaveLength(1);
  });

  it("run() reports escalated when postcondition fails mid-run", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([
        ["thing-done", (): boolean => true],
        ["second-done", (): boolean => false], // fails on step two
      ]),
    );

    const executor = mockExecutor({
      "step-one": { text: "Done 1", toolResults: [], payload: { step: 1 } },
      "step-two": { text: "Tried", toolResults: [], payload: {} },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.run(instance.id, actorId);

    expect(result.final_outcome).toBe("escalated");
    expect(result.steps).toHaveLength(2);
    expect(result.steps[0]!.outcome).toBe("fired");
    expect(result.steps[1]!.outcome).toBe("escalated");
  });
});
