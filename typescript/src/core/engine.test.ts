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
