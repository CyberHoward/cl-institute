import { describe, it, expect } from "vitest";
import type {
  Institution,
  Role,
  Transition,
  Place,
  Token,
  AuditEntry,
} from "./types.js";

describe("core types", () => {
  it("can construct an Institution", () => {
    const inst: Institution = {
      id: "asada-1",
      name: "ASADA Playas de Nosara",
      description: "Community water association",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    expect(inst.name).toBe("ASADA Playas de Nosara");
  });

  it("can construct a Transition with full institutional metadata", () => {
    const t: Transition = {
      id: "board-decision",
      net_id: "carta-de-agua",
      consumes: ["board-ready"],
      produces: ["decided"],
      intent: "Board reviews case and issues decision",
      mode: "judgment",
      decision_type: "approval",
      requires_authority: 4,
      context_sources: ["case-data", "technical-report"],
      postconditions: {
        required: ["decision-made", "rationale-documented"],
        desired: ["conditions-specified-if-applicable"],
        escalation: ["escalate-to-aya"],
      },
      evidence_requirements: [
        {
          id: "board-resolution",
          description: "Board resolution number",
          type: "reference",
          required: true,
        },
      ],
      available_tools: [],
      input_schema: { type: "object", properties: { caseId: { type: "string" } } },
      output_schema: {
        type: "object",
        properties: {
          decision: { type: "string", enum: ["approve", "deny", "conditional", "defer"] },
          rationale: { type: "string" },
        },
      },
    };
    expect(t.mode).toBe("judgment");
    expect(t.requires_authority).toBe(4);
    expect(t.evidence_requirements).toHaveLength(1);
  });

  it("can construct a Token with unstructured payload", () => {
    const token: Token = {
      id: "tok-1",
      instance_id: "inst-1",
      place_id: "intake",
      payload: {
        applicant: "Juan Pérez",
        cadastral_plan: "SJ-12345",
        channel: "whatsapp",
      },
      created_at: new Date().toISOString(),
    };
    expect(token.payload["applicant"]).toBe("Juan Pérez");
  });
});
