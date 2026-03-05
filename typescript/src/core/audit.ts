import { randomUUID, createHash } from "node:crypto";
import type { DB } from "./db.js";
import type { AuditAction, AuditEntry, Evidence } from "./types.js";

export interface AppendInput {
  instance_id: string;
  action: AuditAction;
  actor: { actor_id: string; role_id: string; authority_level: number };
  transition_id?: string | undefined;
  marking_before?: Record<string, unknown> | undefined;
  marking_after?: Record<string, unknown> | undefined;
  evidence?: Evidence[] | undefined;
  reasoning?: string | undefined;
}

function computeHash(data: string): string {
  return createHash("sha256").update(data).digest("hex");
}

export class AuditLog {
  constructor(private readonly db: DB) {}

  append(input: AppendInput): AuditEntry {
    const id = randomUUID();
    const timestamp = new Date().toISOString();

    // Get the previous entry for this instance to chain hashes
    const prevEntry = this.db.sqlite
      .prepare(
        `SELECT entry_hash, sequence FROM audit_entries
         WHERE instance_id = ? ORDER BY sequence DESC LIMIT 1`,
      )
      .get(input.instance_id) as { entry_hash: string; sequence: number } | undefined;

    const prevHash = prevEntry?.entry_hash ?? "GENESIS";
    const sequence = (prevEntry?.sequence ?? 0) + 1;

    // Compute hash of this entry
    const hashInput = JSON.stringify({
      id,
      timestamp,
      sequence,
      action: input.action,
      actor: input.actor,
      transition_id: input.transition_id,
      prev_hash: prevHash,
    });
    const entryHash = computeHash(hashInput);

    const entry: AuditEntry = {
      id,
      instance_id: input.instance_id,
      timestamp,
      sequence,
      action: input.action,
      actor: input.actor,
      transition_id: input.transition_id,
      marking_before: input.marking_before,
      marking_after: input.marking_after,
      evidence: input.evidence,
      reasoning: input.reasoning,
      prev_hash: prevHash,
      entry_hash: entryHash,
    };

    this.db.sqlite
      .prepare(
        `INSERT INTO audit_entries (
          id, instance_id, timestamp, sequence, action, actor_json,
          transition_id, marking_before_json, marking_after_json,
          evidence_json, reasoning, prev_hash, entry_hash
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(
        id,
        input.instance_id,
        timestamp,
        sequence,
        input.action,
        JSON.stringify(input.actor),
        input.transition_id ?? null,
        input.marking_before ? JSON.stringify(input.marking_before) : null,
        input.marking_after ? JSON.stringify(input.marking_after) : null,
        input.evidence ? JSON.stringify(input.evidence) : null,
        input.reasoning ?? null,
        prevHash,
        entryHash,
      );

    return entry;
  }

  getEntries(instanceId: string): AuditEntry[] {
    const rows = this.db.sqlite
      .prepare("SELECT * FROM audit_entries WHERE instance_id = ? ORDER BY sequence")
      .all(instanceId) as Array<Record<string, unknown>>;
    return rows.map((row) => this.rowToEntry(row));
  }

  verifyChain(instanceId: string): { valid: boolean; entries_checked: number; errors: string[] } {
    const entries = this.getEntries(instanceId);
    const errors: string[] = [];

    for (let i = 0; i < entries.length; i++) {
      const entry = entries[i]!;

      // Verify prev_hash chain
      if (i === 0) {
        if (entry.prev_hash !== "GENESIS") {
          errors.push(`Entry ${entry.sequence}: expected prev_hash GENESIS, got ${entry.prev_hash}`);
        }
      } else {
        const prev = entries[i - 1]!;
        if (entry.prev_hash !== prev.entry_hash) {
          errors.push(
            `Entry ${entry.sequence}: prev_hash mismatch. Expected ${prev.entry_hash}, got ${entry.prev_hash}`,
          );
        }
      }

      // Verify entry_hash
      const hashInput = JSON.stringify({
        id: entry.id,
        timestamp: entry.timestamp,
        sequence: entry.sequence,
        action: entry.action,
        actor: entry.actor,
        transition_id: entry.transition_id,
        prev_hash: entry.prev_hash,
      });
      const expectedHash = computeHash(hashInput);
      if (entry.entry_hash !== expectedHash) {
        errors.push(`Entry ${entry.sequence}: entry_hash mismatch. Content may have been tampered.`);
      }
    }

    return { valid: errors.length === 0, entries_checked: entries.length, errors };
  }

  private rowToEntry(row: Record<string, unknown>): AuditEntry {
    return {
      id: row["id"] as string,
      instance_id: row["instance_id"] as string,
      timestamp: row["timestamp"] as string,
      sequence: row["sequence"] as number,
      action: row["action"] as AuditAction,
      actor: JSON.parse(row["actor_json"] as string),
      transition_id: (row["transition_id"] as string) ?? undefined,
      marking_before: row["marking_before_json"]
        ? JSON.parse(row["marking_before_json"] as string)
        : undefined,
      marking_after: row["marking_after_json"]
        ? JSON.parse(row["marking_after_json"] as string)
        : undefined,
      evidence: row["evidence_json"]
        ? JSON.parse(row["evidence_json"] as string)
        : undefined,
      reasoning: (row["reasoning"] as string) ?? undefined,
      prev_hash: row["prev_hash"] as string,
      entry_hash: row["entry_hash"] as string,
    };
  }
}
