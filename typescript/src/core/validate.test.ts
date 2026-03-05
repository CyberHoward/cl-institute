import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "./engine.js";
import { validateNet } from "./validate.js";

describe("validateNet", () => {
  let engine: Engine;
  let instId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA").id;
  });

  afterEach(() => {
    engine.close();
  });

  it("valid net passes validation", () => {
    const net = engine.createNet(instId, "Simple");
    engine.addPlace(net.id, "start", "Start");
    engine.addPlace(net.id, "end", "End");
    engine.addTransition(net.id, {
      id: "go",
      consumes: ["start"],
      produces: ["end"],
      intent: "Move forward",
      mode: "deterministic",
      requires_authority: 0,
      context_sources: [],
      postconditions: { required: [] },
      evidence_requirements: [],
      available_tools: [],
    });

    const result = validateNet(engine, net.id);
    expect(result.is_valid).toBe(true);
  });

  it("detects orphan places (no arcs)", () => {
    const net = engine.createNet(instId, "Orphan");
    engine.addPlace(net.id, "start", "Start");
    engine.addPlace(net.id, "orphan", "Orphan — no transitions connect here");
    engine.addPlace(net.id, "end", "End");
    engine.addTransition(net.id, {
      id: "go",
      consumes: ["start"],
      produces: ["end"],
      intent: "Move forward",
      mode: "deterministic",
      requires_authority: 0,
      context_sources: [],
      postconditions: { required: [] },
      evidence_requirements: [],
      available_tools: [],
    });

    const result = validateNet(engine, net.id);
    expect(result.violations.some((v) => v.constraint_name === "orphan_place")).toBe(true);
  });

  it("warns when judgment transition has no policies", () => {
    const net = engine.createNet(instId, "No Policy", "test-domain");
    engine.addPlace(net.id, "start", "Start");
    engine.addPlace(net.id, "end", "End");
    engine.addTransition(net.id, {
      id: "decide",
      consumes: ["start"],
      produces: ["end"],
      intent: "Make a judgment",
      mode: "judgment",
      decision_type: "approval",
      requires_authority: 4,
      context_sources: [],
      postconditions: { required: ["decided"] },
      evidence_requirements: [],
      available_tools: [],
    });

    const result = validateNet(engine, net.id);
    expect(result.violations.some((v) => v.constraint_name === "judgment_without_policy")).toBe(true);
  });
});
