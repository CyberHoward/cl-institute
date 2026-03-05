import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { DB } from "./db.js";
import { AuditLog } from "./audit.js";

describe("AuditLog", () => {
  let db: DB;
  let audit: AuditLog;

  beforeEach(() => {
    db = new DB(":memory:");
    // Disable FK checks for isolated audit unit tests
    db.sqlite.pragma("foreign_keys = OFF");
    audit = new AuditLog(db);
  });

  afterEach(() => {
    db.close();
  });

  it("appends an entry with hash chaining", () => {
    const entry = audit.append({
      instance_id: "inst-1",
      action: "instance_created",
      actor: { actor_id: "actor-1", role_id: "role-1", authority_level: 2 },
    });
    expect(entry.sequence).toBe(1);
    expect(entry.prev_hash).toBe("GENESIS");
    expect(entry.entry_hash).toBeTruthy();
  });

  it("chains entries via prev_hash", () => {
    const e1 = audit.append({
      instance_id: "inst-1",
      action: "instance_created",
      actor: { actor_id: "a1", role_id: "r1", authority_level: 2 },
    });
    const e2 = audit.append({
      instance_id: "inst-1",
      action: "transition_fired",
      actor: { actor_id: "a1", role_id: "r1", authority_level: 2 },
      transition_id: "check-completeness",
    });
    expect(e2.prev_hash).toBe(e1.entry_hash);
    expect(e2.sequence).toBe(2);
  });

  it("verifies chain integrity", () => {
    audit.append({
      instance_id: "inst-1",
      action: "instance_created",
      actor: { actor_id: "a1", role_id: "r1", authority_level: 2 },
    });
    audit.append({
      instance_id: "inst-1",
      action: "transition_fired",
      actor: { actor_id: "a1", role_id: "r1", authority_level: 2 },
    });
    const verification = audit.verifyChain("inst-1");
    expect(verification.valid).toBe(true);
    expect(verification.entries_checked).toBe(2);
  });

  it("retrieves entries by instance", () => {
    audit.append({
      instance_id: "inst-1",
      action: "instance_created",
      actor: { actor_id: "a1", role_id: "r1", authority_level: 2 },
    });
    audit.append({
      instance_id: "inst-2",
      action: "instance_created",
      actor: { actor_id: "a2", role_id: "r2", authority_level: 4 },
    });
    const entries = audit.getEntries("inst-1");
    expect(entries).toHaveLength(1);
  });
});
