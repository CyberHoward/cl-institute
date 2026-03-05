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
