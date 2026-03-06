import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "./engine.js";
import { InstitutionalContextStore } from "./context-store.js";

describe("InstitutionalContextStore", () => {
  let engine: Engine;
  let instId: string;
  let store: InstitutionalContextStore;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("Test Org").id;
    store = new InstitutionalContextStore(engine, instId);
  });

  afterEach(() => {
    engine.close();
  });

  it("sets and gets a value", () => {
    store.set("contacts.secretary.email", "maria@example.com");
    expect(store.get("contacts.secretary.email")).toBe("maria@example.com");
  });

  it("returns undefined for missing keys", () => {
    expect(store.get("nonexistent")).toBeUndefined();
  });

  it("overwrites existing values", () => {
    store.set("config.sms_gateway", "old");
    store.set("config.sms_gateway", "new");
    expect(store.get("config.sms_gateway")).toBe("new");
  });

  it("stores and retrieves complex objects", () => {
    const obj = { endpoint: "https://api.example.com", timeout: 5000 };
    store.set("config.api", obj);
    expect(store.get("config.api")).toEqual(obj);
  });

  it("resolves multiple keys at once", () => {
    store.set("contacts.admin.phone", "+506-1111");
    store.set("contacts.admin.email", "admin@example.com");
    store.set("config.org_name", "Test Org");

    const resolved = store.resolve(["contacts.admin.phone", "config.org_name", "missing.key"]);
    expect(resolved).toEqual({
      "contacts.admin.phone": "+506-1111",
      "config.org_name": "Test Org",
    });
    expect(resolved["missing.key"]).toBeUndefined();
  });

  it("deletes a key", () => {
    store.set("temp.value", "hello");
    expect(store.get("temp.value")).toBe("hello");
    store.delete("temp.value");
    expect(store.get("temp.value")).toBeUndefined();
  });

  it("persists across store instances", () => {
    store.set("persistent.key", "value");
    const store2 = new InstitutionalContextStore(engine, instId);
    expect(store2.get("persistent.key")).toBe("value");
  });
});
