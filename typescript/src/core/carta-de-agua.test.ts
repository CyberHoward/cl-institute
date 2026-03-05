import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "./engine.js";
import { buildWorkOrder } from "./context.js";
import { validateNet } from "./validate.js";

describe("Carta de Agua — end-to-end", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let adminId: string;
  let techId: string;
  let boardId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");

    // Institution
    instId = engine.createInstitution(
      "ASADA Playas de Nosara",
      "Community water association — Nosara, Guanacaste",
    ).id;

    // Roles
    const adminRole = engine.createRole(instId, "administrator", 2, "Manages daily operations");
    const techRole = engine.createRole(instId, "technical-operator", 2, "Conducts inspections and technical reviews");
    const boardRole = engine.createRole(instId, "junta-directiva", 4, "Board of directors — final decision authority");
    const presidentRole = engine.createRole(instId, "president", 3, "Signs official letters");

    // Actors
    const admin = engine.createActor(instId, "Don Carlos Mora", "human");
    engine.assignRole(admin.id, adminRole.id);
    adminId = admin.id;

    const tech = engine.createActor(instId, "Technical Operator", "human");
    engine.assignRole(tech.id, techRole.id);
    techId = tech.id;

    const board = engine.createActor(instId, "Junta Directiva", "human");
    engine.assignRole(board.id, boardRole.id);
    boardId = board.id;

    // Net
    const net = engine.createNet(instId, "Carta de Agua", "carta-de-agua", "Water availability letter process");
    netId = net.id;

    // Places
    engine.addPlace(netId, "intake", "Request received, case ID assigned");
    engine.addPlace(netId, "documents-pending", "Awaiting missing documents");
    engine.addPlace(netId, "documents-complete", "All required documents received");
    engine.addPlace(netId, "triaged", "Case classified by impact level");
    engine.addPlace(netId, "scarcity-hold", "Case held due to source stress");
    engine.addPlace(netId, "technical-review-ready", "Ready for technical assessment");
    engine.addPlace(netId, "board-ready", "Board packet assembled");
    engine.addPlace(netId, "decided", "Board has issued decision");
    engine.addPlace(netId, "delivered", "Decision letter delivered to applicant");

    // Transitions
    engine.addTransition(netId, {
      id: "receive-request",
      consumes: ["intake"],
      produces: ["documents-pending"],
      intent: "Assign case ID, timestamp, send receipt to applicant via their contact channel",
      mode: "deterministic",
      requires_authority: 2,
      context_sources: [],
      postconditions: { required: ["case-id-assigned", "receipt-sent"] },
      evidence_requirements: [
        { id: "case-id", description: "Assigned case ID", type: "reference", required: true },
        { id: "receipt-confirmation", description: "Receipt delivery confirmation", type: "artifact", required: true },
      ],
      available_tools: ["assign-case-id", "send-receipt"],
    });

    engine.addTransition(netId, {
      id: "check-completeness",
      consumes: ["documents-pending"],
      produces: ["documents-complete"],
      intent: "Review submitted documents against required checklist: request form + cadastral plan. Check subscriber payment status if existing abonado.",
      mode: "agentic",
      requires_authority: 2,
      context_sources: ["document-checklist"],
      postconditions: { required: ["all-required-docs-present"] },
      evidence_requirements: [
        { id: "checklist-result", description: "Completed document verification checklist", type: "artifact", required: true },
      ],
      available_tools: ["verify-documents", "check-payment-status"],
      output_schema: {
        type: "object",
        properties: {
          complete: { type: "boolean" },
          missing_items: { type: "array" },
          payment_current: { type: "boolean" },
        },
      },
    });

    engine.addTransition(netId, {
      id: "triage-case",
      consumes: ["documents-complete"],
      produces: ["triaged"],
      intent: "Classify case by impact level: residential, commercial, or high-impact (hotel/large development). Determines scrutiny path.",
      mode: "judgment",
      decision_type: "classification",
      requires_authority: 2,
      context_sources: ["case-data", "cadastral-info"],
      postconditions: { required: ["impact-level-classified"] },
      evidence_requirements: [
        { id: "classification-rationale", description: "Reason for classification", type: "attestation", required: true },
      ],
      available_tools: [],
      output_schema: {
        type: "object",
        properties: {
          impact_level: { type: "string", enum: ["residential", "commercial", "high-impact"] },
          rationale: { type: "string" },
        },
      },
    });

    engine.addTransition(netId, {
      id: "check-scarcity",
      consumes: ["triaged"],
      produces: ["technical-review-ready"],
      intent: "Check current source flow against scarcity threshold. If below 4 L/s, route to hold.",
      mode: "deterministic",
      requires_authority: 2,
      context_sources: ["source-flow-data"],
      postconditions: { required: ["scarcity-status-determined"] },
      evidence_requirements: [
        { id: "flow-reading", description: "Current source flow measurement", type: "artifact", required: true },
      ],
      available_tools: ["read-flow-meter"],
    });

    engine.addTransition(netId, {
      id: "compile-board-packet",
      consumes: ["technical-review-ready"],
      produces: ["board-ready"],
      intent: "Assemble board packet: request, all evidence, technical verification note, inspection report (if any), and administrator recommendation.",
      mode: "agentic",
      requires_authority: 2,
      context_sources: ["case-data", "technical-report", "inspection-report"],
      postconditions: { required: ["packet-assembled", "recommendation-included"] },
      evidence_requirements: [
        { id: "board-packet", description: "Complete board review packet", type: "artifact", required: true },
      ],
      available_tools: ["generate-document", "compile-packet"],
    });

    engine.addTransition(netId, {
      id: "board-decision",
      consumes: ["board-ready"],
      produces: ["decided"],
      intent: "Board reviews case packet and issues decision: approve, deny, conditional approval, or defer for more information.",
      mode: "judgment",
      decision_type: "approval",
      requires_authority: 4,
      context_sources: ["board-packet", "precedent"],
      postconditions: {
        required: ["decision-issued", "rationale-documented"],
        desired: ["conditions-specified-if-conditional"],
        escalation: ["escalate-to-aya"],
      },
      evidence_requirements: [
        { id: "board-resolution", description: "Board resolution number", type: "reference", required: true },
        { id: "vote-record", description: "Record of board vote", type: "artifact", required: true },
      ],
      available_tools: [],
      output_schema: {
        type: "object",
        properties: {
          decision: { type: "string", enum: ["approve", "deny", "conditional", "defer"] },
          conditions: { type: "array" },
          rationale: { type: "string" },
        },
      },
    });

    engine.addTransition(netId, {
      id: "deliver-decision",
      consumes: ["decided"],
      produces: ["delivered"],
      intent: "Generate official letter with decision, reasoning, conditions (if any), and appeal instructions. Obtain required signatures. Deliver via applicant's preferred channel.",
      mode: "agentic",
      requires_authority: 2,
      context_sources: ["decision-data", "applicant-contact"],
      postconditions: { required: ["letter-generated", "letter-signed", "letter-delivered"] },
      evidence_requirements: [
        { id: "signed-letter", description: "Signed decision letter", type: "artifact", required: true },
        { id: "delivery-confirmation", description: "Delivery confirmation to applicant", type: "artifact", required: true },
      ],
      available_tools: ["generate-document", "send-notification", "request-signature"],
    });

    // Policies
    engine.attachPolicy(instId, "carta-de-agua.*", "constraint",
      "Every request must receive a case ID and receipt before any processing begins.");
    engine.attachPolicy(instId, "carta-de-agua.check-scarcity", "constraint",
      "Source flow must be at or above 4 L/s to proceed with new connection approvals.");
    engine.attachPolicy(instId, "carta-de-agua.board-decision", "constraint",
      "Only the Junta Directiva may approve or deny carta de agua requests.");
    engine.attachPolicy(instId, "carta-de-agua.deliver-decision", "procedure",
      "Decision letter must include: basis for decision, conditions (if any), and appeal instructions.");
    engine.attachPolicy(instId, "carta-de-agua.check-completeness", "preference",
      "Deficiency notices should be specific: list each missing document individually, not 'documents are missing'.");
    engine.attachPolicy(instId, "carta-de-agua.triage-case", "preference",
      "High-impact projects (hotels, multi-unit developments) should receive additional scrutiny.");
    engine.attachPolicy(instId, "carta-de-agua.*", "context",
      "Development pressure from tourism and investment is the central tension. The administrator's obligation is to existing residents who depend on this water.");
  });

  afterEach(() => {
    engine.close();
  });

  it("validates the net structure", () => {
    const result = validateNet(engine, netId);
    // Should be valid (no error-severity violations)
    expect(result.is_valid).toBe(true);
  });

  it("runs a residential case through the full process", () => {
    // 1. Intake
    const instance = engine.instantiate(netId, "intake", {
      applicant: "Juan Pérez",
      channel: "whatsapp",
      phone: "+506-8888-1234",
      property: "Lote 45, Playa Guiones",
      cadastral_plan: "GN-2026-0045",
      request_type: "residential",
    });

    // 2. Receive request (deterministic — admin)
    const r1 = engine.fireTransition(instance.id, "receive-request", adminId, {
      case_id: "CDA-2026-001",
      receipt_sent: true,
      receipt_channel: "whatsapp",
    });
    expect(r1.success).toBe(true);

    // 3. Check completeness (agentic — admin)
    const r2 = engine.fireTransition(instance.id, "check-completeness", adminId, {
      complete: true,
      missing_items: [],
      payment_current: true,
    });
    expect(r2.success).toBe(true);

    // 4. Triage (judgment — admin classifies)
    const workOrder = buildWorkOrder(engine, instance.id, "triage-case");
    expect(workOrder.mode).toBe("judgment");
    expect(workOrder.policies.length).toBeGreaterThan(0);

    const r3 = engine.resolveJudgment(
      instance.id,
      "triage-case",
      adminId,
      { impact_level: "residential", rationale: "Single-unit residential on connected street" },
      "Standard residential request, no additional scrutiny needed",
      [{ requirement_id: "classification-rationale", type: "attestation", content: "Single-unit residential", captured_at: new Date().toISOString() }],
    );
    expect(r3.success).toBe(true);

    // 5. Check scarcity (deterministic — admin)
    const r4 = engine.fireTransition(instance.id, "check-scarcity", adminId, {
      source_flow_lps: 6.2,
      scarcity_status: "normal",
    });
    expect(r4.success).toBe(true);

    // 6. Compile board packet (agentic — admin)
    const r5 = engine.fireTransition(instance.id, "compile-board-packet", adminId, {
      packet_complete: true,
      recommendation: "approve",
    });
    expect(r5.success).toBe(true);

    // 7. Board decision (judgment — board only, authority 4)
    const pending = engine.getPendingJudgments(instance.id);
    expect(pending).toHaveLength(1);
    expect(pending[0]!.transition_id).toBe("board-decision");

    // Admin can't do this
    const r6fail = engine.resolveJudgment(
      instance.id,
      "board-decision",
      adminId,
      { decision: "approve" },
      "I approve",
    );
    expect(r6fail.success).toBe(false);

    // Board can
    const r6 = engine.resolveJudgment(
      instance.id,
      "board-decision",
      boardId,
      { decision: "approve", conditions: [], rationale: "Capacity confirmed, residential single-unit" },
      "Unanimous approval",
      [
        { requirement_id: "board-resolution", type: "reference", content: "RES-2026-042", captured_at: new Date().toISOString() },
        { requirement_id: "vote-record", type: "artifact", content: "5-0 unanimous", captured_at: new Date().toISOString() },
      ],
    );
    expect(r6.success).toBe(true);

    // 8. Deliver decision (agentic — admin)
    const r7 = engine.fireTransition(instance.id, "deliver-decision", adminId, {
      letter_generated: true,
      letter_signed: true,
      delivered_via: "whatsapp",
    });
    expect(r7.success).toBe(true);

    // Verify final state
    const marking = engine.getMarking(instance.id);
    expect(marking.has("delivered")).toBe(true);
    expect(marking.has("intake")).toBe(false);

    // Verify audit trail
    const history = engine.getHistory(instance.id);
    expect(history.length).toBeGreaterThanOrEqual(8); // instance_created + 7 transitions
    expect(history.filter((e) => e.action === "transition_fired")).toHaveLength(7);

    // Verify board decision is recorded with evidence
    const boardEntry = history.find((e) => e.transition_id === "board-decision");
    expect(boardEntry).toBeDefined();
    expect(boardEntry!.evidence).toBeDefined();
    expect(boardEntry!.evidence!.length).toBe(2);
  });

  it("blocks low-authority actors from board decisions", () => {
    const instance = engine.instantiate(netId, "board-ready", { case: "CDA-test" });
    const enabled = engine.getEnabledTransitions(instance.id, adminId);
    // Admin (authority 2) should NOT see board-decision (requires 4)
    expect(enabled.some((t) => t.id === "board-decision")).toBe(false);

    // Board (authority 4) should see it
    const boardEnabled = engine.getEnabledTransitions(instance.id, boardId);
    expect(boardEnabled.some((t) => t.id === "board-decision")).toBe(true);
  });

  it("assembles work order with policies for agentic transitions", () => {
    const instance = engine.instantiate(netId, "documents-pending", { applicant: "María" });
    const workOrder = buildWorkOrder(engine, instance.id, "check-completeness");

    expect(workOrder.intent).toContain("cadastral plan");
    expect(workOrder.available_tools).toContain("verify-documents");
    expect(workOrder.evidence_requirements).toHaveLength(1);
    expect(workOrder.policies.length).toBeGreaterThan(0);
    // Should have the specific preference about being specific in deficiency notices
    expect(workOrder.policies.some((p) => p.text.includes("specific"))).toBe(true);
  });
});
