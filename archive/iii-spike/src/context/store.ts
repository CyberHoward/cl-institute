/**
 * Simple key-value context store.
 * Tokens carry payloads through the net — this store holds the accumulated
 * context that transitions can read from via their `context_sources`.
 */
export class ContextStore {
  private data = new Map<string, unknown>();

  get(key: string): unknown {
    return this.data.get(key);
  }

  set(key: string, value: unknown): void {
    this.data.set(key, value);
  }

  has(key: string): boolean {
    return this.data.has(key);
  }

  /** Retrieve multiple keys as a record for passing to agents */
  gather(keys: string[]): Record<string, unknown> {
    const result: Record<string, unknown> = {};
    for (const key of keys) {
      if (this.data.has(key)) {
        result[key] = this.data.get(key);
      }
    }
    return result;
  }

  /** Merge a record into the store */
  merge(record: Record<string, unknown>): void {
    for (const [key, value] of Object.entries(record)) {
      this.data.set(key, value);
    }
  }

  dump(): Record<string, unknown> {
    return Object.fromEntries(this.data);
  }
}
