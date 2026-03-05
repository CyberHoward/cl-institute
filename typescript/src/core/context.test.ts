import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "./engine.js";
import { buildWorkOrder } from "./context.js";

describe("buildWorkOrder", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let instanceId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA").id;
    const net = engine.createNet(instId, "Carta de Agua", "carta-de-agua");
    netId = net.id;

    engine.addPlace(netId, "intake", "Request received");
    engine.addPlace(netId, "docs-complete", "Docs verified");

    engine.addTransition(netId, {
      id: "check-completeness",
      consumes: ["intake"],
      produces: ["docs-complete"],
      intent: "Verify all required documents are present in the submission",
      mode: "agentic",
      requires_authority: 2,
      context_sources: ["checklist"],
      postconditions: {
        required: ["all-docs-present"],
        desired: ["docs-quality-verified"],
      },
      evidence_requirements: [
        { id: "checklist-result", description: "Completed document checklist", type: "artifact", required: true },
      ],
      available_tools: ["check-documents"],
      input_schema: { type: "object", properties: { applicant: { type: "string" } } },
      output_schema: { type: "object", properties: { complete: { type: "boolean" }, missing: { type: "array" } } },
    });

    engine.attachPolicy(instId, "carta-de-agua.check-completeness", "procedure", "Minimum documents: request form + cadastral plan");
    engine.attachPolicy(instId, "carta-de-agua.*", "preference", "Be specific about missing items");

    const instance = engine.instantiate(netId, "intake", { applicant: "Juan Pérez", cadastral_plan: null });
    instanceId = instance.id;
  });

  afterEach(() => {
    engine.close();
  });

  it("assembles a complete work order from net state", () => {
    const workOrder = buildWorkOrder(engine, instanceId, "check-completeness");

    // 1. Intent
    expect(workOrder.intent).toContain("Verify all required documents");

    // 2. Token payloads
    expect(workOrder.token_payloads).toHaveLength(1);
    expect(workOrder.token_payloads[0]!["applicant"]).toBe("Juan Pérez");

    // 3. Input schema
    expect(workOrder.input_schema).toBeDefined();

    // 4. Policies (ordered by strength)
    expect(workOrder.policies).toHaveLength(2);
    expect(workOrder.policies[0]!.strength).toBe("procedure");

    // 5. Context sources
    expect(workOrder.context_sources).toEqual(["checklist"]);

    // 6. Output schema
    expect(workOrder.output_schema).toBeDefined();

    // 7. Postconditions
    expect(workOrder.postconditions.required).toContain("all-docs-present");

    // 8. Evidence requirements
    expect(workOrder.evidence_requirements).toHaveLength(1);

    // 9. Available tools
    expect(workOrder.available_tools).toContain("check-documents");

    // Mode
    expect(workOrder.mode).toBe("agentic");
  });
});
