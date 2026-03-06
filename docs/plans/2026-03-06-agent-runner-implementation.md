# Agent Runner Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the AgentRunner — a standalone class that drives agentic transition execution by consuming WorkOrders, invoking LLM agents with tools, verifying postconditions, and firing transitions through the Engine API.

**Architecture:** Runner pattern — AgentRunner sits above the core engine as a consumer of its API. Three context layers feed the agent: token payloads (case data), institutional context store (standing facts, key-value), and tools (live data). Postconditions are verified via deterministic registry + LLM-as-judge fallback with confidence tracking.

**Tech Stack:** TypeScript (strict, ESM), SQLite via better-sqlite3, pi-agent-core Agent + AgentTool, pi-ai getModel, vitest for tests, zod for validation.

---

### Task 1: Postcondition verification types and result interface

**Files:**
- Modify: `typescript/src/core/types.ts`

**Step 1: Write the new types**

Add after the `FiringResult` interface (after line ~234):

```typescript
// ---------------------------------------------------------------------------
// Postcondition verification
// ---------------------------------------------------------------------------

export type VerificationMethod = "deterministic" | "llm";

export interface PostconditionResult {
  postcondition: string;
  satisfied: boolean;
  method: VerificationMethod;
  confidence: number; // 1.0 for deterministic, 0-1 for LLM
}

// ---------------------------------------------------------------------------
// Agent execution
// ---------------------------------------------------------------------------

export type StepOutcome = "fired" | "escalated" | "no_enabled_transitions" | "error";

export interface StepResult {
  outcome: StepOutcome;
  transition_id?: string | undefined;
  instance_id: string;
  firing_result?: FiringResult | undefined;
  postcondition_results?: PostconditionResult[] | undefined;
  retries_used?: number | undefined;
  error?: string | undefined;
}

export interface RunResult {
  instance_id: string;
  steps: StepResult[];
  final_outcome: "completed" | "stuck" | "escalated" | "error" | "max_steps";
}
```

**Step 2: Export the new types from barrel**

Add to `typescript/src/core/index.ts`:

```typescript
export type {
  VerificationMethod,
  PostconditionResult,
  StepOutcome,
  StepResult,
  RunResult,
} from "./types.js";
```

**Step 3: Run typecheck**

Run: `cd typescript && npx tsc --noEmit`
Expected: no errors

**Step 4: Commit**

```bash
git add typescript/src/core/types.ts typescript/src/core/index.ts
git commit -m "feat: add postcondition verification and agent execution types"
```

---

### Task 2: Institutional context store — schema and implementation

**Files:**
- Modify: `typescript/src/core/db.ts` (add `context_entries` table)
- Create: `typescript/src/core/context-store.ts`
- Create: `typescript/src/core/context-store.test.ts`

**Step 1: Add the context_entries table to db.ts**

Add before the `CREATE INDEX` statements in the SCHEMA string:

```sql
  CREATE TABLE IF NOT EXISTS context_entries (
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    key TEXT NOT NULL,
    value_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (institution_id, key)
  );
```

**Step 2: Write the failing test**

Create `typescript/src/core/context-store.test.ts`:

```typescript
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
```

**Step 3: Run test to verify it fails**

Run: `cd typescript && npx vitest run src/core/context-store.test.ts`
Expected: FAIL (module not found)

**Step 4: Write the implementation**

Create `typescript/src/core/context-store.ts`:

```typescript
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
```

**Step 5: Run tests**

Run: `cd typescript && npx vitest run src/core/context-store.test.ts`
Expected: all 7 tests PASS

**Step 6: Export from barrel**

Add to `typescript/src/core/index.ts`:

```typescript
export { InstitutionalContextStore } from "./context-store.js";
```

**Step 7: Run full test suite**

Run: `cd typescript && npx vitest run`
Expected: all tests pass (existing + new)

**Step 8: Commit**

```bash
git add typescript/src/core/db.ts typescript/src/core/context-store.ts typescript/src/core/context-store.test.ts typescript/src/core/index.ts
git commit -m "feat: institutional context store — key-value store for standing facts"
```

---

### Task 3: Postcondition verifier

**Files:**
- Create: `typescript/src/agent/postconditions.ts`
- Create: `typescript/src/agent/postconditions.test.ts`

**Step 1: Write the failing test**

Create `typescript/src/agent/postconditions.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { PostconditionVerifier } from "./postconditions.js";
import type { Postconditions, PostconditionResult } from "../core/types.js";

// Fake execution result for testing
const makeResult = (toolResults: Array<{ toolName: string; result: Record<string, unknown> }>) => ({
  text: "Agent completed the task.",
  toolResults,
  payload: {},
});

describe("PostconditionVerifier", () => {
  it("uses deterministic verifier when registered", async () => {
    const verifier = new PostconditionVerifier(
      new Map([
        ["doc-generated", (r) => r.toolResults.some((t) => t.toolName === "generate-document" && t.result["success"] === true)],
      ]),
    );

    const result = makeResult([
      { toolName: "generate-document", result: { success: true } },
    ]);

    const postconditions: Postconditions = {
      required: ["doc-generated"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results).toHaveLength(1);
    expect(results[0]!.postcondition).toBe("doc-generated");
    expect(results[0]!.satisfied).toBe(true);
    expect(results[0]!.method).toBe("deterministic");
    expect(results[0]!.confidence).toBe(1.0);
  });

  it("reports deterministic failure correctly", async () => {
    const verifier = new PostconditionVerifier(
      new Map([
        ["doc-generated", (r) => r.toolResults.some((t) => t.toolName === "generate-document" && t.result["success"] === true)],
      ]),
    );

    const result = makeResult([]);

    const postconditions: Postconditions = {
      required: ["doc-generated"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results[0]!.satisfied).toBe(false);
    expect(results[0]!.method).toBe("deterministic");
    expect(results[0]!.confidence).toBe(1.0);
  });

  it("verifies both required and desired postconditions", async () => {
    const verifier = new PostconditionVerifier(
      new Map([
        ["required-thing", () => true],
        ["nice-to-have", () => false],
      ]),
    );

    const result = makeResult([]);
    const postconditions: Postconditions = {
      required: ["required-thing"],
      desired: ["nice-to-have"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results).toHaveLength(2);
    expect(results.find((r) => r.postcondition === "required-thing")!.satisfied).toBe(true);
    expect(results.find((r) => r.postcondition === "nice-to-have")!.satisfied).toBe(false);
  });

  it("checks allRequiredMet correctly", async () => {
    const verifier = new PostconditionVerifier(
      new Map([
        ["a", () => true],
        ["b", () => false],
      ]),
    );

    const result = makeResult([]);
    const postconditions: Postconditions = {
      required: ["a", "b"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(PostconditionVerifier.allRequiredMet(postconditions, results)).toBe(false);
  });

  it("falls back to llm judge when no deterministic verifier and judge provided", async () => {
    const mockJudge = async (_pc: string, _result: unknown): Promise<{ satisfied: boolean; confidence: number }> => {
      return { satisfied: true, confidence: 0.85 };
    };

    const verifier = new PostconditionVerifier(new Map(), mockJudge);

    const result = makeResult([]);
    const postconditions: Postconditions = {
      required: ["semantic-postcondition"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results[0]!.method).toBe("llm");
    expect(results[0]!.satisfied).toBe(true);
    expect(results[0]!.confidence).toBe(0.85);
  });

  it("fails postcondition when no verifier and no judge", async () => {
    const verifier = new PostconditionVerifier(new Map());

    const result = makeResult([]);
    const postconditions: Postconditions = {
      required: ["unverifiable-postcondition"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results[0]!.satisfied).toBe(false);
    expect(results[0]!.method).toBe("deterministic");
    expect(results[0]!.confidence).toBe(0);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd typescript && npx vitest run src/agent/postconditions.test.ts`
Expected: FAIL (module not found)

**Step 3: Write the implementation**

Create `typescript/src/agent/postconditions.ts`:

```typescript
import type { Postconditions, PostconditionResult } from "../core/types.js";

/** The execution evidence passed to verifiers. */
export interface ExecutionEvidence {
  text: string;
  toolResults: Array<{ toolName: string; result: Record<string, unknown> }>;
  payload: Record<string, unknown>;
}

/** A deterministic verifier: inspects execution evidence, returns boolean. */
export type DeterministicVerifier = (evidence: ExecutionEvidence) => boolean;

/** LLM-as-judge function signature. */
export type LlmJudge = (
  postcondition: string,
  evidence: ExecutionEvidence,
) => Promise<{ satisfied: boolean; confidence: number }>;

/**
 * Verifies postconditions using deterministic verifiers with LLM-as-judge fallback.
 * Tracks verification method and confidence for audit trail transparency.
 */
export class PostconditionVerifier {
  constructor(
    private readonly verifiers: Map<string, DeterministicVerifier>,
    private readonly llmJudge?: LlmJudge | undefined,
  ) {}

  /**
   * Verify all postconditions (required + desired) against execution evidence.
   */
  async verify(
    postconditions: Postconditions,
    evidence: ExecutionEvidence,
  ): Promise<PostconditionResult[]> {
    const allPcs = [
      ...postconditions.required,
      ...(postconditions.desired ?? []),
    ];

    const results: PostconditionResult[] = [];

    for (const pc of allPcs) {
      const deterministicFn = this.verifiers.get(pc);

      if (deterministicFn) {
        results.push({
          postcondition: pc,
          satisfied: deterministicFn(evidence),
          method: "deterministic",
          confidence: 1.0,
        });
      } else if (this.llmJudge) {
        const judgment = await this.llmJudge(pc, evidence);
        results.push({
          postcondition: pc,
          satisfied: judgment.satisfied,
          method: "llm",
          confidence: judgment.confidence,
        });
      } else {
        // No verifier available — fail safely
        results.push({
          postcondition: pc,
          satisfied: false,
          method: "deterministic",
          confidence: 0,
        });
      }
    }

    return results;
  }

  /** Check if all required postconditions are satisfied. */
  static allRequiredMet(
    postconditions: Postconditions,
    results: PostconditionResult[],
  ): boolean {
    return postconditions.required.every((pc) => {
      const result = results.find((r) => r.postcondition === pc);
      return result?.satisfied === true;
    });
  }
}
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/agent/postconditions.test.ts`
Expected: all 6 tests PASS

**Step 5: Commit**

```bash
git add typescript/src/agent/postconditions.ts typescript/src/agent/postconditions.test.ts
git commit -m "feat: postcondition verifier — deterministic registry + LLM-as-judge fallback"
```

---

### Task 4: Prompt construction

**Files:**
- Create: `typescript/src/agent/prompt.ts`
- Create: `typescript/src/agent/prompt.test.ts`

**Step 1: Write the failing test**

Create `typescript/src/agent/prompt.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { buildSystemPrompt, buildContextPrompt } from "./prompt.js";
import type { WorkOrder } from "../core/context.js";
import type { Policy } from "../core/types.js";

const makeWorkOrder = (overrides?: Partial<WorkOrder>): WorkOrder => ({
  transition_id: "check-docs",
  instance_id: "inst-1",
  mode: "agentic",
  intent: "Verify all required documents are present",
  token_payloads: [{ applicant: "Juan", doc_type: "residential" }],
  policies: [
    {
      id: "p1",
      institution_id: "i1",
      scope: "carta-de-agua.check-docs",
      strength: "constraint",
      text: "All documents must be verified against the official checklist.",
      created_at: "",
      updated_at: "",
    } satisfies Policy,
  ],
  context_sources: ["document-checklist"],
  postconditions: {
    required: ["all-docs-verified"],
    desired: ["deficiency-notice-sent-if-incomplete"],
  },
  evidence_requirements: [
    { id: "checklist", description: "Completed checklist", type: "artifact", required: true },
  ],
  available_tools: ["verify-documents", "send-notification"],
  ...overrides,
});

describe("buildSystemPrompt", () => {
  it("includes intent, postconditions, policies, evidence requirements, and tools", () => {
    const prompt = buildSystemPrompt(makeWorkOrder());

    expect(prompt).toContain("Verify all required documents are present");
    expect(prompt).toContain("all-docs-verified");
    expect(prompt).toContain("deficiency-notice-sent-if-incomplete");
    expect(prompt).toContain("official checklist");
    expect(prompt).toContain("Completed checklist");
    expect(prompt).toContain("verify-documents");
    expect(prompt).toContain("send-notification");
  });

  it("includes output schema when present", () => {
    const wo = makeWorkOrder({
      output_schema: { type: "object", properties: { complete: { type: "boolean" } } },
    });
    const prompt = buildSystemPrompt(wo);
    expect(prompt).toContain('"complete"');
  });

  it("labels policy strengths", () => {
    const prompt = buildSystemPrompt(makeWorkOrder());
    expect(prompt).toMatch(/constraint/i);
  });
});

describe("buildContextPrompt", () => {
  it("includes token payloads and institutional context", () => {
    const wo = makeWorkOrder();
    const institutionalContext = { "document-checklist": "1. Request form\n2. Cadastral plan" };

    const prompt = buildContextPrompt(wo, institutionalContext);
    expect(prompt).toContain("Juan");
    expect(prompt).toContain("Cadastral plan");
  });

  it("handles empty institutional context", () => {
    const wo = makeWorkOrder();
    const prompt = buildContextPrompt(wo, {});
    expect(prompt).toContain("Juan");
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd typescript && npx vitest run src/agent/prompt.test.ts`
Expected: FAIL (module not found)

**Step 3: Write the implementation**

Create `typescript/src/agent/prompt.ts`:

```typescript
import type { WorkOrder } from "../core/context.js";

/**
 * Build the system prompt for an agentic transition.
 * Contains: task intent, policies, postconditions, evidence requirements,
 * output schema, and available tools.
 */
export function buildSystemPrompt(workOrder: WorkOrder): string {
  const requiredPcs = workOrder.postconditions.required
    .map((pc) => `- [ ] ${pc}`)
    .join("\n");

  const desiredPcs = workOrder.postconditions.desired
    ?.map((pc) => `- [ ] ${pc}`)
    .join("\n");

  const policySection = workOrder.policies
    .map((p) => `[${p.strength.toUpperCase()}] ${p.text}`)
    .join("\n");

  const evidenceSection = workOrder.evidence_requirements
    .map((e) => `- ${e.description} (${e.type}${e.required ? ", required" : ""})`)
    .join("\n");

  const outputSchemaSection = workOrder.output_schema
    ? `\n## Output Schema\nYour output must conform to:\n\`\`\`json\n${JSON.stringify(workOrder.output_schema, null, 2)}\n\`\`\``
    : "";

  return `You are executing a single transition in an institutional workflow.

## Your Task
${workOrder.intent}

## Governing Policies
${policySection || "No specific policies."}

## Success Criteria
The following MUST be true when you are done:
${requiredPcs}
${desiredPcs ? `\nThe following are desirable but not required:\n${desiredPcs}` : ""}

## Evidence Requirements
${evidenceSection || "No evidence requirements."}
${outputSchemaSection}

## Available Tools
${workOrder.available_tools.join(", ") || "None"}

## Instructions
1. Review the context, policies, and success criteria carefully.
2. Use the available tools to accomplish the task.
3. Ensure all required postconditions are satisfied.
4. Capture required evidence.

After completing your work, provide a structured summary:
- Which success criteria you satisfied and the evidence.
- Any issues encountered.
- Data or artifacts produced.

IMPORTANT: You must actually call the tools. Do not just describe what you would do.`;
}

/**
 * Build the context prompt with case data and institutional context.
 */
export function buildContextPrompt(
  workOrder: WorkOrder,
  institutionalContext: Record<string, unknown>,
): string {
  const tokenSection = workOrder.token_payloads
    .map((p, i) => `### Token ${i + 1}\n${JSON.stringify(p, null, 2)}`)
    .join("\n\n");

  const contextEntries = Object.entries(institutionalContext);
  const contextSection = contextEntries.length > 0
    ? contextEntries
        .map(([key, value]) => {
          const formatted = typeof value === "string" ? value : JSON.stringify(value, null, 2);
          return `### ${key}\n${formatted}`;
        })
        .join("\n\n")
    : "No additional institutional context.";

  return `## Case Data
${tokenSection || "No token data."}

## Institutional Context
${contextSection}

Please proceed with executing this transition.`;
}
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/agent/prompt.test.ts`
Expected: all 5 tests PASS

**Step 5: Commit**

```bash
git add typescript/src/agent/prompt.ts typescript/src/agent/prompt.test.ts
git commit -m "feat: prompt construction — system + context prompts from work orders"
```

---

### Task 5: AgentRunner — core step/run logic

**Files:**
- Create: `typescript/src/agent/runner.ts`
- Create: `typescript/src/agent/runner.test.ts`

This is the main component. Tests will use a mock strategy to avoid actual LLM calls.

**Step 1: Write the failing test**

Create `typescript/src/agent/runner.test.ts`:

```typescript
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "../core/engine.js";
import { InstitutionalContextStore } from "../core/context-store.js";
import { PostconditionVerifier } from "./postconditions.js";
import { AgentRunner } from "./runner.js";
import type { StepResult } from "../core/types.js";

/**
 * Mock agent executor — replaces actual LLM calls.
 * Returns canned responses keyed by transition ID.
 */
function mockExecutor(responses: Record<string, {
  text: string;
  toolResults: Array<{ toolName: string; result: Record<string, unknown> }>;
  payload: Record<string, unknown>;
}>) {
  return async (transitionId: string, _systemPrompt: string, _contextPrompt: string, _tools: unknown[]) => {
    const response = responses[transitionId];
    if (!response) throw new Error(`No mock response for transition: ${transitionId}`);
    return response;
  };
}

/** Mock executor that throws a runtime error */
function throwingExecutor(errorMessage: string) {
  return async () => {
    throw new Error(errorMessage);
  };
}

describe("AgentRunner", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let actorId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    const inst = engine.createInstitution("Test Org");
    instId = inst.id;

    const role = engine.createRole(instId, "operator", 5);
    const actor = engine.createActor(instId, "Agent Bot", "agent");
    engine.assignRole(actor.id, role.id);
    actorId = actor.id;

    const net = engine.createNet(instId, "Test Flow", "test");
    netId = net.id;

    engine.addPlace(netId, "start", "Starting place");
    engine.addPlace(netId, "middle", "Middle place");
    engine.addPlace(netId, "end", "End place");

    engine.addTransition(netId, {
      id: "step-one",
      consumes: ["start"],
      produces: ["middle"],
      intent: "Do the first thing",
      mode: "agentic",
      requires_authority: 1,
      context_sources: [],
      postconditions: { required: ["thing-done"] },
      evidence_requirements: [],
      available_tools: ["tool-a"],
    });

    engine.addTransition(netId, {
      id: "step-two",
      consumes: ["middle"],
      produces: ["end"],
      intent: "Do the second thing",
      mode: "agentic",
      requires_authority: 1,
      context_sources: [],
      postconditions: { required: ["second-done"] },
      evidence_requirements: [],
      available_tools: ["tool-b"],
    });
  });

  afterEach(() => {
    engine.close();
  });

  it("step() fires one agentic transition and returns result", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([["thing-done", () => true]]),
    );

    const executor = mockExecutor({
      "step-one": {
        text: "Done",
        toolResults: [{ toolName: "tool-a", result: { ok: true } }],
        payload: { step_one_output: true },
      },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.step(instance.id, actorId);

    expect(result.outcome).toBe("fired");
    expect(result.transition_id).toBe("step-one");
    expect(result.firing_result?.success).toBe(true);
    expect(result.postcondition_results?.[0]?.satisfied).toBe(true);

    // Token should now be in "middle"
    const marking = engine.getMarking(instance.id);
    expect(marking.has("middle")).toBe(true);
    expect(marking.has("start")).toBe(false);
  });

  it("step() returns no_enabled_transitions when none available", async () => {
    const instance = engine.instantiate(netId, "end", { done: true });

    const verifier = new PostconditionVerifier(new Map());
    const runner = new AgentRunner(engine, verifier, { executor: mockExecutor({}) });

    const result = await runner.step(instance.id, actorId);
    expect(result.outcome).toBe("no_enabled_transitions");
  });

  it("step() skips judgment transitions", async () => {
    // Add a judgment transition from start
    engine.addPlace(netId, "judgment-out", "After judgment");
    engine.addTransition(netId, {
      id: "judge-it",
      consumes: ["start"],
      produces: ["judgment-out"],
      intent: "Make a judgment call",
      mode: "judgment",
      decision_type: "approval",
      requires_authority: 1,
      context_sources: [],
      postconditions: { required: ["decided"] },
      evidence_requirements: [],
      available_tools: [],
    });

    // Remove the agentic transition's input tokens — put token only at start
    // Both judge-it and step-one consume from "start", but runner should only fire agentic
    const instance = engine.instantiate(netId, "start", { data: "test" });

    const verifier = new PostconditionVerifier(
      new Map([["thing-done", () => true]]),
    );
    const executor = mockExecutor({
      "step-one": {
        text: "Done",
        toolResults: [],
        payload: { done: true },
      },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.step(instance.id, actorId);

    // Should fire step-one (agentic), not judge-it (judgment)
    expect(result.outcome).toBe("fired");
    expect(result.transition_id).toBe("step-one");
  });

  it("run() executes all agentic transitions until none remain", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([
        ["thing-done", () => true],
        ["second-done", () => true],
      ]),
    );

    const executor = mockExecutor({
      "step-one": { text: "Done 1", toolResults: [], payload: { step: 1 } },
      "step-two": { text: "Done 2", toolResults: [], payload: { step: 2 } },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.run(instance.id, actorId);

    expect(result.final_outcome).toBe("completed");
    expect(result.steps).toHaveLength(2);
    expect(result.steps[0]!.transition_id).toBe("step-one");
    expect(result.steps[1]!.transition_id).toBe("step-two");

    // Final marking should be at "end"
    const marking = engine.getMarking(instance.id);
    expect(marking.has("end")).toBe(true);
  });

  it("escalates when postconditions fail", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([["thing-done", () => false]]), // always fails
    );

    const executor = mockExecutor({
      "step-one": { text: "Tried", toolResults: [], payload: {} },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.step(instance.id, actorId);

    expect(result.outcome).toBe("escalated");
    expect(result.transition_id).toBe("step-one");
    // Token should NOT have moved
    const marking = engine.getMarking(instance.id);
    expect(marking.has("start")).toBe(true);
    expect(marking.has("middle")).toBe(false);
  });

  it("retries on runtime error then escalates after max retries", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(new Map());
    const executor = throwingExecutor("Network timeout");

    const runner = new AgentRunner(engine, verifier, { executor, maxRetries: 2 });
    const result = await runner.step(instance.id, actorId);

    expect(result.outcome).toBe("error");
    expect(result.retries_used).toBe(2);
    expect(result.error).toContain("Network timeout");

    // Token should NOT have moved
    const marking = engine.getMarking(instance.id);
    expect(marking.has("start")).toBe(true);
  });

  it("run() stops at max steps", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([
        ["thing-done", () => true],
        ["second-done", () => true],
      ]),
    );

    const executor = mockExecutor({
      "step-one": { text: "Done 1", toolResults: [], payload: { step: 1 } },
      "step-two": { text: "Done 2", toolResults: [], payload: { step: 2 } },
    });

    const runner = new AgentRunner(engine, verifier, { executor, maxSteps: 1 });
    const result = await runner.run(instance.id, actorId);

    expect(result.final_outcome).toBe("max_steps");
    expect(result.steps).toHaveLength(1);
  });

  it("run() reports escalated when postcondition fails mid-run", async () => {
    const instance = engine.instantiate(netId, "start", { data: "hello" });

    const verifier = new PostconditionVerifier(
      new Map([
        ["thing-done", () => true],
        ["second-done", () => false], // fails on step two
      ]),
    );

    const executor = mockExecutor({
      "step-one": { text: "Done 1", toolResults: [], payload: { step: 1 } },
      "step-two": { text: "Tried", toolResults: [], payload: {} },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.run(instance.id, actorId);

    expect(result.final_outcome).toBe("escalated");
    expect(result.steps).toHaveLength(2);
    expect(result.steps[0]!.outcome).toBe("fired");
    expect(result.steps[1]!.outcome).toBe("escalated");
  });
});
```

**Step 2: Run test to verify it fails**

Run: `cd typescript && npx vitest run src/agent/runner.test.ts`
Expected: FAIL (module not found)

**Step 3: Write the implementation**

Create `typescript/src/agent/runner.ts`:

```typescript
import type { Engine } from "../core/engine.js";
import type { StepResult, RunResult, Transition } from "../core/types.js";
import { buildWorkOrder } from "../core/context.js";
import type { InstitutionalContextStore } from "../core/context-store.js";
import { PostconditionVerifier, type ExecutionEvidence } from "./postconditions.js";
import { buildSystemPrompt, buildContextPrompt } from "./prompt.js";

/** The function signature for executing a transition via LLM. */
export type TransitionExecutor = (
  transitionId: string,
  systemPrompt: string,
  contextPrompt: string,
  tools: unknown[],
) => Promise<ExecutionEvidence>;

export interface AgentRunnerOptions {
  /** Function that executes a transition (LLM agent call). */
  executor: TransitionExecutor;
  /** Institutional context store for standing facts. Optional. */
  contextStore?: InstitutionalContextStore | undefined;
  /** Tool registry. Maps tool name to tool object. Optional — tools are passed through to executor. */
  toolRegistry?: Map<string, unknown> | undefined;
  /** Max retries for runtime errors. Default: 3. */
  maxRetries?: number | undefined;
  /** Max steps for run(). Default: 20. */
  maxSteps?: number | undefined;
}

/**
 * AgentRunner drives agentic transition execution.
 * Sits above the Engine — observes enabled transitions, builds work orders,
 * invokes the executor (LLM), verifies postconditions, and fires transitions.
 */
export class AgentRunner {
  private readonly maxRetries: number;
  private readonly maxSteps: number;

  constructor(
    private readonly engine: Engine,
    private readonly verifier: PostconditionVerifier,
    private readonly options: AgentRunnerOptions,
  ) {
    this.maxRetries = options.maxRetries ?? 3;
    this.maxSteps = options.maxSteps ?? 20;
  }

  /**
   * Execute one enabled agentic transition.
   * Returns the step result (fired, escalated, no_enabled_transitions, or error).
   */
  async step(instanceId: string, actorId: string): Promise<StepResult> {
    // 1. Find enabled agentic transitions
    const enabled = this.engine.getEnabledTransitions(instanceId, actorId);
    const agenticTransitions = enabled.filter((t) => t.mode === "agentic");

    if (agenticTransitions.length === 0) {
      return { outcome: "no_enabled_transitions", instance_id: instanceId };
    }

    const transition = agenticTransitions[0]!;
    return this.executeTransition(instanceId, actorId, transition);
  }

  /**
   * Loop step() until no agentic transitions remain or limits are hit.
   */
  async run(instanceId: string, actorId: string): Promise<RunResult> {
    const steps: StepResult[] = [];

    for (let i = 0; i < this.maxSteps; i++) {
      const result = await this.step(instanceId, actorId);
      steps.push(result);

      if (result.outcome === "no_enabled_transitions") {
        // Remove the last "no transitions" step from results — it's not a real step
        steps.pop();
        return { instance_id: instanceId, steps, final_outcome: "completed" };
      }

      if (result.outcome === "escalated") {
        return { instance_id: instanceId, steps, final_outcome: "escalated" };
      }

      if (result.outcome === "error") {
        return { instance_id: instanceId, steps, final_outcome: "error" };
      }
    }

    return { instance_id: instanceId, steps, final_outcome: "max_steps" };
  }

  private async executeTransition(
    instanceId: string,
    actorId: string,
    transition: Transition,
  ): Promise<StepResult> {
    // 2. Build work order
    const workOrder = buildWorkOrder(this.engine, instanceId, transition.id);

    // 3. Resolve institutional context
    const institutionalContext = this.options.contextStore
      ? this.options.contextStore.resolve(workOrder.context_sources)
      : {};

    // 4. Build prompts
    const systemPrompt = buildSystemPrompt(workOrder);
    const contextPrompt = buildContextPrompt(workOrder, institutionalContext);

    // 5. Resolve tools
    const tools: unknown[] = [];
    if (this.options.toolRegistry) {
      for (const toolName of workOrder.available_tools) {
        const tool = this.options.toolRegistry.get(toolName);
        if (tool) tools.push(tool);
      }
    }

    // 6. Execute with retry on runtime errors
    let evidence: ExecutionEvidence | undefined;
    let retries = 0;
    let lastError: string | undefined;

    while (retries <= this.maxRetries) {
      try {
        evidence = await this.options.executor(
          transition.id,
          systemPrompt,
          contextPrompt,
          tools,
        );
        break; // success — exit retry loop
      } catch (err) {
        lastError = err instanceof Error ? err.message : String(err);
        retries++;
        if (retries > this.maxRetries) {
          return {
            outcome: "error",
            transition_id: transition.id,
            instance_id: instanceId,
            retries_used: this.maxRetries,
            error: lastError,
          };
        }
      }
    }

    // 7. Verify postconditions
    const pcResults = await this.verifier.verify(
      workOrder.postconditions,
      evidence!,
    );

    const allMet = PostconditionVerifier.allRequiredMet(
      workOrder.postconditions,
      pcResults,
    );

    if (!allMet) {
      return {
        outcome: "escalated",
        transition_id: transition.id,
        instance_id: instanceId,
        postcondition_results: pcResults,
      };
    }

    // 8. Fire transition through the engine
    const firingResult = this.engine.fireTransition(
      instanceId,
      transition.id,
      actorId,
      evidence!.payload,
      undefined, // evidence — TODO: map from execution evidence
      `Agent executed transition. Postconditions verified: ${pcResults.map((r) => `${r.postcondition}=${r.satisfied}(${r.method})`).join(", ")}`,
    );

    return {
      outcome: firingResult.success ? "fired" : "error",
      transition_id: transition.id,
      instance_id: instanceId,
      firing_result: firingResult,
      postcondition_results: pcResults,
      retries_used: retries > 0 ? retries : undefined,
      error: firingResult.success ? undefined : firingResult.error,
    };
  }
}
```

**Step 4: Run tests**

Run: `cd typescript && npx vitest run src/agent/runner.test.ts`
Expected: all 8 tests PASS

**Step 5: Run full test suite**

Run: `cd typescript && npx vitest run`
Expected: all tests pass (existing + new)

**Step 6: Commit**

```bash
git add typescript/src/agent/runner.ts typescript/src/agent/runner.test.ts
git commit -m "feat: AgentRunner — step/run loop with retry, escalation, postcondition verification"
```

---

### Task 6: Agent module barrel export

**Files:**
- Create: `typescript/src/agent/index.ts`
- Modify: `typescript/src/index.ts`

**Step 1: Create agent barrel**

Create `typescript/src/agent/index.ts`:

```typescript
export { AgentRunner, type TransitionExecutor, type AgentRunnerOptions } from "./runner.js";
export {
  PostconditionVerifier,
  type DeterministicVerifier,
  type LlmJudge,
  type ExecutionEvidence,
} from "./postconditions.js";
export { buildSystemPrompt, buildContextPrompt } from "./prompt.js";
```

**Step 2: Add to package entry**

Add to `typescript/src/index.ts`:

```typescript
// Agent layer
export {
  AgentRunner,
  type TransitionExecutor,
  type AgentRunnerOptions,
  PostconditionVerifier,
  type DeterministicVerifier,
  type LlmJudge,
  type ExecutionEvidence,
  buildSystemPrompt,
  buildContextPrompt,
} from "./agent/index.js";
```

**Step 3: Typecheck**

Run: `cd typescript && npx tsc --noEmit`
Expected: no errors

**Step 4: Run full test suite**

Run: `cd typescript && npx vitest run`
Expected: all tests pass

**Step 5: Commit**

```bash
git add typescript/src/agent/index.ts typescript/src/index.ts
git commit -m "feat: agent module barrel export — public API for intelligence layer"
```

---

### Task 7: Integration test — Carta de Agua with AgentRunner

**Files:**
- Create: `typescript/src/agent/carta-de-agua-runner.test.ts`

This test validates the runner against the real Carta de Agua workflow, using mock executors for the three agentic transitions.

**Step 1: Write the integration test**

Create `typescript/src/agent/carta-de-agua-runner.test.ts`:

```typescript
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { Engine } from "../core/engine.js";
import { PostconditionVerifier } from "./postconditions.js";
import { AgentRunner } from "./runner.js";

describe("Carta de Agua — AgentRunner integration", () => {
  let engine: Engine;
  let instId: string;
  let netId: string;
  let adminId: string;
  let boardId: string;

  beforeEach(() => {
    engine = new Engine(":memory:");
    instId = engine.createInstitution("ASADA Nosara").id;

    const adminRole = engine.createRole(instId, "administrator", 2);
    const boardRole = engine.createRole(instId, "junta-directiva", 4);

    const admin = engine.createActor(instId, "Don Carlos", "human");
    engine.assignRole(admin.id, adminRole.id);
    adminId = admin.id;

    const board = engine.createActor(instId, "Junta Directiva", "human");
    engine.assignRole(board.id, boardRole.id);
    boardId = board.id;

    const net = engine.createNet(instId, "Carta de Agua", "carta-de-agua");
    netId = net.id;

    // Minimal workflow: intake → [receive-request] → docs-pending → [check-completeness] → docs-complete → [triage] → triaged
    engine.addPlace(netId, "intake", "Request received");
    engine.addPlace(netId, "docs-pending", "Awaiting documents");
    engine.addPlace(netId, "docs-complete", "Documents verified");
    engine.addPlace(netId, "triaged", "Case classified");

    engine.addTransition(netId, {
      id: "receive-request",
      consumes: ["intake"],
      produces: ["docs-pending"],
      intent: "Assign case ID, send receipt",
      mode: "deterministic",
      requires_authority: 2,
      context_sources: [],
      postconditions: { required: ["case-id-assigned"] },
      evidence_requirements: [],
      available_tools: [],
    });

    engine.addTransition(netId, {
      id: "check-completeness",
      consumes: ["docs-pending"],
      produces: ["docs-complete"],
      intent: "Verify documents against checklist",
      mode: "agentic",
      requires_authority: 2,
      context_sources: ["document-checklist"],
      postconditions: { required: ["all-docs-verified"] },
      evidence_requirements: [],
      available_tools: ["verify-documents"],
    });

    engine.addTransition(netId, {
      id: "triage-case",
      consumes: ["docs-complete"],
      produces: ["triaged"],
      intent: "Classify case by impact level",
      mode: "judgment",
      decision_type: "classification",
      requires_authority: 2,
      context_sources: [],
      postconditions: { required: ["classified"] },
      evidence_requirements: [],
      available_tools: [],
    });
  });

  afterEach(() => {
    engine.close();
  });

  it("runner fires agentic transitions, stops at judgment points", async () => {
    // 1. Start at intake
    const instance = engine.instantiate(netId, "intake", { applicant: "Juan" });

    // 2. Manually fire the deterministic transition (runner doesn't handle these)
    engine.fireTransition(instance.id, "receive-request", adminId, {
      case_id: "CDA-001",
    });

    // 3. Now token is at docs-pending — runner should fire check-completeness
    const verifier = new PostconditionVerifier(
      new Map([["all-docs-verified", () => true]]),
    );

    const executor = async () => ({
      text: "All documents verified.",
      toolResults: [{ toolName: "verify-documents", result: { complete: true } }],
      payload: { complete: true, missing: [] },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    const result = await runner.run(instance.id, adminId);

    // Should fire check-completeness, then stop (triage is judgment)
    expect(result.final_outcome).toBe("completed");
    expect(result.steps).toHaveLength(1);
    expect(result.steps[0]!.transition_id).toBe("check-completeness");

    // Token should now be at docs-complete
    const marking = engine.getMarking(instance.id);
    expect(marking.has("docs-complete")).toBe(true);

    // Triage should be pending as a judgment
    const pending = engine.getPendingJudgments(instance.id);
    expect(pending).toHaveLength(1);
    expect(pending[0]!.transition_id).toBe("triage-case");
  });

  it("audit trail captures agent execution details", async () => {
    const instance = engine.instantiate(netId, "docs-pending", { applicant: "María" });

    const verifier = new PostconditionVerifier(
      new Map([["all-docs-verified", () => true]]),
    );

    const executor = async () => ({
      text: "Verified.",
      toolResults: [],
      payload: { complete: true },
    });

    const runner = new AgentRunner(engine, verifier, { executor });
    await runner.step(instance.id, adminId);

    const history = engine.getHistory(instance.id);
    const agentEntry = history.find((e) => e.transition_id === "check-completeness");
    expect(agentEntry).toBeDefined();
    expect(agentEntry!.action).toBe("transition_fired");
    expect(agentEntry!.reasoning).toContain("Postconditions verified");
    expect(agentEntry!.reasoning).toContain("all-docs-verified=true(deterministic)");
  });
});
```

**Step 2: Run the integration test**

Run: `cd typescript && npx vitest run src/agent/carta-de-agua-runner.test.ts`
Expected: all 2 tests PASS

**Step 3: Run full test suite**

Run: `cd typescript && npx vitest run`
Expected: all tests pass

**Step 4: Commit**

```bash
git add typescript/src/agent/carta-de-agua-runner.test.ts
git commit -m "test: Carta de Agua integration test with AgentRunner"
```

---

### Task 8: Typecheck, full test run, update design doc

**Step 1: Typecheck**

Run: `cd typescript && npx tsc --noEmit`
Expected: no errors

**Step 2: Full test suite**

Run: `cd typescript && npx vitest run`
Expected: all tests pass

**Step 3: Update architecture doc milestone**

In `intelligent-institution-architecture.md`, update milestone 4 from "Next" to "✓":

Change:
```
4. **Next: Agent runtime.**
```
To:
```
4. **✓ Agent runtime (phase 1).** AgentRunner with step/run loop, postcondition verification (deterministic + LLM fallback with confidence tracking), institutional context store, prompt construction. Mock executor for testing; real LLM integration deferred to phase 2.
```

**Step 4: Commit**

```bash
git add -A
git commit -m "docs: update milestones — agent runtime phase 1 complete"
```
