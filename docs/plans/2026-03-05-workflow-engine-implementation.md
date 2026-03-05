# Workflow Engine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the core workflow engine library — CPN with institutional semantics, SQLite persistence, authority enforcement, audit logging — and encode the Carta de Agua process as the first real workflow.

**Architecture:** TypeScript library (`src/core/`) with SQLite storage. The unified CPN model merges Places, Transitions, Tokens, and Markings with institutional concepts (Institution, Role, Actor, Policy). The existing spike code (`src/spike/`) remains untouched — the new core replaces it structurally but doesn't delete it. The existing scaffolding modules (`src/cli-bridge/`, `src/orchestration/`, etc.) will be superseded by the new core; they remain in place for now.

**Tech Stack:** TypeScript (strict, ESM), `better-sqlite3` for persistence, `zod` for schema validation, existing `@mariozechner/pi-agent-core` and `@mariozechner/pi-ai` for agent execution, `vitest` for testing.

---

## Task 1: Project setup — add dependencies and test infrastructure

**Files:**
- Modify: `typescript/package.json`
- Create: `typescript/vitest.config.ts`

**Step 1: Install dependencies**

Run:
```bash
cd typescript && npm install better-sqlite3 && npm install -D @types/better-sqlite3 vitest
```

**Step 2: Create vitest config**

Create `typescript/vitest.config.ts`:

```typescript
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    globals: false,
  },
});
```

**Step 3: Add test script to package.json**

In `typescript/package.json`, replace the test script:

```json
"test": "vitest run",
"test:watch": "vitest"
```

**Step 4: Verify setup**

Run:
```bash
cd typescript && npx vitest run
```

Expected: exits cleanly with "no test files found" or similar.

**Step 5: Commit**

```bash
git add -A && git commit -m "chore: add better-sqlite3, vitest, test infrastructure"
```

---

## Task 2: Core types — the unified CPN + institutional model

**Files:**
- Create: `typescript/src/core/types.ts`
- Create: `typescript/src/core/types.test.ts`

**Step 1: Write the type definitions**

Create `typescript/src/core/types.ts`. These are the canonical types from the design doc — the merged CPN + institutional model.

```typescript
import { z } from "zod";

// ---------------------------------------------------------------------------
// Institution
// ---------------------------------------------------------------------------

export interface Institution {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Role
// ---------------------------------------------------------------------------

export interface Role {
  id: string;
  institution_id: string;
  name: string;
  description?: string;
  authority_level: number;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Actor
// ---------------------------------------------------------------------------

export type ActorType = "human" | "agent";

export interface Actor {
  id: string;
  institution_id: string;
  name: string;
  type: ActorType;
  created_at: string;
  updated_at: string;
}

export interface ActorRoleAssignment {
  actor_id: string;
  role_id: string;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Policy
// ---------------------------------------------------------------------------

export type PolicyStrength = "constraint" | "procedure" | "preference" | "context";

export interface Policy {
  id: string;
  institution_id: string;
  scope: string;
  strength: PolicyStrength;
  text: string;
  metadata?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Net (workflow)
// ---------------------------------------------------------------------------

export interface Net {
  id: string;
  institution_id: string;
  domain?: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Place
// ---------------------------------------------------------------------------

export const JsonSchemaValue = z.record(z.unknown());
export type JsonSchema = z.infer<typeof JsonSchemaValue>;

export interface Place {
  id: string;
  net_id: string;
  description: string;
  schema?: JsonSchema;
}

// ---------------------------------------------------------------------------
// Transition
// ---------------------------------------------------------------------------

export type TransitionMode = "deterministic" | "judgment" | "agentic";

export type DecisionType =
  | "approval"
  | "classification"
  | "prioritization"
  | "allocation"
  | "exception_handling";

export interface Postconditions {
  required: string[];
  desired?: string[];
  escalation?: string[];
}

export type EvidenceType = "artifact" | "reference" | "attestation";

export interface EvidenceRequirement {
  id: string;
  description: string;
  type: EvidenceType;
  required: boolean;
}

export interface Transition {
  id: string;
  net_id: string;

  // CPN core
  consumes: string[];
  produces: string[];
  guard?: string;

  // Institutional semantics
  intent: string;
  mode: TransitionMode;
  decision_type?: DecisionType;
  requires_authority: number;
  authorized_roles?: string[];

  // Data flow
  input_schema?: JsonSchema;
  output_schema?: JsonSchema;
  context_sources: string[];

  // Execution contract
  postconditions: Postconditions;
  evidence_requirements: EvidenceRequirement[];
  available_tools: string[];
  timeout?: number;
}

// ---------------------------------------------------------------------------
// Arc (connects places to transitions)
// ---------------------------------------------------------------------------

export type ArcDirection = "place_to_transition" | "transition_to_place";

export interface Arc {
  id: string;
  net_id: string;
  place_id: string;
  transition_id: string;
  direction: ArcDirection;
}

// ---------------------------------------------------------------------------
// Token and Marking (runtime)
// ---------------------------------------------------------------------------

export interface Token {
  id: string;
  instance_id: string;
  place_id: string;
  payload: Record<string, unknown>;
  created_at: string;
}

// ---------------------------------------------------------------------------
// Workflow Instance
// ---------------------------------------------------------------------------

export type InstanceStatus = "running" | "completed" | "stuck" | "suspended";

export interface WorkflowInstance {
  id: string;
  net_id: string;
  status: InstanceStatus;
  created_at: string;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

export type AuditAction =
  | "instance_created"
  | "transition_fired"
  | "judgment_pending"
  | "judgment_resolved"
  | "postcondition_failed"
  | "escalation_triggered"
  | "policy_consulted"
  | "override_applied";

export interface Evidence {
  requirement_id: string;
  type: EvidenceType;
  content: unknown;
  captured_at: string;
}

export interface AuditEntry {
  id: string;
  instance_id: string;
  timestamp: string;
  sequence: number;
  action: AuditAction;
  actor: { actor_id: string; role_id: string; authority_level: number };
  transition_id?: string;
  marking_before?: Record<string, unknown>;
  marking_after?: Record<string, unknown>;
  evidence?: Evidence[];
  reasoning?: string;
  prev_hash: string;
  entry_hash: string;
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

export type Severity = "error" | "warning";

export interface ConstraintViolation {
  constraint_name: string;
  severity: Severity;
  message: string;
  location: string;
  suggestion?: string;
}

export interface ValidationResult {
  violations: ConstraintViolation[];
  is_valid: boolean;
}

// ---------------------------------------------------------------------------
// Firing result (returned by fireTransition)
// ---------------------------------------------------------------------------

export interface FiringResult {
  success: boolean;
  transition_id: string;
  instance_id: string;
  tokens_consumed: Token[];
  tokens_produced: Token[];
  postcondition_results: Record<string, boolean>;
  evidence: Evidence[];
  audit_entry_id: string;
  error?: string;
}

// ---------------------------------------------------------------------------
// Pending judgment (returned by getPendingJudgments)
// ---------------------------------------------------------------------------

export interface PendingJudgment {
  instance_id: string;
  transition_id: string;
  transition_intent: string;
  transition_mode: "judgment";
  requires_authority: number;
  token_payloads: Record<string, unknown>[];
  policies: Policy[];
}
```

**Step 2: Write a basic type test**

Create `typescript/src/core/types.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import type {
  Institution,
  Role,
  Transition,
  Place,
  Token,
  AuditEntry,
} from "./types.js";

describe("core types", () => {
  it("can construct an Institution", () => {
    const inst: Institution = {
      id: "asada-1",
      name: "ASADA Playas de Nosara",
      description: "Community water association",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    expect(inst.name).toBe("ASADA Playas de Nosara");
  });

  it("can construct a Transition with full institutional metadata", () => {
    const t: Transition = {
      id: "board-decision",
      net_id: "carta-de-agua",
      consumes: ["board-ready"],
      produces: ["decided"],
      intent: "Board reviews case and issues decision",
      mode: "judgment",
      decision_type: "approval",
      requires_authority: 4,
      context_sources: ["case-data", "technical-report"],
      postconditions: {
        required: ["decision-made", "rationale-documented"],
        desired: ["conditions-specified-if-applicable"],
        escalation: ["escalate-to-aya"],
      },
      evidence_requirements: [
        {
          id: "board-resolution",
          description: "Board resolution number",
          type: "reference",
          required: true,
        },
      ],
      available_tools: [],
      input_schema: { type: "object", properties: { caseId: { type: "string" } } },
      output_schema: {
        type: "object",
        properties: {
          decision: { type: "string", enum: ["approve", "deny", "conditional", "defer"] },
          rationale: { type: "string" },
        },
      },
    };
    expect(t.mode).toBe("judgment");
    expect(t.requires_authority).toBe(4);
    expect(t.evidence_requirements).toHaveLength(1);
  });

  it("can construct a Token with unstructured payload", () => {
    const token: Token = {
      id: "tok-1",
      instance_id: "inst-1",
      place_id: "intake",
      payload: {
        applicant: "Juan Pérez",
        cadastral_plan: "SJ-12345",
        channel: "whatsapp",
      },
      created_at: new Date().toISOString(),
    };
    expect(token.payload["applicant"]).toBe("Juan Pérez");
  });
});
```

**Step 3: Run test**

Run: `cd typescript && npx vitest run src/core/types.test.ts`

Expected: PASS (3 tests)

**Step 4: Commit**

```bash
git add -A && git commit -m "feat: core types — unified CPN + institutional model"
```

---

## Task 3: SQLite schema and database module

**Files:**
- Create: `typescript/src/core/db.ts`
- Create: `typescript/src/core/db.test.ts`

**Step 1: Write the failing test**

Create `typescript/src/core/db.test.ts`:

```typescript
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
```

**Step 2: Run test to verify it fails**

Run: `cd typescript && npx vitest run src/core/db.test.ts`

Expected: FAIL — `db.js` does not exist.

**Step 3: Implement DB**

Create `typescript/src/core/db.ts`:

```typescript
import Database from "better-sqlite3";

const SCHEMA = `
  CREATE TABLE IF NOT EXISTS institutions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    name TEXT NOT NULL,
    description TEXT,
    authority_level INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(institution_id, name)
  );

  CREATE TABLE IF NOT EXISTS actors (
    id TEXT PRIMARY KEY,
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('human', 'agent')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS actor_roles (
    actor_id TEXT NOT NULL REFERENCES actors(id),
    role_id TEXT NOT NULL REFERENCES roles(id),
    created_at TEXT NOT NULL,
    PRIMARY KEY (actor_id, role_id)
  );

  CREATE TABLE IF NOT EXISTS nets (
    id TEXT PRIMARY KEY,
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    domain TEXT,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS places (
    id TEXT PRIMARY KEY,
    net_id TEXT NOT NULL REFERENCES nets(id),
    description TEXT NOT NULL,
    schema_json TEXT
  );

  CREATE TABLE IF NOT EXISTS transitions (
    id TEXT PRIMARY KEY,
    net_id TEXT NOT NULL REFERENCES nets(id),

    -- CPN core
    guard TEXT,

    -- Institutional semantics
    intent TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('deterministic', 'judgment', 'agentic')),
    decision_type TEXT,
    requires_authority INTEGER NOT NULL DEFAULT 0,
    authorized_roles_json TEXT,

    -- Data flow
    input_schema_json TEXT,
    output_schema_json TEXT,
    context_sources_json TEXT NOT NULL DEFAULT '[]',

    -- Execution contract
    postconditions_json TEXT NOT NULL,
    evidence_requirements_json TEXT NOT NULL DEFAULT '[]',
    available_tools_json TEXT NOT NULL DEFAULT '[]',
    timeout INTEGER
  );

  CREATE TABLE IF NOT EXISTS arcs (
    id TEXT PRIMARY KEY,
    net_id TEXT NOT NULL REFERENCES nets(id),
    place_id TEXT NOT NULL REFERENCES places(id),
    transition_id TEXT NOT NULL REFERENCES transitions(id),
    direction TEXT NOT NULL CHECK(direction IN ('place_to_transition', 'transition_to_place'))
  );

  CREATE TABLE IF NOT EXISTS policies (
    id TEXT PRIMARY KEY,
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    scope TEXT NOT NULL,
    strength TEXT NOT NULL CHECK(strength IN ('constraint', 'procedure', 'preference', 'context')),
    text TEXT NOT NULL,
    metadata_json TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS instances (
    id TEXT PRIMARY KEY,
    net_id TEXT NOT NULL REFERENCES nets(id),
    status TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running', 'completed', 'stuck', 'suspended')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS tokens (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES instances(id),
    place_id TEXT NOT NULL REFERENCES places(id),
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS audit_entries (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES instances(id),
    timestamp TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    action TEXT NOT NULL,
    actor_json TEXT NOT NULL,
    transition_id TEXT,
    marking_before_json TEXT,
    marking_after_json TEXT,
    evidence_json TEXT,
    reasoning TEXT,
    prev_hash TEXT NOT NULL,
    entry_hash TEXT NOT NULL
  );

  CREATE INDEX IF NOT EXISTS idx_tokens_instance ON tokens(instance_id);
  CREATE INDEX IF NOT EXISTS idx_tokens_place ON tokens(place_id);
  CREATE INDEX IF NOT EXISTS idx_audit_instance ON audit_entries(instance_id);
  CREATE INDEX IF NOT EXISTS idx_audit_sequence ON audit_entries(instance_id, sequence);
  CREATE INDEX IF NOT EXISTS idx_policies_scope ON policies(scope);
  CREATE INDEX IF NOT EXISTS idx_arcs_transition ON arcs(transition_id);
  CREATE INDEX IF NOT EXISTS idx_arcs_place ON arcs(place_id);
`;

export class DB {
  readonly sqlite: Database.Database;

  constructor(path: string) {
    this.sqlite = new Database(path);
    this.sqlite.pragma("journal_mode = WAL");
    this.sqlite.pragma("foreign_keys = ON");
    this.sqlite.exec(SCHEMA);
  }

  listTables(): string[] {
    const rows = this.sqlite
      .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
      .all() as Array<{ name: string }>;
    return rows.map((r) => r.name);
  }

  close(): void {
    this.sqlite.close();
  }
}
```

**Step 4: Run test to verify it passes**

Run: `cd typescript && npx vitest run src/core/db.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: SQLite schema and DB module"
```

---

## Task 4: Definition operations — createInstitution, createRole, createActor, assignRole

**Files:**
- Create: `typescript/src/core/engine.ts`
- Create: `typescript/src/core/engine.test.ts`

This is the main API module. All operations live here as methods on an `Engine` class that wraps the DB.

**Step 1: Write failing tests**

Create `typescript/src/core/engine.test.ts`:

```typescript
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
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: FAIL — `engine.js` does not exist.

**Step 3: Implement Engine with definition operations**

Create `typescript/src/core/engine.ts`:

```typescript
import { randomUUID } from "node:crypto";
import { DB } from "./db.js";
import type {
  Institution,
  Role,
  Actor,
  ActorType,
  ActorRoleAssignment,
} from "./types.js";

export class Engine {
  private readonly db: DB;

  constructor(dbPath: string) {
    this.db = new DB(dbPath);
  }

  // -----------------------------------------------------------------------
  // Institution
  // -----------------------------------------------------------------------

  createInstitution(name: string, description?: string): Institution {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO institutions (id, name, description, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)`,
      )
      .run(id, name, description ?? null, now, now);
    return { id, name, description, created_at: now, updated_at: now };
  }

  getInstitution(id: string): Institution {
    const row = this.db.sqlite
      .prepare("SELECT * FROM institutions WHERE id = ?")
      .get(id) as Record<string, unknown> | undefined;
    if (!row) throw new Error(`Institution not found: ${id}`);
    return {
      id: row["id"] as string,
      name: row["name"] as string,
      description: (row["description"] as string) ?? undefined,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    };
  }

  // -----------------------------------------------------------------------
  // Roles
  // -----------------------------------------------------------------------

  createRole(
    institutionId: string,
    name: string,
    authorityLevel: number,
    description?: string,
  ): Role {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO roles (id, institution_id, name, description, authority_level, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(id, institutionId, name, description ?? null, authorityLevel, now, now);
    return {
      id,
      institution_id: institutionId,
      name,
      description,
      authority_level: authorityLevel,
      created_at: now,
      updated_at: now,
    };
  }

  // -----------------------------------------------------------------------
  // Actors
  // -----------------------------------------------------------------------

  createActor(institutionId: string, name: string, type: ActorType): Actor {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO actors (id, institution_id, name, type, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)`,
      )
      .run(id, institutionId, name, type, now, now);
    return {
      id,
      institution_id: institutionId,
      name,
      type,
      created_at: now,
      updated_at: now,
    };
  }

  assignRole(actorId: string, roleId: string): void {
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO actor_roles (actor_id, role_id, created_at)
         VALUES (?, ?, ?)`,
      )
      .run(actorId, roleId, now);
  }

  getActorRoles(actorId: string): Role[] {
    const rows = this.db.sqlite
      .prepare(
        `SELECT r.* FROM roles r
         JOIN actor_roles ar ON ar.role_id = r.id
         WHERE ar.actor_id = ?`,
      )
      .all(actorId) as Array<Record<string, unknown>>;
    return rows.map((row) => ({
      id: row["id"] as string,
      institution_id: row["institution_id"] as string,
      name: row["name"] as string,
      description: (row["description"] as string) ?? undefined,
      authority_level: row["authority_level"] as number,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    }));
  }

  /**
   * Get the maximum authority level an actor has across all assigned roles.
   */
  getActorAuthority(actorId: string): number {
    const roles = this.getActorRoles(actorId);
    if (roles.length === 0) return 0;
    return Math.max(...roles.map((r) => r.authority_level));
  }

  // -----------------------------------------------------------------------
  // Lifecycle
  // -----------------------------------------------------------------------

  close(): void {
    this.db.close();
  }
}
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: PASS (5 tests)

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: Engine with institution, role, actor definition operations"
```

---

## Task 5: Net definition — createNet, addPlace, addTransition

**Files:**
- Modify: `typescript/src/core/engine.ts`
- Modify: `typescript/src/core/engine.test.ts`

**Step 1: Write failing tests**

Append to `typescript/src/core/engine.test.ts`:

```typescript
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
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: FAIL — `createNet`, `addPlace`, `addTransition` not defined.

**Step 3: Implement net definition operations**

Add to `typescript/src/core/engine.ts`, in the `Engine` class:

```typescript
import type {
  Institution,
  Role,
  Actor,
  ActorType,
  Net,
  Place,
  Transition,
  TransitionMode,
  DecisionType,
  Postconditions,
  EvidenceRequirement,
  Arc,
  ArcDirection,
  JsonSchema,
} from "./types.js";

// Add this type for addTransition input
export interface TransitionDef {
  id: string;
  consumes: string[];
  produces: string[];
  guard?: string;
  intent: string;
  mode: TransitionMode;
  decision_type?: DecisionType;
  requires_authority: number;
  authorized_roles?: string[];
  input_schema?: JsonSchema;
  output_schema?: JsonSchema;
  context_sources: string[];
  postconditions: Postconditions;
  evidence_requirements: EvidenceRequirement[];
  available_tools: string[];
  timeout?: number;
}

// --- Net operations (inside Engine class) ---

  createNet(institutionId: string, name: string, domain?: string, description?: string): Net {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO nets (id, institution_id, domain, name, description, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(id, institutionId, domain ?? null, name, description ?? null, now, now);
    return { id, institution_id: institutionId, domain, name, description, created_at: now, updated_at: now };
  }

  addPlace(netId: string, id: string, description: string, schema?: JsonSchema): Place {
    this.db.sqlite
      .prepare(
        `INSERT INTO places (id, net_id, description, schema_json)
         VALUES (?, ?, ?, ?)`,
      )
      .run(id, netId, description, schema ? JSON.stringify(schema) : null);
    return { id, net_id: netId, description, schema };
  }

  addTransition(netId: string, def: TransitionDef): Transition {
    // Validate that all consumed and produced places exist
    const allPlaces = def.consumes.concat(def.produces);
    for (const placeId of allPlaces) {
      const exists = this.db.sqlite
        .prepare("SELECT 1 FROM places WHERE id = ? AND net_id = ?")
        .get(placeId, netId);
      if (!exists) {
        throw new Error(`Place '${placeId}' does not exist in net '${netId}'`);
      }
    }

    this.db.sqlite
      .prepare(
        `INSERT INTO transitions (
          id, net_id, guard, intent, mode, decision_type, requires_authority,
          authorized_roles_json, input_schema_json, output_schema_json,
          context_sources_json, postconditions_json, evidence_requirements_json,
          available_tools_json, timeout
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(
        def.id,
        netId,
        def.guard ?? null,
        def.intent,
        def.mode,
        def.decision_type ?? null,
        def.requires_authority,
        def.authorized_roles ? JSON.stringify(def.authorized_roles) : null,
        def.input_schema ? JSON.stringify(def.input_schema) : null,
        def.output_schema ? JSON.stringify(def.output_schema) : null,
        JSON.stringify(def.context_sources),
        JSON.stringify(def.postconditions),
        JSON.stringify(def.evidence_requirements),
        JSON.stringify(def.available_tools),
        def.timeout ?? null,
      );

    // Create arcs for consumes (place → transition)
    for (const placeId of def.consumes) {
      this.db.sqlite
        .prepare(
          `INSERT INTO arcs (id, net_id, place_id, transition_id, direction)
           VALUES (?, ?, ?, ?, ?)`,
        )
        .run(randomUUID(), netId, placeId, def.id, "place_to_transition");
    }

    // Create arcs for produces (transition → place)
    for (const placeId of def.produces) {
      this.db.sqlite
        .prepare(
          `INSERT INTO arcs (id, net_id, place_id, transition_id, direction)
           VALUES (?, ?, ?, ?, ?)`,
        )
        .run(randomUUID(), netId, placeId, def.id, "transition_to_place");
    }

    return {
      id: def.id,
      net_id: netId,
      consumes: def.consumes,
      produces: def.produces,
      guard: def.guard,
      intent: def.intent,
      mode: def.mode,
      decision_type: def.decision_type,
      requires_authority: def.requires_authority,
      authorized_roles: def.authorized_roles,
      input_schema: def.input_schema,
      output_schema: def.output_schema,
      context_sources: def.context_sources,
      postconditions: def.postconditions,
      evidence_requirements: def.evidence_requirements,
      available_tools: def.available_tools,
      timeout: def.timeout,
    };
  }

  getArcs(netId: string): Arc[] {
    const rows = this.db.sqlite
      .prepare("SELECT * FROM arcs WHERE net_id = ?")
      .all(netId) as Array<Record<string, unknown>>;
    return rows.map((row) => ({
      id: row["id"] as string,
      net_id: row["net_id"] as string,
      place_id: row["place_id"] as string,
      transition_id: row["transition_id"] as string,
      direction: row["direction"] as ArcDirection,
    }));
  }

  getNetWithGraph(netId: string): { net: Net; places: Place[]; transitions: Transition[] } {
    const netRow = this.db.sqlite
      .prepare("SELECT * FROM nets WHERE id = ?")
      .get(netId) as Record<string, unknown> | undefined;
    if (!netRow) throw new Error(`Net not found: ${netId}`);

    const net: Net = {
      id: netRow["id"] as string,
      institution_id: netRow["institution_id"] as string,
      domain: (netRow["domain"] as string) ?? undefined,
      name: netRow["name"] as string,
      description: (netRow["description"] as string) ?? undefined,
      created_at: netRow["created_at"] as string,
      updated_at: netRow["updated_at"] as string,
    };

    const placeRows = this.db.sqlite
      .prepare("SELECT * FROM places WHERE net_id = ?")
      .all(netId) as Array<Record<string, unknown>>;
    const places: Place[] = placeRows.map((row) => ({
      id: row["id"] as string,
      net_id: row["net_id"] as string,
      description: row["description"] as string,
      schema: row["schema_json"] ? JSON.parse(row["schema_json"] as string) : undefined,
    }));

    const transitionRows = this.db.sqlite
      .prepare("SELECT * FROM transitions WHERE net_id = ?")
      .all(netId) as Array<Record<string, unknown>>;

    const arcs = this.getArcs(netId);
    const transitions: Transition[] = transitionRows.map((row) => {
      const tId = row["id"] as string;
      const consumeArcs = arcs.filter(
        (a) => a.transition_id === tId && a.direction === "place_to_transition",
      );
      const produceArcs = arcs.filter(
        (a) => a.transition_id === tId && a.direction === "transition_to_place",
      );
      return {
        id: tId,
        net_id: row["net_id"] as string,
        consumes: consumeArcs.map((a) => a.place_id),
        produces: produceArcs.map((a) => a.place_id),
        guard: (row["guard"] as string) ?? undefined,
        intent: row["intent"] as string,
        mode: row["mode"] as TransitionMode,
        decision_type: (row["decision_type"] as DecisionType) ?? undefined,
        requires_authority: row["requires_authority"] as number,
        authorized_roles: row["authorized_roles_json"]
          ? JSON.parse(row["authorized_roles_json"] as string)
          : undefined,
        input_schema: row["input_schema_json"]
          ? JSON.parse(row["input_schema_json"] as string)
          : undefined,
        output_schema: row["output_schema_json"]
          ? JSON.parse(row["output_schema_json"] as string)
          : undefined,
        context_sources: JSON.parse(row["context_sources_json"] as string),
        postconditions: JSON.parse(row["postconditions_json"] as string),
        evidence_requirements: JSON.parse(row["evidence_requirements_json"] as string),
        available_tools: JSON.parse(row["available_tools_json"] as string),
        timeout: (row["timeout"] as number) ?? undefined,
      };
    });

    return { net, places, transitions };
  }
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: PASS (all tests including new net definition tests)

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: net definition — createNet, addPlace, addTransition"
```

---

## Task 6: Policy operations — attachPolicy, getPolicies

**Files:**
- Modify: `typescript/src/core/engine.ts`
- Modify: `typescript/src/core/engine.test.ts`

**Step 1: Write failing tests**

Append to `typescript/src/core/engine.test.ts`:

```typescript
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
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: FAIL — `attachPolicy`, `getPolicies` not defined.

**Step 3: Implement policy operations**

Add to the `Engine` class in `typescript/src/core/engine.ts`:

```typescript
  attachPolicy(
    institutionId: string,
    scope: string,
    strength: PolicyStrength,
    text: string,
    metadata?: Record<string, unknown>,
  ): Policy {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO policies (id, institution_id, scope, strength, text, metadata_json, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(id, institutionId, scope, strength, text, metadata ? JSON.stringify(metadata) : null, now, now);
    return { id, institution_id: institutionId, scope, strength, text, metadata, created_at: now, updated_at: now };
  }

  /**
   * Get policies matching a scope, ordered by strength (constraint first)
   * then by specificity (exact match first, then parent, then global).
   */
  getPolicies(scope: string): Policy[] {
    // Build list of matching scopes: exact, parent wildcards, global
    const matchScopes: string[] = [scope];
    const parts = scope.split(".");
    for (let i = parts.length - 1; i >= 1; i--) {
      matchScopes.push(parts.slice(0, i).join(".") + ".*");
    }
    matchScopes.push("*");

    const placeholders = matchScopes.map(() => "?").join(", ");
    const rows = this.db.sqlite
      .prepare(`SELECT * FROM policies WHERE scope IN (${placeholders})`)
      .all(...matchScopes) as Array<Record<string, unknown>>;

    const strengthOrder: Record<string, number> = {
      constraint: 0,
      procedure: 1,
      preference: 2,
      context: 3,
    };

    const scopeSpecificity = (s: string): number => {
      const idx = matchScopes.indexOf(s);
      return idx === -1 ? matchScopes.length : idx;
    };

    const policies: Policy[] = rows.map((row) => ({
      id: row["id"] as string,
      institution_id: row["institution_id"] as string,
      scope: row["scope"] as string,
      strength: row["strength"] as PolicyStrength,
      text: row["text"] as string,
      metadata: row["metadata_json"] ? JSON.parse(row["metadata_json"] as string) : undefined,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    }));

    policies.sort((a, b) => {
      const strengthDiff = (strengthOrder[a.strength] ?? 99) - (strengthOrder[b.strength] ?? 99);
      if (strengthDiff !== 0) return strengthDiff;
      return scopeSpecificity(a.scope) - scopeSpecificity(b.scope);
    });

    return policies;
  }
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: policy operations — attachPolicy, getPolicies with scope resolution"
```

---

## Task 7: Runtime operations — instantiate, getMarking, getEnabledTransitions

**Files:**
- Modify: `typescript/src/core/engine.ts`
- Modify: `typescript/src/core/engine.test.ts`

**Step 1: Write failing tests**

Append to `typescript/src/core/engine.test.ts`:

```typescript
describe("Engine — runtime", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA").id;
    const net = engine.createNet(instId, "Carta de Agua", "carta-de-agua");
    netId = net.id;

    // Build a simple 3-place, 2-transition net
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

    // Create role and actor
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

      // Check that triage is NOT enabled (no token in docs-complete)
      const enabled = engine.getEnabledTransitions(instance.id, actorId);
      expect(enabled.every((t) => t.id !== "triage")).toBe(true);
    });
  });
});
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: FAIL — `instantiate`, `getMarking`, `getEnabledTransitions`, `listActors` not defined.

**Step 3: Implement runtime operations**

Add to the `Engine` class in `typescript/src/core/engine.ts`:

```typescript
  listActors(institutionId: string): Actor[] {
    const rows = this.db.sqlite
      .prepare("SELECT * FROM actors WHERE institution_id = ?")
      .all(institutionId) as Array<Record<string, unknown>>;
    return rows.map((row) => ({
      id: row["id"] as string,
      institution_id: row["institution_id"] as string,
      name: row["name"] as string,
      type: row["type"] as ActorType,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    }));
  }

  instantiate(
    netId: string,
    startPlaceId: string,
    initialPayload: Record<string, unknown>,
  ): WorkflowInstance {
    const instanceId = randomUUID();
    const now = new Date().toISOString();

    this.db.sqlite
      .prepare(
        `INSERT INTO instances (id, net_id, status, created_at, updated_at)
         VALUES (?, ?, 'running', ?, ?)`,
      )
      .run(instanceId, netId, now, now);

    // Create the initial token
    const tokenId = randomUUID();
    this.db.sqlite
      .prepare(
        `INSERT INTO tokens (id, instance_id, place_id, payload_json, created_at)
         VALUES (?, ?, ?, ?, ?)`,
      )
      .run(tokenId, instanceId, startPlaceId, JSON.stringify(initialPayload), now);

    return { id: instanceId, net_id: netId, status: "running", created_at: now, updated_at: now };
  }

  getMarking(instanceId: string): Map<string, Token[]> {
    const rows = this.db.sqlite
      .prepare("SELECT * FROM tokens WHERE instance_id = ?")
      .all(instanceId) as Array<Record<string, unknown>>;

    const marking = new Map<string, Token[]>();
    for (const row of rows) {
      const token: Token = {
        id: row["id"] as string,
        instance_id: row["instance_id"] as string,
        place_id: row["place_id"] as string,
        payload: JSON.parse(row["payload_json"] as string),
        created_at: row["created_at"] as string,
      };
      const existing = marking.get(token.place_id) ?? [];
      existing.push(token);
      marking.set(token.place_id, existing);
    }
    return marking;
  }

  /**
   * Find transitions that are enabled for a given actor in a given instance.
   * A transition is enabled when:
   * 1. All input places (consumes) have at least one token
   * 2. The actor has sufficient authority
   */
  getEnabledTransitions(instanceId: string, actorId: string): Transition[] {
    // Get the instance to find the net
    const instance = this.db.sqlite
      .prepare("SELECT * FROM instances WHERE id = ?")
      .get(instanceId) as Record<string, unknown> | undefined;
    if (!instance) throw new Error(`Instance not found: ${instanceId}`);

    const netId = instance["net_id"] as string;
    const { transitions } = this.getNetWithGraph(netId);
    const marking = this.getMarking(instanceId);
    const actorAuthority = this.getActorAuthority(actorId);

    return transitions.filter((t) => {
      // Check authority
      if (actorAuthority < t.requires_authority) return false;

      // Check all input places have tokens
      return t.consumes.every((placeId) => {
        const tokens = marking.get(placeId);
        return tokens != null && tokens.length > 0;
      });
    });
  }
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: runtime operations — instantiate, getMarking, getEnabledTransitions"
```

---

## Task 8: Audit log with hash chaining

**Files:**
- Create: `typescript/src/core/audit.ts`
- Create: `typescript/src/core/audit.test.ts`

**Step 1: Write failing test**

Create `typescript/src/core/audit.test.ts`:

```typescript
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { DB } from "./db.js";
import { AuditLog } from "./audit.js";

describe("AuditLog", () => {
  let db: DB;
  let audit: AuditLog;

  beforeEach(() => {
    db = new DB(":memory:");
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
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/audit.test.ts`

Expected: FAIL — `audit.js` does not exist.

**Step 3: Implement AuditLog**

Create `typescript/src/core/audit.ts`:

```typescript
import { randomUUID, createHash } from "node:crypto";
import type { DB } from "./db.js";
import type { AuditAction, AuditEntry, Evidence } from "./types.js";

interface AppendInput {
  instance_id: string;
  action: AuditAction;
  actor: { actor_id: string; role_id: string; authority_level: number };
  transition_id?: string;
  marking_before?: Record<string, unknown>;
  marking_after?: Record<string, unknown>;
  evidence?: Evidence[];
  reasoning?: string;
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
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/core/audit.test.ts`

Expected: PASS (4 tests)

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: audit log with cryptographic hash chaining"
```

---

## Task 9: fireTransition — the core runtime operation

This is the biggest task. `fireTransition` checks authority, consumes tokens, produces tokens, writes audit entries. For now it does NOT call the agent — that's Task 11. This task builds the mechanical firing logic.

**Files:**
- Modify: `typescript/src/core/engine.ts`
- Modify: `typescript/src/core/engine.test.ts`

**Step 1: Write failing tests**

Append to `typescript/src/core/engine.test.ts`:

```typescript
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

    // Token moved from intake to docs-complete
    const marking = engine.getMarking(instance.id);
    expect(marking.has("intake")).toBe(false);
    expect(marking.get("docs-complete")).toHaveLength(1);
    expect(marking.get("docs-complete")![0]!.payload["docs_verified"]).toBe(true);
  });

  it("writes an audit entry on fire", () => {
    const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });
    engine.fireTransition(instance.id, "check-completeness", actorId, {});
    const history = engine.getHistory(instance.id);
    // instance_created + transition_fired
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
    // Fire once to move token
    engine.fireTransition(instance.id, "check-completeness", actorId, {});
    // Try to fire again — intake is empty now
    const result = engine.fireTransition(instance.id, "check-completeness", actorId, {});
    expect(result.success).toBe(false);
    expect(result.error).toMatch(/token/i);
  });
});
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: FAIL — `fireTransition`, `getHistory` not defined.

**Step 3: Implement fireTransition**

Add to `typescript/src/core/engine.ts`. The Engine needs to own an AuditLog instance:

```typescript
import { AuditLog } from "./audit.js";

// In Engine constructor, add:
  private readonly audit: AuditLog;

  constructor(dbPath: string) {
    this.db = new DB(dbPath);
    this.audit = new AuditLog(this.db);
  }

// In instantiate method, add audit entry after creating token:
    this.audit.append({
      instance_id: instanceId,
      action: "instance_created",
      actor: { actor_id: "system", role_id: "system", authority_level: 0 },
      marking_after: { [startPlaceId]: [initialPayload] },
    });

// New methods:

  fireTransition(
    instanceId: string,
    transitionId: string,
    actorId: string,
    outputPayload: Record<string, unknown>,
    evidence?: Evidence[],
    reasoning?: string,
  ): FiringResult {
    const instance = this.db.sqlite
      .prepare("SELECT * FROM instances WHERE id = ?")
      .get(instanceId) as Record<string, unknown> | undefined;
    if (!instance) throw new Error(`Instance not found: ${instanceId}`);

    const netId = instance["net_id"] as string;
    const { transitions } = this.getNetWithGraph(netId);
    const transition = transitions.find((t) => t.id === transitionId);
    if (!transition) throw new Error(`Transition not found: ${transitionId}`);

    // Check authority
    const actorAuthority = this.getActorAuthority(actorId);
    const actorRoles = this.getActorRoles(actorId);
    const actingRole = actorRoles.reduce(
      (best, r) => (r.authority_level > (best?.authority_level ?? -1) ? r : best),
      actorRoles[0],
    );

    if (actorAuthority < transition.requires_authority) {
      return {
        success: false,
        transition_id: transitionId,
        instance_id: instanceId,
        tokens_consumed: [],
        tokens_produced: [],
        postcondition_results: {},
        evidence: [],
        audit_entry_id: "",
        error: `Insufficient authority: actor has ${actorAuthority}, transition requires ${transition.requires_authority}`,
      };
    }

    // Check tokens in input places
    const marking = this.getMarking(instanceId);
    for (const placeId of transition.consumes) {
      const tokens = marking.get(placeId);
      if (!tokens || tokens.length === 0) {
        return {
          success: false,
          transition_id: transitionId,
          instance_id: instanceId,
          tokens_consumed: [],
          tokens_produced: [],
          postcondition_results: {},
          evidence: [],
          audit_entry_id: "",
          error: `No token in input place '${placeId}'`,
        };
      }
    }

    // Snapshot marking before
    const markingBefore: Record<string, unknown> = {};
    for (const [placeId, tokens] of marking) {
      markingBefore[placeId] = tokens.map((t) => t.payload);
    }

    // Consume tokens (one per input place)
    const consumedTokens: Token[] = [];
    for (const placeId of transition.consumes) {
      const tokens = marking.get(placeId)!;
      const token = tokens[0]!;
      this.db.sqlite.prepare("DELETE FROM tokens WHERE id = ?").run(token.id);
      consumedTokens.push(token);
    }

    // Produce tokens (one per output place)
    const now = new Date().toISOString();
    const producedTokens: Token[] = [];
    for (const placeId of transition.produces) {
      const tokenId = randomUUID();
      this.db.sqlite
        .prepare(
          `INSERT INTO tokens (id, instance_id, place_id, payload_json, created_at)
           VALUES (?, ?, ?, ?, ?)`,
        )
        .run(tokenId, instanceId, placeId, JSON.stringify(outputPayload), now);
      producedTokens.push({
        id: tokenId,
        instance_id: instanceId,
        place_id: placeId,
        payload: outputPayload,
        created_at: now,
      });
    }

    // Snapshot marking after
    const markingAfterMap = this.getMarking(instanceId);
    const markingAfter: Record<string, unknown> = {};
    for (const [placeId, tokens] of markingAfterMap) {
      markingAfter[placeId] = tokens.map((t) => t.payload);
    }

    // Write audit entry
    const auditEntry = this.audit.append({
      instance_id: instanceId,
      action: "transition_fired",
      actor: {
        actor_id: actorId,
        role_id: actingRole?.id ?? "unknown",
        authority_level: actorAuthority,
      },
      transition_id: transitionId,
      marking_before: markingBefore,
      marking_after: markingAfter,
      evidence: evidence,
      reasoning: reasoning,
    });

    return {
      success: true,
      transition_id: transitionId,
      instance_id: instanceId,
      tokens_consumed: consumedTokens,
      tokens_produced: producedTokens,
      postcondition_results: {},
      evidence: evidence ?? [],
      audit_entry_id: auditEntry.id,
    };
  }

  getHistory(instanceId: string): AuditEntry[] {
    return this.audit.getEntries(instanceId);
  }
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: fireTransition with authority checks, token flow, audit logging"
```

---

## Task 10: Judgment points — getPendingJudgments, resolveJudgment

**Files:**
- Modify: `typescript/src/core/engine.ts`
- Modify: `typescript/src/core/engine.test.ts`

**Step 1: Write failing tests**

Append to `typescript/src/core/engine.test.ts`:

```typescript
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

    // Admin (authority 2) — can't fire board decisions
    const adminRole = engine.createRole(instId, "administrator", 2);
    const admin = engine.createActor(instId, "Don Carlos", "human");
    engine.assignRole(admin.id, adminRole.id);
    adminId = admin.id;

    // Board (authority 4) — can fire board decisions
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
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: FAIL — `getPendingJudgments`, `resolveJudgment` not defined.

**Step 3: Implement judgment operations**

Add to the `Engine` class in `typescript/src/core/engine.ts`:

```typescript
  getPendingJudgments(instanceId: string): PendingJudgment[] {
    const instance = this.db.sqlite
      .prepare("SELECT * FROM instances WHERE id = ?")
      .get(instanceId) as Record<string, unknown> | undefined;
    if (!instance) throw new Error(`Instance not found: ${instanceId}`);

    const netId = instance["net_id"] as string;
    const { transitions } = this.getNetWithGraph(netId);
    const marking = this.getMarking(instanceId);
    const net = this.db.sqlite
      .prepare("SELECT * FROM nets WHERE id = ?")
      .get(netId) as Record<string, unknown>;
    const institutionId = net["institution_id"] as string;

    const pending: PendingJudgment[] = [];

    for (const t of transitions) {
      if (t.mode !== "judgment") continue;

      // Check all input places have tokens
      const allInputsHaveTokens = t.consumes.every((placeId) => {
        const tokens = marking.get(placeId);
        return tokens != null && tokens.length > 0;
      });
      if (!allInputsHaveTokens) continue;

      // Gather token payloads from input places
      const tokenPayloads: Record<string, unknown>[] = [];
      for (const placeId of t.consumes) {
        const tokens = marking.get(placeId)!;
        for (const token of tokens) {
          tokenPayloads.push(token.payload);
        }
      }

      // Resolve policies for this transition's scope
      const domain = (net["domain"] as string) ?? "";
      const scope = domain ? `${domain}.${t.id}` : t.id;
      const policies = this.getPolicies(scope);

      pending.push({
        instance_id: instanceId,
        transition_id: t.id,
        transition_intent: t.intent,
        transition_mode: "judgment",
        requires_authority: t.requires_authority,
        token_payloads: tokenPayloads,
        policies,
      });
    }

    return pending;
  }

  resolveJudgment(
    instanceId: string,
    transitionId: string,
    actorId: string,
    decision: Record<string, unknown>,
    reasoning?: string,
    evidence?: Evidence[],
  ): FiringResult {
    // Validate this is actually a judgment transition
    const instance = this.db.sqlite
      .prepare("SELECT * FROM instances WHERE id = ?")
      .get(instanceId) as Record<string, unknown> | undefined;
    if (!instance) throw new Error(`Instance not found: ${instanceId}`);

    const netId = instance["net_id"] as string;
    const { transitions } = this.getNetWithGraph(netId);
    const transition = transitions.find((t) => t.id === transitionId);
    if (!transition) throw new Error(`Transition not found: ${transitionId}`);
    if (transition.mode !== "judgment") {
      throw new Error(`Transition '${transitionId}' is not a judgment point (mode: ${transition.mode})`);
    }

    // Delegate to fireTransition — it handles authority checks and token flow
    return this.fireTransition(instanceId, transitionId, actorId, decision, evidence, reasoning);
  }
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/core/engine.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: judgment points — getPendingJudgments, resolveJudgment"
```

---

## Task 11: Context assembly — buildWorkOrder

This is the function that assembles the 9-step context for an agent. It reads the transition, gathers token payloads, resolves policies, and builds the structured work order.

**Files:**
- Create: `typescript/src/core/context.ts`
- Create: `typescript/src/core/context.test.ts`

**Step 1: Write failing test**

Create `typescript/src/core/context.test.ts`:

```typescript
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "./engine.js";
import { buildWorkOrder, type WorkOrder } from "./context.js";

describe("buildWorkOrder", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let instanceId: string;

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
      intent: "Verify all required documents are present in the submission",
      mode: "agentic",
      requires_authority: 2,
      context_sources: ["checklist"],
      postconditions: {
        required: ["all-docs-present"],
        desired: ["docs-quality-verified"],
      },
      evidence_requirements: [
        { id: "checklist-result", description: "Completed document checklist", type: "artifact", required: true },
      ],
      available_tools: ["check-documents"],
      input_schema: { type: "object", properties: { applicant: { type: "string" } } },
      output_schema: { type: "object", properties: { complete: { type: "boolean" }, missing: { type: "array" } } },
    });

    engine.attachPolicy(instId, "carta-de-agua.check-completeness", "procedure", "Minimum documents: request form + cadastral plan");
    engine.attachPolicy(instId, "carta-de-agua.*", "preference", "Be specific about missing items");

    const instance = engine.instantiate(netId, "intake", { applicant: "Juan Pérez", cadastral_plan: null });
    instanceId = instance.id;
  });

  afterEach(() => {
    engine.close();
  });

  it("assembles a complete work order from net state", () => {
    const workOrder = buildWorkOrder(engine, instanceId, "check-completeness");

    // 1. Intent
    expect(workOrder.intent).toContain("Verify all required documents");

    // 2. Token payloads
    expect(workOrder.token_payloads).toHaveLength(1);
    expect(workOrder.token_payloads[0]!["applicant"]).toBe("Juan Pérez");

    // 3. Input schema
    expect(workOrder.input_schema).toBeDefined();

    // 4. Policies (ordered by strength)
    expect(workOrder.policies).toHaveLength(2);
    expect(workOrder.policies[0]!.strength).toBe("procedure");

    // 5. Context sources
    expect(workOrder.context_sources).toEqual(["checklist"]);

    // 6. Output schema
    expect(workOrder.output_schema).toBeDefined();

    // 7. Postconditions
    expect(workOrder.postconditions.required).toContain("all-docs-present");

    // 8. Evidence requirements
    expect(workOrder.evidence_requirements).toHaveLength(1);

    // 9. Available tools
    expect(workOrder.available_tools).toContain("check-documents");

    // Mode
    expect(workOrder.mode).toBe("agentic");
  });
});
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/context.test.ts`

Expected: FAIL — `context.js` does not exist.

**Step 3: Implement buildWorkOrder**

Create `typescript/src/core/context.ts`:

```typescript
import type { Engine } from "./engine.js";
import type {
  Policy,
  Postconditions,
  EvidenceRequirement,
  TransitionMode,
  DecisionType,
  JsonSchema,
} from "./types.js";

export interface WorkOrder {
  // Identity
  transition_id: string;
  instance_id: string;
  mode: TransitionMode;
  decision_type?: DecisionType;

  // 1. What am I trying to accomplish?
  intent: string;

  // 2. What case data do I have?
  token_payloads: Record<string, unknown>[];

  // 3. Do I have everything I need?
  input_schema?: JsonSchema;

  // 4. What rules govern this?
  policies: Policy[];

  // 5. What else should I consider?
  context_sources: string[];

  // 6. What data must I produce?
  output_schema?: JsonSchema;

  // 7. What must be true when I'm done?
  postconditions: Postconditions;

  // 8. What proof must I attach?
  evidence_requirements: EvidenceRequirement[];

  // 9. What can I use?
  available_tools: string[];
}

/**
 * Assemble a complete work order for an agent from the current net state.
 * This is the 9-step context assembly from the design doc.
 */
export function buildWorkOrder(
  engine: Engine,
  instanceId: string,
  transitionId: string,
): WorkOrder {
  // Get instance and net
  const marking = engine.getMarking(instanceId);

  // We need the instance to find the net
  // Use a package-internal method — this will be on Engine
  const instance = engine.getInstance(instanceId);
  const { net, transitions } = engine.getNetWithGraph(instance.net_id);
  const transition = transitions.find((t) => t.id === transitionId);
  if (!transition) {
    throw new Error(`Transition '${transitionId}' not found in net '${instance.net_id}'`);
  }

  // 2. Gather token payloads from input places
  const tokenPayloads: Record<string, unknown>[] = [];
  for (const placeId of transition.consumes) {
    const tokens = marking.get(placeId);
    if (tokens) {
      for (const token of tokens) {
        tokenPayloads.push(token.payload);
      }
    }
  }

  // 4. Resolve policies by scope
  const domain = net.domain ?? "";
  const scope = domain ? `${domain}.${transitionId}` : transitionId;
  const policies = engine.getPolicies(scope);

  return {
    transition_id: transitionId,
    instance_id: instanceId,
    mode: transition.mode,
    decision_type: transition.decision_type,
    intent: transition.intent,
    token_payloads: tokenPayloads,
    input_schema: transition.input_schema,
    policies,
    context_sources: transition.context_sources,
    output_schema: transition.output_schema,
    postconditions: transition.postconditions,
    evidence_requirements: transition.evidence_requirements,
    available_tools: transition.available_tools,
  };
}
```

This requires adding `getInstance` to the Engine. Add to `typescript/src/core/engine.ts`:

```typescript
  getInstance(instanceId: string): WorkflowInstance {
    const row = this.db.sqlite
      .prepare("SELECT * FROM instances WHERE id = ?")
      .get(instanceId) as Record<string, unknown> | undefined;
    if (!row) throw new Error(`Instance not found: ${instanceId}`);
    return {
      id: row["id"] as string,
      net_id: row["net_id"] as string,
      status: row["status"] as InstanceStatus,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    };
  }
```

**Step 4: Run test**

Run: `cd typescript && npx vitest run src/core/context.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: context assembly — buildWorkOrder for agent goal construction"
```

---

## Task 12: Validation — validate net structure

**Files:**
- Create: `typescript/src/core/validate.ts`
- Create: `typescript/src/core/validate.test.ts`

**Step 1: Write failing test**

Create `typescript/src/core/validate.test.ts`:

```typescript
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "./engine.js";
import { validateNet } from "./validate.js";

describe("validateNet", () => {
  let engine: Engine;
  let instId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA").id;
  });

  afterEach(() => {
    engine.close();
  });

  it("valid net passes validation", () => {
    const net = engine.createNet(instId, "Simple");
    engine.addPlace(net.id, "start", "Start");
    engine.addPlace(net.id, "end", "End");
    engine.addTransition(net.id, {
      id: "go",
      consumes: ["start"],
      produces: ["end"],
      intent: "Move forward",
      mode: "deterministic",
      requires_authority: 0,
      context_sources: [],
      postconditions: { required: [] },
      evidence_requirements: [],
      available_tools: [],
    });

    const result = validateNet(engine, net.id);
    expect(result.is_valid).toBe(true);
  });

  it("detects orphan places (no arcs)", () => {
    const net = engine.createNet(instId, "Orphan");
    engine.addPlace(net.id, "start", "Start");
    engine.addPlace(net.id, "orphan", "Orphan — no transitions connect here");
    engine.addPlace(net.id, "end", "End");
    engine.addTransition(net.id, {
      id: "go",
      consumes: ["start"],
      produces: ["end"],
      intent: "Move forward",
      mode: "deterministic",
      requires_authority: 0,
      context_sources: [],
      postconditions: { required: [] },
      evidence_requirements: [],
      available_tools: [],
    });

    const result = validateNet(engine, net.id);
    expect(result.violations.some((v) => v.constraint_name === "orphan_place")).toBe(true);
  });

  it("warns when judgment transition has no policies", () => {
    const net = engine.createNet(instId, "No Policy", "test-domain");
    engine.addPlace(net.id, "start", "Start");
    engine.addPlace(net.id, "end", "End");
    engine.addTransition(net.id, {
      id: "decide",
      consumes: ["start"],
      produces: ["end"],
      intent: "Make a judgment",
      mode: "judgment",
      decision_type: "approval",
      requires_authority: 4,
      context_sources: [],
      postconditions: { required: ["decided"] },
      evidence_requirements: [],
      available_tools: [],
    });

    const result = validateNet(engine, net.id);
    expect(result.violations.some((v) => v.constraint_name === "judgment_without_policy")).toBe(true);
  });
});
```

**Step 2: Run to verify failure**

Run: `cd typescript && npx vitest run src/core/validate.test.ts`

Expected: FAIL — `validate.js` does not exist.

**Step 3: Implement validateNet**

Create `typescript/src/core/validate.ts`:

```typescript
import type { Engine } from "./engine.js";
import type { ConstraintViolation, ValidationResult } from "./types.js";

export function validateNet(engine: Engine, netId: string): ValidationResult {
  const { net, places, transitions } = engine.getNetWithGraph(netId);
  const arcs = engine.getArcs(netId);
  const violations: ConstraintViolation[] = [];

  // Check for orphan places (places with no arcs)
  for (const place of places) {
    const hasArc = arcs.some((a) => a.place_id === place.id);
    if (!hasArc) {
      violations.push({
        constraint_name: "orphan_place",
        severity: "warning",
        message: `Place '${place.id}' has no connections to any transition`,
        location: `net:${netId}/place:${place.id}`,
        suggestion: "Connect this place to a transition or remove it",
      });
    }
  }

  // Check judgment transitions have policies
  for (const t of transitions) {
    if (t.mode === "judgment") {
      const domain = net.domain ?? "";
      const scope = domain ? `${domain}.${t.id}` : t.id;
      const policies = engine.getPolicies(scope);
      if (policies.length === 0) {
        violations.push({
          constraint_name: "judgment_without_policy",
          severity: "warning",
          message: `Judgment transition '${t.id}' has no governing policies (scope: ${scope})`,
          location: `net:${netId}/transition:${t.id}`,
          suggestion: `Attach policies to scope '${scope}' or a parent scope`,
        });
      }
    }
  }

  // Check transitions reference valid places (should always pass if addTransition validates, but belt-and-suspenders)
  const placeIds = new Set(places.map((p) => p.id));
  for (const t of transitions) {
    for (const placeId of t.consumes.concat(t.produces)) {
      if (!placeIds.has(placeId)) {
        violations.push({
          constraint_name: "invalid_place_reference",
          severity: "error",
          message: `Transition '${t.id}' references non-existent place '${placeId}'`,
          location: `net:${netId}/transition:${t.id}`,
        });
      }
    }
  }

  // Check for transitions with no input (potential deadlock source)
  for (const t of transitions) {
    if (t.consumes.length === 0) {
      violations.push({
        constraint_name: "sourceless_transition",
        severity: "warning",
        message: `Transition '${t.id}' has no input places — it can fire at any time`,
        location: `net:${netId}/transition:${t.id}`,
        suggestion: "This may be intentional for event-triggered transitions",
      });
    }
  }

  const hasErrors = violations.some((v) => v.severity === "error");
  return { violations, is_valid: !hasErrors };
}
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/core/validate.test.ts`

Expected: PASS

**Step 5: Commit**

```bash
git add -A && git commit -m "feat: net validation — orphan places, policy coverage, structural checks"
```

---

## Task 13: Core module index — export everything

**Files:**
- Create: `typescript/src/core/index.ts`

**Step 1: Create the barrel export**

Create `typescript/src/core/index.ts`:

```typescript
export { Engine } from "./engine.js";
export type { TransitionDef } from "./engine.js";
export { AuditLog } from "./audit.js";
export { buildWorkOrder } from "./context.js";
export type { WorkOrder } from "./context.js";
export { validateNet } from "./validate.js";
export { DB } from "./db.js";

// Re-export all types
export type {
  Institution,
  Role,
  Actor,
  ActorType,
  ActorRoleAssignment,
  PolicyStrength,
  Policy,
  Net,
  Place,
  JsonSchema,
  TransitionMode,
  DecisionType,
  Postconditions,
  EvidenceType,
  EvidenceRequirement,
  Transition,
  ArcDirection,
  Arc,
  Token,
  InstanceStatus,
  WorkflowInstance,
  AuditAction,
  Evidence,
  AuditEntry,
  Severity,
  ConstraintViolation,
  ValidationResult,
  FiringResult,
  PendingJudgment,
} from "./types.js";
```

**Step 2: Verify all tests pass**

Run: `cd typescript && npx vitest run`

Expected: ALL PASS

**Step 3: Verify typecheck**

Run: `cd typescript && npx tsc --noEmit`

Expected: No errors

**Step 4: Commit**

```bash
git add -A && git commit -m "feat: core module barrel export"
```

---

## Task 14: Encode the Carta de Agua process

This is the integration test — encode the ASADA Carta de Agua workflow from the interview and run a case through it.

**Files:**
- Create: `typescript/src/core/carta-de-agua.test.ts`

**Step 1: Write the integration test**

Create `typescript/src/core/carta-de-agua.test.ts`:

```typescript
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
```

**Step 2: Run test**

Run: `cd typescript && npx vitest run src/core/carta-de-agua.test.ts`

Expected: PASS — all assertions should hold given the engine implementation from prior tasks.

**Step 3: Run ALL tests**

Run: `cd typescript && npx vitest run`

Expected: ALL PASS

**Step 4: Commit**

```bash
git add -A && git commit -m "feat: Carta de Agua end-to-end — full process encoded and tested"
```

---

## Task 15: Final verification and typecheck

**Step 1: Run full test suite**

Run: `cd typescript && npx vitest run`

Expected: ALL PASS

**Step 2: Run typecheck**

Run: `cd typescript && npx tsc --noEmit`

Expected: No errors

**Step 3: Commit**

```bash
git add -A && git commit -m "chore: verify all tests pass and types check"
```

---

## Summary

After completing all 15 tasks, the project will have:

- **`src/core/types.ts`** — 25+ types: the unified CPN + institutional model
- **`src/core/db.ts`** — SQLite schema and connection management
- **`src/core/engine.ts`** — The Engine class with ~15 operations: definition, runtime, query
- **`src/core/audit.ts`** — Hash-chained audit log
- **`src/core/context.ts`** — 9-step work order assembly for agents
- **`src/core/validate.ts`** — Net structural validation
- **`src/core/index.ts`** — Barrel export
- **`src/core/carta-de-agua.test.ts`** — Full Carta de Agua process encoded and running end-to-end

The existing `src/spike/` code remains untouched. The existing `src/types/`, `src/cli-bridge/`, `src/orchestration/`, etc. are superseded but not deleted — they can be migrated or removed in a follow-up.
