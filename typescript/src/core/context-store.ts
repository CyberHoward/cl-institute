import type { Engine } from "./engine.js";

/**
 * Key-value store for institutional standing facts.
 * Persisted in SQLite, scoped to an institution.
 * Keys are namespaced strings (e.g., "contacts.secretary.email").
 */
export class InstitutionalContextStore {
  constructor(
    private readonly engine: Engine,
    private readonly institutionId: string,
  ) {}

  /** Get a single value by key. Returns undefined if not found. */
  get(key: string): unknown {
    const row = this.engine["db"].sqlite
      .prepare(
        "SELECT value_json FROM context_entries WHERE institution_id = ? AND key = ?",
      )
      .get(this.institutionId, key) as { value_json: string } | undefined;
    if (!row) return undefined;
    return JSON.parse(row.value_json);
  }

  /** Set a value. Creates or overwrites. */
  set(key: string, value: unknown): void {
    const now = new Date().toISOString();
    this.engine["db"].sqlite
      .prepare(
        `INSERT INTO context_entries (institution_id, key, value_json, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (institution_id, key)
         DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at`,
      )
      .run(this.institutionId, key, JSON.stringify(value), now, now);
  }

  /** Delete a key. */
  delete(key: string): void {
    this.engine["db"].sqlite
      .prepare("DELETE FROM context_entries WHERE institution_id = ? AND key = ?")
      .run(this.institutionId, key);
  }

  /**
   * Resolve multiple keys at once. Returns a record of key → value
   * for all keys that exist. Missing keys are omitted.
   */
  resolve(keys: string[]): Record<string, unknown> {
    const result: Record<string, unknown> = {};
    for (const key of keys) {
      const value = this.get(key);
      if (value !== undefined) {
        result[key] = value;
      }
    }
    return result;
  }
}
