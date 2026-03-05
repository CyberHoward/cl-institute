import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "./engine.js";

describe("Engine — definition operations", () => {
  let engine: Engine;

  beforeEach(() => {
    engine = new Engine(":memory:");
  });

  afterEach(() => {
    engine.close();
  });

  describe("createInstitution", () => {
    it("creates an institution and returns it", () => {
      const inst = engine.createInstitution("ASADA Playas de Nosara", "Community water association");
      expect(inst.name).toBe("ASADA Playas de Nosara");
      expect(inst.description).toBe("Community water association");
      expect(inst.id).toBeTruthy();
      expect(inst.created_at).toBeTruthy();
    });
  });

  describe("createRole", () => {
    it("creates a role within an institution", () => {
      const inst = engine.createInstitution("ASADA");
      const role = engine.createRole(inst.id, "administrator", 2, "Manages operations");
      expect(role.name).toBe("administrator");
      expect(role.authority_level).toBe(2);
      expect(role.institution_id).toBe(inst.id);
    });

    it("rejects duplicate role names within the same institution", () => {
      const inst = engine.createInstitution("ASADA");
      engine.createRole(inst.id, "administrator", 2);
      expect(() => engine.createRole(inst.id, "administrator", 3)).toThrow();
    });
  });

  describe("createActor", () => {
    it("creates a human actor", () => {
      const inst = engine.createInstitution("ASADA");
      const actor = engine.createActor(inst.id, "Don Carlos", "human");
      expect(actor.name).toBe("Don Carlos");
      expect(actor.type).toBe("human");
    });

    it("creates an agent actor", () => {
      const inst = engine.createInstitution("ASADA");
      const actor = engine.createActor(inst.id, "carta-agent", "agent");
      expect(actor.type).toBe("agent");
    });
  });

  describe("assignRole", () => {
    it("assigns a role to an actor", () => {
      const inst = engine.createInstitution("ASADA");
      const role = engine.createRole(inst.id, "administrator", 2);
      const actor = engine.createActor(inst.id, "Don Carlos", "human");
      engine.assignRole(actor.id, role.id);
      const roles = engine.getActorRoles(actor.id);
      expect(roles).toHaveLength(1);
      expect(roles[0]!.name).toBe("administrator");
    });

    it("supports multiple roles per actor", () => {
      const inst = engine.createInstitution("ASADA");
      const r1 = engine.createRole(inst.id, "administrator", 2);
      const r2 = engine.createRole(inst.id, "secretary", 3);
      const actor = engine.createActor(inst.id, "Don Carlos", "human");
      engine.assignRole(actor.id, r1.id);
      engine.assignRole(actor.id, r2.id);
      const roles = engine.getActorRoles(actor.id);
      expect(roles).toHaveLength(2);
    });
  });
});

describe("Engine — net definition", () => {
  let engine: Engine;
  let instId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA").id;
  });

  afterEach(() => {
    engine.close();
  });

  describe("createNet", () => {
    it("creates a net within an institution", () => {
      const net = engine.createNet(instId, "Carta de Agua", "carta-de-agua");
      expect(net.name).toBe("Carta de Agua");
      expect(net.domain).toBe("carta-de-agua");
      expect(net.institution_id).toBe(instId);
    });
  });

  describe("addPlace", () => {
    it("adds a place to a net", () => {
      const net = engine.createNet(instId, "Carta de Agua");
      const place = engine.addPlace(net.id, "intake", "Request received");
      expect(place.id).toBe("intake");
      expect(place.net_id).toBe(net.id);
    });

    it("adds a place with a schema", () => {
      const net = engine.createNet(instId, "Carta de Agua");
      const schema = { type: "object", properties: { applicant: { type: "string" } } };
      const place = engine.addPlace(net.id, "intake", "Request received", schema);
      expect(place.schema).toEqual(schema);
    });
  });

  describe("addTransition", () => {
    it("adds a transition with arcs derived from consumes/produces", () => {
      const net = engine.createNet(instId, "Carta de Agua");
      engine.addPlace(net.id, "intake", "Request received");
      engine.addPlace(net.id, "documents-complete", "Docs verified");

      const transition = engine.addTransition(net.id, {
        id: "check-completeness",
        consumes: ["intake"],
        produces: ["documents-complete"],
        intent: "Review submitted docs against checklist",
        mode: "agentic",
        requires_authority: 2,
        context_sources: ["case-data"],
        postconditions: { required: ["docs-verified"] },
        evidence_requirements: [],
        available_tools: ["check-documents"],
      });

      expect(transition.id).toBe("check-completeness");
      expect(transition.consumes).toEqual(["intake"]);
      expect(transition.produces).toEqual(["documents-complete"]);
      expect(transition.mode).toBe("agentic");

      // Verify arcs were created
      const arcs = engine.getArcs(net.id);
      expect(arcs).toHaveLength(2);
      expect(arcs.find((a) => a.direction === "place_to_transition")).toBeTruthy();
      expect(arcs.find((a) => a.direction === "transition_to_place")).toBeTruthy();
    });

    it("rejects transition referencing non-existent place", () => {
      const net = engine.createNet(instId, "Carta de Agua");
      expect(() =>
        engine.addTransition(net.id, {
          id: "bad",
          consumes: ["nonexistent"],
          produces: [],
          intent: "test",
          mode: "deterministic",
          requires_authority: 0,
          context_sources: [],
          postconditions: { required: [] },
          evidence_requirements: [],
          available_tools: [],
        }),
      ).toThrow();
    });
  });

  describe("getNet", () => {
    it("returns net with places and transitions", () => {
      const net = engine.createNet(instId, "Carta de Agua");
      engine.addPlace(net.id, "intake", "Request received");
      engine.addPlace(net.id, "complete", "Docs verified");
      engine.addTransition(net.id, {
        id: "check",
        consumes: ["intake"],
        produces: ["complete"],
        intent: "Check docs",
        mode: "agentic",
        requires_authority: 2,
        context_sources: [],
        postconditions: { required: ["checked"] },
        evidence_requirements: [],
        available_tools: [],
      });

      const full = engine.getNetWithGraph(net.id);
      expect(full.places).toHaveLength(2);
      expect(full.transitions).toHaveLength(1);
    });
  });
});

describe("Engine — policies", () => {
  let engine: Engine;
  let instId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA").id;
  });

  afterEach(() => {
    engine.close();
  });

  it("attaches a policy to a scope", () => {
    const policy = engine.attachPolicy(instId, "carta-de-agua.check-scarcity", "constraint", "Source flow must be >= 4 L/s");
    expect(policy.scope).toBe("carta-de-agua.check-scarcity");
    expect(policy.strength).toBe("constraint");
  });

  it("retrieves policies ordered by strength then specificity", () => {
    engine.attachPolicy(instId, "carta-de-agua.*", "context", "Development pressure from tourism");
    engine.attachPolicy(instId, "carta-de-agua.check-scarcity", "constraint", "Source flow >= 4 L/s");
    engine.attachPolicy(instId, "carta-de-agua.*", "preference", "Be specific in notices");
    engine.attachPolicy(instId, "*", "context", "Global context");

    // Query for a specific transition scope
    const policies = engine.getPolicies("carta-de-agua.check-scarcity");

    // Constraints first, then preference, then context
    // Exact match before parent before global
    expect(policies[0]!.strength).toBe("constraint");
    expect(policies[0]!.scope).toBe("carta-de-agua.check-scarcity");

    // All 4 should be returned (exact + parent + global)
    expect(policies).toHaveLength(4);

    // Last should be global context
    expect(policies[policies.length - 1]!.scope).toBe("*");
  });

  it("returns empty array for unmatched scope", () => {
    engine.attachPolicy(instId, "procurement.*", "preference", "Prefer local vendors");
    const policies = engine.getPolicies("carta-de-agua.intake");
    expect(policies).toHaveLength(0);
  });
});

describe("Engine — runtime", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA").id;
    const net = engine.createNet(instId, "Carta de Agua", "carta-de-agua");
    netId = net.id;

    engine.addPlace(netId, "intake", "Request received");
    engine.addPlace(netId, "docs-complete", "Docs verified");
    engine.addPlace(netId, "triaged", "Case classified");

    engine.addTransition(netId, {
      id: "check-completeness",
      consumes: ["intake"],
      produces: ["docs-complete"],
      intent: "Verify all documents are present",
      mode: "agentic",
      requires_authority: 2,
      context_sources: [],
      postconditions: { required: ["docs-verified"] },
      evidence_requirements: [],
      available_tools: ["check-documents"],
    });

    engine.addTransition(netId, {
      id: "triage",
      consumes: ["docs-complete"],
      produces: ["triaged"],
      intent: "Classify case by impact level",
      mode: "judgment",
      requires_authority: 2,
      context_sources: [],
      postconditions: { required: ["classified"] },
      evidence_requirements: [],
      available_tools: [],
    });

    const role = engine.createRole(instId, "administrator", 2);
    const actor = engine.createActor(instId, "Don Carlos", "human");
    engine.assignRole(actor.id, role.id);
  });

  afterEach(() => {
    engine.close();
  });

  describe("instantiate", () => {
    it("creates an instance with a token in the start place", () => {
      const instance = engine.instantiate(netId, "intake", {
        applicant: "Juan Pérez",
      });
      expect(instance.status).toBe("running");
      const marking = engine.getMarking(instance.id);
      expect(marking.get("intake")).toHaveLength(1);
      expect(marking.get("intake")![0]!.payload["applicant"]).toBe("Juan Pérez");
    });
  });

  describe("getEnabledTransitions", () => {
    it("returns transitions whose input places have tokens", () => {
      const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });
      const actors = engine.listActors(instId);
      const actorId = actors[0]!.id;

      const enabled = engine.getEnabledTransitions(instance.id, actorId);
      expect(enabled).toHaveLength(1);
      expect(enabled[0]!.id).toBe("check-completeness");
    });

    it("respects authority — low authority actor sees nothing", () => {
      const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });
      const viewerRole = engine.createRole(instId, "viewer", 0);
      const viewer = engine.createActor(instId, "Viewer", "human");
      engine.assignRole(viewer.id, viewerRole.id);

      const enabled = engine.getEnabledTransitions(instance.id, viewer.id);
      expect(enabled).toHaveLength(0);
    });

    it("returns empty when no tokens in input places", () => {
      const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });
      const actors = engine.listActors(instId);
      const actorId = actors[0]!.id;

      const enabled = engine.getEnabledTransitions(instance.id, actorId);
      expect(enabled.every((t) => t.id !== "triage")).toBe(true);
    });
  });
});

describe("Engine — fireTransition", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let actorId: string;
  let roleId: string;

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
      intent: "Verify all documents are present",
      mode: "deterministic",
      requires_authority: 2,
      context_sources: [],
      postconditions: { required: [] },
      evidence_requirements: [],
      available_tools: [],
    });

    const role = engine.createRole(instId, "administrator", 2);
    roleId = role.id;
    const actor = engine.createActor(instId, "Don Carlos", "human");
    actorId = actor.id;
    engine.assignRole(actorId, roleId);
  });

  afterEach(() => {
    engine.close();
  });

  it("fires a transition: consumes input token, produces output token", () => {
    const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });
    const result = engine.fireTransition(instance.id, "check-completeness", actorId, {
      docs_verified: true,
    });
    expect(result.success).toBe(true);
    expect(result.tokens_consumed).toHaveLength(1);
    expect(result.tokens_produced).toHaveLength(1);

    const marking = engine.getMarking(instance.id);
    expect(marking.has("intake")).toBe(false);
    expect(marking.get("docs-complete")).toHaveLength(1);
    expect(marking.get("docs-complete")![0]!.payload["docs_verified"]).toBe(true);
  });

  it("writes an audit entry on fire", () => {
    const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });
    engine.fireTransition(instance.id, "check-completeness", actorId, {});
    const history = engine.getHistory(instance.id);
    expect(history.length).toBeGreaterThanOrEqual(2);
    expect(history.some((e) => e.action === "transition_fired")).toBe(true);
  });

  it("rejects firing when actor lacks authority", () => {
    const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });
    const lowRole = engine.createRole(instId, "viewer", 0);
    const lowActor = engine.createActor(instId, "Viewer", "human");
    engine.assignRole(lowActor.id, lowRole.id);

    const result = engine.fireTransition(instance.id, "check-completeness", lowActor.id, {});
    expect(result.success).toBe(false);
    expect(result.error).toMatch(/authority/i);
  });

  it("rejects firing when input place has no token", () => {
    const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });
    engine.fireTransition(instance.id, "check-completeness", actorId, {});
    const result = engine.fireTransition(instance.id, "check-completeness", actorId, {});
    expect(result.success).toBe(false);
    expect(result.error).toMatch(/token/i);
  });
});

describe("Engine — judgment points", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let adminId: string;
  let boardActorId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA").id;
    const net = engine.createNet(instId, "Carta de Agua", "carta-de-agua");
    netId = net.id;

    engine.addPlace(netId, "board-ready", "Packet assembled");
    engine.addPlace(netId, "decided", "Board has decided");

    engine.addTransition(netId, {
      id: "board-decision",
      consumes: ["board-ready"],
      produces: ["decided"],
      intent: "Board reviews case and makes decision",
      mode: "judgment",
      decision_type: "approval",
      requires_authority: 4,
      context_sources: ["case-data"],
      postconditions: { required: ["decision-made"] },
      evidence_requirements: [
        { id: "resolution-number", description: "Board resolution number", type: "reference", required: true },
      ],
      available_tools: [],
    });

    const adminRole = engine.createRole(instId, "administrator", 2);
    const admin = engine.createActor(instId, "Don Carlos", "human");
    engine.assignRole(admin.id, adminRole.id);
    adminId = admin.id;

    const boardRole = engine.createRole(instId, "junta-directiva", 4);
    const boardActor = engine.createActor(instId, "Board", "human");
    engine.assignRole(boardActor.id, boardRole.id);
    boardActorId = boardActor.id;

    engine.attachPolicy(instId, "carta-de-agua.board-decision", "constraint", "Board decision required for all approvals");
  });

  afterEach(() => {
    engine.close();
  });

  it("lists pending judgments with context", () => {
    const instance = engine.instantiate(netId, "board-ready", { case: "CDA-001" });
    const pending = engine.getPendingJudgments(instance.id);
    expect(pending).toHaveLength(1);
    expect(pending[0]!.transition_id).toBe("board-decision");
    expect(pending[0]!.transition_mode).toBe("judgment");
    expect(pending[0]!.requires_authority).toBe(4);
    expect(pending[0]!.token_payloads).toHaveLength(1);
    expect(pending[0]!.policies).toHaveLength(1);
  });

  it("resolves a judgment — fires the transition with decision payload", () => {
    const instance = engine.instantiate(netId, "board-ready", { case: "CDA-001" });
    const result = engine.resolveJudgment(
      instance.id,
      "board-decision",
      boardActorId,
      { decision: "approve", conditions: [] },
      "Capacity confirmed by technical report",
      [{ requirement_id: "resolution-number", type: "reference", content: "RES-2026-042", captured_at: new Date().toISOString() }],
    );
    expect(result.success).toBe(true);

    const marking = engine.getMarking(instance.id);
    expect(marking.has("decided")).toBe(true);
    expect(marking.get("decided")![0]!.payload["decision"]).toBe("approve");
  });

  it("rejects judgment resolution by unauthorized actor", () => {
    const instance = engine.instantiate(netId, "board-ready", { case: "CDA-001" });
    const result = engine.resolveJudgment(
      instance.id,
      "board-decision",
      adminId,
      { decision: "approve" },
      "I approve this",
    );
    expect(result.success).toBe(false);
    expect(result.error).toMatch(/authority/i);
  });
});
