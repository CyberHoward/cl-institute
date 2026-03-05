import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { DB } from "./db.js";

describe("DB", () => {
  let db: DB;

  beforeEach(() => {
    db = new DB(":memory:");
  });

  afterEach(() => {
    db.close();
  });

  it("creates tables on initialization", () => {
    const tables = db.listTables();
    expect(tables).toContain("institutions");
    expect(tables).toContain("roles");
    expect(tables).toContain("actors");
    expect(tables).toContain("actor_roles");
    expect(tables).toContain("nets");
    expect(tables).toContain("places");
    expect(tables).toContain("transitions");
    expect(tables).toContain("arcs");
    expect(tables).toContain("policies");
    expect(tables).toContain("instances");
    expect(tables).toContain("tokens");
    expect(tables).toContain("audit_entries");
  });
});
