# Workflow Execution Specification

## Overview

Workflows are modeled as **Coloured Petri Nets (CPNs)**: a formal computational model where state is represented as tokens flowing through a graph of places and transitions. Unlike sequential workflow engines, this model natively represents concurrency, non-determinism, and data-parameterized branching without duplicating structure.

The core engine (`src/core/engine.ts`) implements this specification with SQLite persistence, authority enforcement, and cryptographic audit logging.

---

## Core Abstractions

### Places

A **place** represents a state or condition in the workflow. Each place has:

- `id` — unique identifier within the net (e.g., `"intake"`, `"board-ready"`)
- `description` — human-readable description of what this state means
- `schema` — optional JSON schema constraining token payloads

Places are defined via `Engine.addPlace(netId, id, description, schema?)`.

### Transitions

A **transition** represents an action that moves the workflow forward. Each transition declares:

| Field | Type | Purpose |
|-------|------|---------|
| `id` | string | Unique identifier |
| `consumes` | string[] | Input place IDs — tokens removed |
| `produces` | string[] | Output place IDs — tokens added |
| `guard` | string? | Expression evaluated against binding (future) |
| `intent` | string | Natural language description of what this accomplishes |
| `mode` | `"deterministic" \| "judgment" \| "agentic"` | Execution mode |
| `decision_type` | DecisionType? | For judgment transitions |
| `requires_authority` | number | Minimum authority level to fire |
| `authorized_roles` | string[]? | Optional role whitelist |
| `input_schema` | JsonSchema? | Expected input structure |
| `output_schema` | JsonSchema? | Required output structure |
| `context_sources` | string[] | Keys for context assembly |
| `postconditions` | Postconditions | Required/desired/escalation outcomes |
| `evidence_requirements` | EvidenceRequirement[] | Proof that must be captured |
| `available_tools` | string[] | Tools the agent may use |
| `timeout` | number? | Execution timeout in seconds |

Transitions are defined via `Engine.addTransition(netId, transitionDef)`, which also creates the corresponding arcs.

### Arcs

**Arcs** connect places to transitions (input arcs) and transitions to places (output arcs). They are created automatically when a transition is defined based on its `consumes` and `produces` arrays.

- `place_to_transition` — input arc: tokens consumed from this place
- `transition_to_place` — output arc: tokens produced into this place

### Tokens

A **token** represents a unit of case data at a specific place. Each token carries:

- `id` — unique identifier
- `instance_id` — which workflow instance this token belongs to
- `place_id` — which place the token is in
- `payload` — `Record<string, unknown>` — arbitrary structured data
- `created_at` — timestamp

Tokens are coloured: they carry payloads that represent the evolving case data as it flows through the net.

### Marking

A **marking** is a snapshot of the full workflow state: a map from place IDs to their current tokens.

```
Marking = Map<PlaceId, Token[]>
```

A workflow instance begins with a single token in a designated start place (created by `Engine.instantiate()`) and progresses as transitions fire, consuming tokens from input places and producing tokens in output places.

Retrieved via `Engine.getMarking(instanceId)`.

---

## Firing Semantics

### Enablement

A transition `T` is **enabled** for actor `A` in instance `I` when:

1. For every place in `T.consumes`, the current marking of `I` has at least one token in that place
2. `A` has authority level ≥ `T.requires_authority` (derived from the maximum authority across all of `A`'s assigned roles)

Enabled transitions for an actor are found via `Engine.getEnabledTransitions(instanceId, actorId)`.

### Firing Protocol

When `Engine.fireTransition(instanceId, transitionId, actorId, outputPayload)` is called:

1. **Authority check** — verify actor's authority ≥ transition's `requires_authority`. If insufficient, return `FiringResult` with `success: false`.

2. **Token check** — verify all input places have at least one token. If any input place is empty, return `FiringResult` with `success: false`.

3. **Atomic execution** (within a SQLite transaction):
   a. Snapshot marking before
   b. **Consume** one token from each input place (`DELETE FROM tokens`)
   c. **Produce** one token in each output place (`INSERT INTO tokens`) with the provided `outputPayload`
   d. Snapshot marking after
   e. **Audit** — append a hash-chained audit entry recording the transition, actor, markings, evidence, and reasoning

4. **Return** `FiringResult` with consumed tokens, produced tokens, and audit entry ID.

Both consumption and production are **atomic** — no intermediate state is observable. This is enforced by SQLite transaction wrapping.

### Judgment Points

A **judgment point** is a transition with `mode: "judgment"`. These transitions represent decisions requiring human discretion or policy evaluation.

When a judgment point becomes enabled:

- It appears in `Engine.getPendingJudgments(instanceId)`, which returns the transition context, required authority, token payloads, and applicable policies
- It is **not** automatically fired — it awaits explicit resolution
- Resolution happens via `Engine.resolveJudgment(instanceId, transitionId, actorId, decision, reasoning?, evidence?)`, which delegates to `fireTransition` with authority enforcement

### Execution Modes

| Mode | Meaning | Current behavior |
|------|---------|-----------------|
| `deterministic` | Mechanical step, no judgment | Fires via `fireTransition` with output payload |
| `judgment` | Requires human/institutional discretion | Appears in `getPendingJudgments`, resolved via `resolveJudgment` |
| `agentic` | Agent executes with tools and LLM | Currently fires via `fireTransition`; future: agent runtime consumes `WorkOrder` |

---

## Authority Model

Authority is numeric and role-based:

- **Roles** have an `authority_level` (integer)
- **Actors** are assigned to one or more roles
- An actor's effective authority is `max(authority_level)` across all assigned roles
- A transition's `requires_authority` is the minimum authority needed to fire it

This is strictly enforced: `fireTransition` and `resolveJudgment` both check authority before any state mutation.

---

## Policy Resolution

Policies are scoped to domains and transitions via dot-separated scope strings:

- `carta-de-agua.board-decision` — specific to one transition
- `carta-de-agua.*` — applies to all transitions in the domain
- `*` — global

`Engine.getPolicies(scope)` resolves policies by:
1. Matching exact scope, parent wildcards, and global
2. Sorting by strength (constraint first) then specificity (exact match first)

Policies are included in `WorkOrder` objects for agent context and in `PendingJudgment` objects for human decision support.

---

## Audit Trail

Every transition firing produces an `AuditEntry` with:

- Instance and transition context
- Actor identity (actor_id, role_id, authority_level)
- Marking snapshots (before and after)
- Evidence array (requirement_id, type, content, timestamp)
- Reasoning text
- Cryptographic hash chain (`prev_hash` → SHA-256 → `entry_hash`)

Chain integrity is verifiable: `AuditLog.verifyChain(instanceId)` recomputes all hashes and checks linkage.

---

## State Representation

```typescript
WorkflowInstance {
  id:         string          // UUID
  net_id:     string          // which net definition
  status:     InstanceStatus  // "running" | "completed" | "stuck" | "suspended"
  created_at: string          // ISO-8601
  updated_at: string          // ISO-8601
}
```

Runtime state is derived from the token table:
- **Marking** = `SELECT * FROM tokens WHERE instance_id = ?`
- **History** = `SELECT * FROM audit_entries WHERE instance_id = ? ORDER BY sequence`

The audit log is an append-only record of all firing events. Full marking history is reconstructible by replaying from the initial marking.

---

## Validation

`validateNet(engine, netId)` checks structural properties:

- **Orphan places** — places with no arcs (warning)
- **Judgment transitions without policies** — judgment mode but no policies in scope (warning)
- **Invalid place references** — transitions referencing non-existent places (error)
- **Sourceless transitions** — transitions with no input places (warning)

Validation returns `{ violations, is_valid }` where `is_valid` is false only if there are error-severity violations.

---

## Future Extensions

- **Guard evaluation** — guards stored as strings, need an expression language and evaluator
- **Instance lifecycle** — automatic detection of terminal markings, status transitions
- **Concurrent firing** — maximal concurrent step semantics for parallel institutional processes
- **Postcondition verification** — deterministic checkers and LLM-as-judge after agent execution
- **Net composition** — hierarchical nets where a transition can expand into a sub-net
- **Binding search** — formal CPN binding enumeration for transitions with multiple input arcs
- **Timed transitions** — timeout-based escalation when transitions remain unfired
