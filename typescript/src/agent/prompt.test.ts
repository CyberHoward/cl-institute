import { describe, it, expect } from "vitest";
import { buildSystemPrompt, buildContextPrompt } from "./prompt.js";
import type { WorkOrder } from "../core/context.js";
import type { Policy } from "../core/types.js";

const makeWorkOrder = (overrides?: Partial<WorkOrder>): WorkOrder => ({
  transition_id: "check-docs",
  instance_id: "inst-1",
  mode: "agentic",
  intent: "Verify all required documents are present",
  token_payloads: [{ applicant: "Juan", doc_type: "residential" }],
  policies: [
    {
      id: "p1",
      institution_id: "i1",
      scope: "carta-de-agua.check-docs",
      strength: "constraint",
      text: "All documents must be verified against the official checklist.",
      created_at: "",
      updated_at: "",
    } satisfies Policy,
  ],
  context_sources: ["document-checklist"],
  postconditions: {
    required: ["all-docs-verified"],
    desired: ["deficiency-notice-sent-if-incomplete"],
  },
  evidence_requirements: [
    { id: "checklist", description: "Completed checklist", type: "artifact", required: true },
  ],
  available_tools: ["verify-documents", "send-notification"],
  ...overrides,
});

describe("buildSystemPrompt", () => {
  it("includes intent, postconditions, policies, evidence requirements, and tools", () => {
    const prompt = buildSystemPrompt(makeWorkOrder());

    expect(prompt).toContain("Verify all required documents are present");
    expect(prompt).toContain("all-docs-verified");
    expect(prompt).toContain("deficiency-notice-sent-if-incomplete");
    expect(prompt).toContain("official checklist");
    expect(prompt).toContain("Completed checklist");
    expect(prompt).toContain("verify-documents");
    expect(prompt).toContain("send-notification");
  });

  it("includes output schema when present", () => {
    const wo = makeWorkOrder({
      output_schema: { type: "object", properties: { complete: { type: "boolean" } } },
    });
    const prompt = buildSystemPrompt(wo);
    expect(prompt).toContain('"complete"');
  });

  it("labels policy strengths", () => {
    const prompt = buildSystemPrompt(makeWorkOrder());
    expect(prompt).toMatch(/constraint/i);
  });
});

describe("buildContextPrompt", () => {
  it("includes token payloads and institutional context", () => {
    const wo = makeWorkOrder();
    const institutionalContext = { "document-checklist": "1. Request form\n2. Cadastral plan" };

    const prompt = buildContextPrompt(wo, institutionalContext);
    expect(prompt).toContain("Juan");
    expect(prompt).toContain("Cadastral plan");
  });

  it("handles empty institutional context", () => {
    const wo = makeWorkOrder();
    const prompt = buildContextPrompt(wo, {});
    expect(prompt).toContain("Juan");
  });
});
