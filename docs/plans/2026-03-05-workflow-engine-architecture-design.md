# Workflow Engine Architecture Design

## Overview

A TypeScript library that models institutional workflows as Colored Petri Nets with institutional semantics. The CPN is the runtime reality — places are states, transitions are judgment points or agentic tasks, tokens carry case data. Institutions define who can fire what. Agents handle telemetry (email, document generation, data gathering); humans make judgment calls. The library exposes ~15 operations consumed by a CLI and HTTP API.

---

## Core Model

### Hierarchy

```
Institution
├── Role (authority level)
├── Actor (person or agent, assigned roles)
└── Net (workflow)
    ├── Place (state between decisions)
    └── Transition (judgment point or agentic task)
        ├── Policies (by formality layer)
        ├── Postconditions
        └── Evidence Requirements
```

An **Institution** is the root container. It has **Roles** with authority levels. **Actors** (humans or AI agents) are assigned roles. **Nets** are workflows belonging to the institution. Transitions within nets require specific authority to fire. Every firing is audit-logged with actor, role, and reasoning.

### Types

#### Institution

```typescript
interface Institution {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}
```

#### Role

```typescript
interface Role {
  id: string;
  institution_id: string;
  name: string;                  // e.g., "administrator", "junta-directiva"
  description?: string;
  authority_level: number;       // 0 = lowest
  created_at: string;
  updated_at: string;
}
```

#### Actor

```typescript
interface Actor {
  id: string;
  institution_id: string;
  name: string;
  type: "human" | "agent";
  role_ids: string[];            // assigned roles
  created_at: string;
  updated_at: string;
}
```

#### Place

```typescript
interface Place {
  id: string;
  net_id: string;
  description: string;
  schema?: JsonSchema;           // optional color set — when present, engine validates token payloads
}
```

#### Transition

The core type. Merges CPN formalism with institutional semantics.

```typescript
interface Transition {
  id: string;
  net_id: string;

  // -- CPN core --
  consumes: string[];            // input place IDs
  produces: string[];            // output place IDs
  guard?: string;                // boolean expression over token data

  // -- Institutional semantics --
  intent: string;                // natural language goal — what this transition accomplishes
  mode: "deterministic" | "judgment" | "agentic";
  decision_type?: DecisionType;  // approval, classification, prioritization, allocation, exception_handling
  requires_authority: number;    // minimum authority level to fire
  authorized_roles?: string[];   // optional fine-grained role restriction

  // -- Policy attachment --
  policies: string[];            // IDs of attached policies, resolved by scope at query time

  // -- Data flow --
  input_schema?: JsonSchema;     // expected shape of incoming token payload — agent validates before acting
  output_schema?: JsonSchema;    // expected shape of outgoing token payload — what the agent must produce
  context_sources: string[];     // keys to resolve from context store (accumulated case data, policy docs)

  // -- Execution contract --
  postconditions: Postconditions;
  evidence_requirements: EvidenceRequirement[];
  available_tools: string[];
  timeout?: number;              // max execution time in seconds
}

type DecisionType =
  | "approval"
  | "classification"
  | "prioritization"
  | "allocation"
  | "exception_handling";

interface Postconditions {
  required: string[];            // must all be true for firing to succeed
  desired?: string[];            // logged but non-blocking
  escalation?: string[];         // triggered if required postconditions fail
}

interface EvidenceRequirement {
  id: string;
  description: string;           // what artifact must be attached — e.g., "WhatsApp message ID"
  type: "artifact" | "reference" | "attestation";
  required: boolean;
}
```

**Key distinction — postconditions vs. evidence requirements:**

- **Postconditions** answer "what must be true?" — `notification-sent`, `risk-level-determined`. They're boolean conditions verified by deterministic checkers or LLM-as-judge.
- **Evidence requirements** answer "what proof must be attached?" — `message-id`, `inspection-report-pdf`, `board-resolution-number`. They're artifacts that go into the audit trail. An agent can satisfy a postcondition without providing evidence (the postcondition checker says "yes, notification was sent") but the evidence requirement forces it to also capture the receipt.

#### Net

```typescript
interface Net {
  id: string;
  institution_id: string;
  domain?: string;               // functional domain — e.g., "carta-de-agua", "procurement"
  name: string;
  description?: string;
  places: Place[];
  transitions: Transition[];
  created_at: string;
  updated_at: string;
}
```

#### Token and Marking (runtime)

```typescript
interface Token {
  id: string;
  place_id: string;
  payload: Record<string, unknown>;   // unstructured by default
  created_at: string;
}

/** A marking is the full runtime state — which places hold which tokens. */
type Marking = Map<string, Token[]>;
```

#### Policy

```typescript
type PolicyStrength = "constraint" | "procedure" | "preference" | "context";

interface Policy {
  id: string;
  institution_id: string;
  scope: string;                 // dot-separated — "carta-de-agua.board-decision", "carta-de-agua.*", "*"
  strength: PolicyStrength;
  text: string;                  // structured natural language
  metadata?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}
```

Policies are resolved by scope specificity and ordered by formality layer (constraint → procedure → preference → context). An agent querying policies for a transition gets them in the order that matters: hard rules first, cultural context last.

#### Audit Entry

```typescript
interface AuditEntry {
  id: string;
  instance_id: string;
  timestamp: string;
  sequence: number;
  action: AuditAction;
  actor: { actor_id: string; role_id: string; authority_level: number };
  transition_id?: string;
  marking_before?: Record<string, unknown>;
  marking_after?: Record<string, unknown>;
  evidence?: Evidence[];         // artifacts attached by agent or human
  reasoning?: string;
  prev_hash: string;
  entry_hash: string;
}

interface Evidence {
  requirement_id: string;        // links to EvidenceRequirement
  type: "artifact" | "reference" | "attestation";
  content: unknown;              // the actual proof — message ID, file reference, signed statement
  captured_at: string;
}

type AuditAction =
  | "instance_created"
  | "transition_fired"
  | "judgment_pending"
  | "judgment_resolved"
  | "postcondition_failed"
  | "escalation_triggered"
  | "policy_consulted"
  | "override_applied";
```

#### Workflow Instance

```typescript
interface WorkflowInstance {
  id: string;
  net_id: string;
  marking: Marking;
  status: "running" | "completed" | "stuck" | "suspended";
  created_at: string;
  updated_at: string;
}
```

---

## API Surface

### Definition Operations

Build the institutional model. Each operation writes an audit entry.

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `createInstitution` | `(name, description?) → Institution` | Create the root entity |
| `createRole` | `(institution_id, name, authority_level) → Role` | Define an authority level |
| `createActor` | `(institution_id, name, type) → Actor` | Register a person or agent |
| `assignRole` | `(actor_id, role_id) → void` | Grant authority to an actor |
| `createNet` | `(institution_id, name, domain?) → Net` | Create an empty workflow |
| `addPlace` | `(net_id, id, description, schema?) → Place` | Add a state to the net |
| `addTransition` | `(net_id, TransitionDef) → Transition` | Add a transition with full metadata |
| `attachPolicy` | `(institution_id, scope, strength, text) → Policy` | Attach policy to a scope |

### Runtime Operations

Execute workflows. The engine enforces authority and logs everything.

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `instantiate` | `(net_id, initial_payload) → WorkflowInstance` | Create a marking with a token in the start place |
| `getEnabledTransitions` | `(instance_id, actor_id) → Transition[]` | What can this actor fire right now? Checks marking + authority |
| `fireTransition` | `(instance_id, transition_id, actor_id, input?) → FiringResult` | Execute: check authority → run agent/tools → verify postconditions → collect evidence → update marking → write audit |
| `getPendingJudgments` | `(instance_id?) → PendingJudgment[]` | Transitions enabled but requiring human resolution |
| `resolveJudgment` | `(instance_id, transition_id, actor_id, decision, reasoning) → FiringResult` | Human provides decision for a judgment point |

### Query Operations

Read the model and its history.

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `getPolicies` | `(scope) → Policy[]` | Ordered by strength and specificity |
| `getHistory` | `(instance_id?, actor_id?, transition_id?) → AuditEntry[]` | Filtered audit trail |
| `validate` | `(net_id) → ValidationResult` | Structural checks: reachability, authority coverage, policy gaps |
| `getMarking` | `(instance_id) → Marking` | Current token state |

---

## Agent Context Assembly

When an agent encounters an enabled transition, the engine assembles a complete work order from the net state. This is mechanical, not creative — the agent doesn't need to figure out what to do, only how to do it.

### Assembly sequence

```
1. Read transition.intent           → "What am I trying to accomplish?"
2. Read token payloads              → "What case data do I have?"
   (from input places)
3. Validate against input_schema    → "Do I have everything I need?"
   (if schema exists)
4. Resolve policies by scope        → "What rules govern this?"
   (ordered: constraint → context)
5. Read context_sources             → "What else should I consider?"
   (prior decisions, policy docs)
6. Read output_schema               → "What data must I produce?"
7. Read postconditions              → "What must be true when I'm done?"
8. Read evidence_requirements       → "What proof must I attach?"
9. Read available_tools             → "What can I use?"
```

The agent receives all nine as structured data. Its goal construction becomes deterministic:

> "I'm at transition `send-deficiency-notice`. The token tells me the applicant is Juan Pérez, contact channel is WhatsApp, missing documents are [cadastral plan]. Policy says be specific about what's missing. I need to produce a token with `{notificationId, channel, timestamp, messageContent}`. I need evidence: the message ID from the WhatsApp API. I have access to the `send-whatsapp` tool. Go."

### Evidence production

After execution, the agent must provide:
1. **Output payload** — data for the output token, conforming to `output_schema` if present
2. **Postcondition claims** — which postconditions it believes it satisfied, verified by deterministic checkers or LLM-as-judge
3. **Evidence artifacts** — proof for each evidence requirement, attached to the audit entry

The engine validates all three before updating the marking.

---

## Persistence

SQLite. One database per institution. Tables mirror the type hierarchy:

- `institutions`, `roles`, `actors`, `actor_roles`
- `nets`, `places`, `transitions`, `policies`
- `instances`, `tokens`, `audit_entries`

The audit log table has `prev_hash` / `entry_hash` columns for tamper evidence, same as the JSONL approach but queryable.

Markings are derived from the `tokens` table (current tokens for an instance, grouped by place). This makes queries like "all instances with a token in `scarcity-hold`" trivial.

---

## Interface Layers

### Library (core)

All operations above as exported async functions. Audit logging happens inside the library — callers can't bypass it. The library owns the SQLite connection.

### CLI

Thin wrapper. One command per operation:

```bash
inst institution create --name "ASADA Playas de Nosara"
inst role create --institution asada --name "administrator" --authority 2
inst net create --institution asada --name "Carta de Agua" --domain carta-de-agua
inst place add --net carta-de-agua --id intake --description "Request received"
inst transition add --net carta-de-agua --id completeness-check --consumes intake --produces documents-complete --mode agentic --authority 2
inst policy attach --institution asada --scope "carta-de-agua.completeness-check" --strength procedure --text "..."
inst instance create --net carta-de-agua --payload '{"applicant": "Juan Pérez", ...}'
inst fire --instance <id> --transition completeness-check --actor don-carlos
inst pending --instance <id>
inst history --instance <id>
```

All commands support `--format json`.

### HTTP API

Same operations as REST endpoints. Enables a future UI and external system integration. Not in scope for first milestone but the library design supports it without changes.

---

## Execution Modes

### Deterministic

Fully automated. The engine fires the transition by calling tools directly. No LLM involved. Used for mechanical steps: assign case ID, send receipt, look up subscriber payment status.

### Agentic

LLM-driven. The agent receives the assembled work order, reasons about how to accomplish the intent, calls tools, produces output and evidence. Used for telemetry tasks: compose and send deficiency notice, compile board packet, generate inspection report.

### Judgment

Requires human resolution. The engine identifies the transition as enabled, emits a `PendingJudgment`, and suspends. A human reviews the assembled context (policies, precedent, case data) and provides a decision via `resolveJudgment`. The agent can prepare context and recommendations, but the human decides. Used for: board approval, scarcity hold decisions, appeal rulings.

---

## First Target: Carta de Agua

The ASADA Playas de Nosara "Carta de Agua" process encoded as a CPN:

### Places

| Place | Description |
|-------|-------------|
| `intake` | Request received, case ID assigned |
| `documents-pending` | Awaiting missing documents from applicant |
| `documents-complete` | All required documents received |
| `triaged` | Case classified by impact level |
| `scarcity-hold` | Case held due to source stress |
| `technical-review-ready` | Ready for technical assessment |
| `inspection-complete` | Field inspection done (if required) |
| `board-ready` | Board packet assembled, awaiting board meeting |
| `decided` | Board has issued decision |
| `delivered` | Decision letter delivered to applicant |
| `appeal-submitted` | Applicant has contested the decision |
| `appeal-decided` | Board has ruled on appeal |

### Transitions

| Transition | Mode | Authority | Intent |
|-----------|------|-----------|--------|
| `receive-request` | deterministic | 2 | Assign case ID, timestamp, send receipt |
| `check-completeness` | agentic | 2 | Review submitted docs against checklist |
| `send-deficiency-notice` | agentic | 2 | Notify applicant of specific missing items |
| `receive-documents` | deterministic | 2 | Log document submission, re-check completeness |
| `triage-case` | judgment | 2 | Classify: residential / commercial / high-impact |
| `check-scarcity` | deterministic | 2 | Check source flow against threshold |
| `hold-for-scarcity` | agentic | 2 | Notify applicant of hold with explanation |
| `determine-inspection` | judgment | 2 | Decide whether field inspection is needed |
| `conduct-inspection` | agentic | 2 | Schedule and document field visit |
| `compile-board-packet` | agentic | 2 | Assemble request, evidence, technical report, recommendation |
| `board-decision` | judgment | 4 | Approve, deny, conditional, or defer |
| `generate-decision-letter` | agentic | 2 | Draft letter with decision, reasoning, conditions, appeal instructions |
| `deliver-decision` | agentic | 2 | Send letter via applicant's preferred channel |
| `receive-appeal` | deterministic | 2 | Log appeal, issue receipt |
| `board-appeal-review` | judgment | 4 | Review appeal and issue ruling |

### Roles

| Role | Authority |
|------|-----------|
| `administrator` | 2 |
| `technical-operator` | 2 |
| `junta-directiva` | 4 |
| `president` | 3 |
| `secretary` | 3 |

### Key policies

- **Constraint:** Source flow must be ≥ 4 L/s to proceed (scope: `carta-de-agua.check-scarcity`)
- **Constraint:** Board decision required for all approvals/denials (scope: `carta-de-agua.board-decision`)
- **Procedure:** Decision letter must include basis, conditions, and appeal instructions (scope: `carta-de-agua.generate-decision-letter`)
- **Preference:** Be specific in deficiency notices (scope: `carta-de-agua.send-deficiency-notice`)
- **Preference:** High-impact projects should require inspection (scope: `carta-de-agua.determine-inspection`)
- **Context:** Development pressure from tourism — obligation is to existing residents (scope: `carta-de-agua.*`)

---

## Deferred

- Workflow versioning and snapshots
- Integration registry and edge compilation to external platforms
- Multi-net interaction (tokens/decisions in one workflow affecting another)
- Agentic encoding (agent helps build the net from interview transcripts)
- RAG for large policy sets
- Concurrent transition firing (maximal concurrent step mode)
- WASM/napi-rs optimization if needed
