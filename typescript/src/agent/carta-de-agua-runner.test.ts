import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "../core/engine.js";
import { PostconditionVerifier } from "./postconditions.js";
import { AgentRunner } from "./runner.js";

describe("Carta de Agua — AgentRunner integration", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let adminId: string;
  let boardId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA Nosara").id;

    const adminRole = engine.createRole(instId, "administrator", 2);
    const boardRole = engine.createRole(instId, "junta-directiva", 4);

    const admin = engine.createActor(instId, "Don Carlos", "human");
    engine.assignRole(admin.id, adminRole.id);
    adminId = admin.id;

    const board = engine.createActor(instId, "Junta Directiva", "human");
    engine.assignRole(board.id, boardRole.id);
    boardId = board.id;

    const net = engine.createNet(instId, "Carta de Agua", "carta-de-agua");
    netId = net.id;

    // Minimal workflow: intake → [receive-request] → docs-pending → [check-completeness] → docs-complete → [triage] → triaged
    engine.addPlace(netId, "intake", "Request received");
    engine.addPlace(netId, "docs-pending", "Awaiting documents");
    engine.addPlace(netId, "docs-complete", "Documents verified");
    engine.addPlace(netId, "triaged", "Case classified");

    engine.addTransition(netId, {
      id: "receive-request",
      consumes: ["intake"],
      produces: ["docs-pending"],
      intent: "Assign case ID, send receipt",
      mode: "deterministic",
      requires_authority: 2,
      context_sources: [],
      postconditions: { required: ["case-id-assigned"] },
      evidence_requirements: [],
      available_tools: [],
    });

    engine.addTransition(netId, {
      id: "check-completeness",
      consumes: ["docs-pending"],
      produces: ["docs-complete"],
      intent: "Verify documents against checklist",
      mode: "agentic",
      requires_authority: 2,
      context_sources: ["document-checklist"],
      postconditions: { required: ["all-docs-verified"] },
      evidence_requirements: [],
      available_tools: ["verify-documents"],
    });

    engine.addTransition(netId, {
      id: "triage-case",
      consumes: ["docs-complete"],
      produces: ["triaged"],
      intent: "Classify case by impact level",
      mode: "judgment",
      decision_type: "classification",
      requires_authority: 2,
      context_sources: [],
      postconditions: { required: ["classified"] },
      evidence_requirements: [],
      available_tools: [],
    });
  });

  afterEach(() => {
    engine.close();
  });

  it("runner fires agentic transitions, stops at judgment points", async () => {
    // 1. Start at intake
    const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });

    // 2. Manually fire the deterministic transition (runner doesn't handle these)
    engine.fireTransition(instance.id, "receive-request", adminId, {
      case_id: "CDA-001",
    });

    // 3. Now token is at docs-pending — runner should fire check-completeness
    const verifier = new PostconditionVerifier(
      new Map([["all-docs-verified", () => true]]),
    );

    const executor = async () => ({
      text: "All documents verified.",
      toolResults: [{ toolName: "verify-documents", result: { complete: true } }],
      payload: { complete: true, missing: [] },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.run(instance.id, adminId);

    // Should fire check-completeness, then stop (triage is judgment)
    expect(result.final_outcome).toBe("completed");
    expect(result.steps).toHaveLength(1);
    expect(result.steps[0]!.transition_id).toBe("check-completeness");

    // Token should now be at docs-complete
    const marking = engine.getMarking(instance.id);
    expect(marking.has("docs-complete")).toBe(true);

    // Triage should be pending as a judgment
    const pending = engine.getPendingJudgments(instance.id);
    expect(pending).toHaveLength(1);
    expect(pending[0]!.transition_id).toBe("triage-case");
  });

  it("audit trail captures agent execution details", async () => {
    const instance = engine.instantiate(netId, "docs-pending", { applicant: "María" });

    const verifier = new PostconditionVerifier(
      new Map([["all-docs-verified", () => true]]),
    );

    const executor = async () => ({
      text: "Verified.",
      toolResults: [],
      payload: { complete: true },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    await runner.step(instance.id, adminId);

    const history = engine.getHistory(instance.id);
    const agentEntry = history.find((e) => e.transition_id === "check-completeness");
    expect(agentEntry).toBeDefined();
    expect(agentEntry!.action).toBe("transition_fired");
    expect(agentEntry!.reasoning).toContain("Postconditions verified");
    expect(agentEntry!.reasoning).toContain("all-docs-verified=true(deterministic)");
  });
});
