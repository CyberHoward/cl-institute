# Intelligent Institution Initiative — Technical Architecture Blueprint

## Executive Summary

The Intelligent Institution Initiative is a system that makes institutions programmable by representing their structure, workflows, and decision-making processes as code. Rather than competing with existing workflow automation tools, the system targets the **judgment layer** — the points in institutional workflows where decisions require context, policy, precedent, and discretion.

The architecture is a unified TypeScript system built around a **Coloured Petri Net engine with institutional semantics**. The core engine owns the workflow model, token-based execution, authority enforcement, policy resolution, and cryptographic audit logging — all persisted in SQLite. An intelligence layer (future) will manage AI-driven agent execution at transitions, consuming structured work orders assembled from the engine's state.

---

## 1. Core Thesis

Institutions can be represented as formal systems expressed in code. This representation inherits the properties of code itself: versioning, diffability, composability, upgradability, and auditability. These are not features to be built — they emerge from the representation.

The system does not model every step in an institutional workflow. Deterministic, mechanical steps are a solved problem addressed by existing automation tools. Instead, the system models the **decision topology** — where institutional judgment is required, what governs it, and how outcomes are recorded.

Natural language is the most effective medium for expressing institutional knowledge. Large language models serve as the bridge between natural language policy and formal execution, enabling a stratified representation where each layer uses the formalism appropriate to its nature.

---

## 2. Foundational Concepts

### 2.1 The Formality Spectrum

Institutional rules exist on a spectrum of formality. The system represents them as **policy strengths** attached to scoped policy objects:

| Strength | Nature | Enforcement |
|----------|--------|-------------|
| **Constraint** | Hard rules, legal/regulatory mandates | Machine-enforced at runtime |
| **Procedure** | Defined steps with authorized deviation | Included in work orders, logged overrides |
| **Preference** | Intent-bearing guidance | Included in agent context, advisory |
| **Context** | Tacit knowledge, institutional culture | Included in agent context, advisory |

Policies are scoped hierarchically (e.g., `carta-de-agua.board-decision` → `carta-de-agua.*` → `*`) and resolved at query time ordered by strength then specificity.

### 2.2 The Decision Point Ontology

A **judgment point** is a transition with `mode: "judgment"`. Its components:

- **Decision type** — approval, classification, prioritization, allocation, exception handling
- **Inputs** — token payloads from consumed places, plus resolved context sources
- **Governing policies** — scoped by domain and transition ID, drawn from the policy store
- **Authority model** — numeric `requires_authority` level, checked against actor role assignments
- **Evidence requirements** — what proof must be attached (artifact, reference, attestation)
- **Postconditions** — required, desired, and escalation outcomes
- **Output schema** — structured data the decision must produce
- **Accountability trail** — cryptographically chained audit entries with full context

### 2.3 Workflow as Coloured Petri Net

Workflows are modeled as **Coloured Petri Nets** (CPNs). Places represent states, transitions represent actions, and coloured tokens carry structured payloads through the net. This gives the system precise semantics for concurrency, synchronization, and data flow.

Each transition declares:
- **consumes/produces** — which places it reads from and writes to
- **intent** — natural language description of what the transition accomplishes
- **mode** — `deterministic`, `judgment`, or `agentic`
- **requires_authority** — minimum authority level to fire
- **context_sources** — keys for assembling the agent's working context
- **postconditions** — required and desired outcomes
- **evidence_requirements** — proof that must be captured
- **available_tools** — which tools the agent may use during execution

---

## 3. Architecture

### 3.1 Overview

```
┌─────────────────────────────────────────────────────────┐
│                    TypeScript System                     │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │           Intelligence Layer (future)              │  │
│  │                                                   │  │
│  │  ┌──────────────────────────────────────────────┐ │  │
│  │  │           Agent Runtime                       │ │  │
│  │  │  Consumes WorkOrders from core/context.ts     │ │  │
│  │  │  Fires transitions via Engine.fireTransition  │ │  │
│  │  └──────────────────┬───────────────────────────┘ │  │
│  └─────────────────────┼─────────────────────────────┘  │
│                        │                                │
│  ┌─────────────────────┼─────────────────────────────┐  │
│  │              Core Layer (implemented)              │  │
│  │                                                   │  │
│  │  ┌──────────────────┴──────────────────────────┐  │  │
│  │  │              Engine                          │  │  │
│  │  │  Institution / Role / Actor / Policy CRUD    │  │  │
│  │  │  Net definition (Place, Transition, Arc)     │  │  │
│  │  │  Runtime (instantiate, fire, marking query)  │  │  │
│  │  │  Judgment points (pending, resolve)          │  │  │
│  │  │  Authority enforcement                       │  │  │
│  │  └──┬──────────┬──────────┬──────────┬─────────┘  │  │
│  │     │          │          │          │            │  │
│  │  ┌──┴───┐  ┌──┴───┐  ┌──┴────┐  ┌──┴─────────┐  │  │
│  │  │SQLite│  │Audit │  │Context│  │ Validate   │  │  │
│  │  │  DB  │  │  Log │  │Assembly│ │            │  │  │
│  │  │(WAL) │  │(hash │  │(work  │  │(structural │  │  │
│  │  │      │  │chain)│  │orders)│  │ checks)    │  │  │
│  │  └──────┘  └──────┘  └───────┘  └────────────┘  │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Core Layer — `src/core/`

The core layer is fully implemented. All components are TypeScript with strict mode, SQLite persistence via `better-sqlite3`, and Zod for schema definitions.

#### 3.2.1 Engine (`engine.ts`)

The Engine class is the primary API. It provides ~15 operations:

**Definition operations:**
- `createInstitution`, `createRole`, `createActor`, `assignRole` — institutional setup
- `createNet`, `addPlace`, `addTransition` — CPN definition
- `attachPolicy` — scope-based policy attachment

**Runtime operations:**
- `instantiate` — create a workflow instance with initial token
- `fireTransition` — authority-checked, transactional token consumption/production with audit
- `resolveJudgment` — fire a judgment transition with decision payload and evidence

**Query operations:**
- `getMarking` — current token distribution for an instance
- `getEnabledTransitions` — transitions an actor can fire given current marking
- `getPendingJudgments` — judgment transitions awaiting resolution
- `getPolicies` — hierarchical scope resolution with strength ordering
- `getHistory` — audit trail for an instance

All mutating operations that touch multiple tables are wrapped in SQLite transactions for atomicity.

#### 3.2.2 Database (`db.ts`)

SQLite with WAL mode and foreign key enforcement. 12 tables covering:
- Institutional structure (institutions, roles, actors, actor_roles)
- Net definition (nets, places, transitions, arcs)
- Governance (policies)
- Runtime state (instances, tokens)
- Audit (audit_entries)

7 indexes for common query patterns.

#### 3.2.3 Audit Log (`audit.ts`)

Append-only, hash-chained audit entries. Each entry includes:
- Instance and transition context
- Actor identity and authority level
- Marking snapshots (before/after)
- Evidence array
- Reasoning text
- Cryptographic chain: `prev_hash` → `entry_hash` (SHA-256)

Chain integrity is verifiable via `AuditLog.verifyChain()`.

#### 3.2.4 Context Assembly (`context.ts`)

The `buildWorkOrder()` function assembles a 9-step structured context for agent execution:

1. **Intent** — what am I trying to accomplish?
2. **Token payloads** — what case data do I have?
3. **Input schema** — do I have everything I need?
4. **Policies** — what rules govern this? (resolved by scope, ordered by strength)
5. **Context sources** — what else should I consider?
6. **Output schema** — what data must I produce?
7. **Postconditions** — what must be true when I'm done?
8. **Evidence requirements** — what proof must I attach?
9. **Available tools** — what can I use?

This is the contract between the core engine and the future agent runtime.

#### 3.2.5 Validation (`validate.ts`)

Structural validation of net definitions:
- Orphan place detection (places with no arcs)
- Judgment transitions without governing policies
- Invalid place references
- Sourceless transitions

### 3.3 Intelligence Layer (future)

The intelligence layer will consume the core layer's `WorkOrder` interface and fire transitions through `Engine.fireTransition()`.

**Agent runtime** — will observe pending transitions, assemble work orders, invoke LLMs with tools, verify postconditions, and fire transitions with evidence. The agent is a role with explicit authority, same as any human actor.

**LLM orchestration** — prompt construction from work orders, tool execution, response parsing.

**Postcondition verification** — deterministic checkers where possible, LLM-as-judge fallback for semantic postconditions.

---

## 4. Formality Spectrum (unchanged)

The four-layer formality spectrum maps directly to the `PolicyStrength` type:

| PolicyStrength | Layer | Representation | Enforcement |
|----------------|-------|----------------|-------------|
| `"constraint"` | Hard rules | Structured text, scoped to transitions | Engine can enforce; included first in work orders |
| `"procedure"` | Defined steps | Structured text | Included in work orders |
| `"preference"` | Guidance | Natural language | Included in agent context |
| `"context"` | Tacit knowledge | Natural language | Advisory, included last |

---

## 5. Progressive Adoption Path

**Stage 1 — Codification (current):** Define the institutional model: institutions, roles, authority levels, workflow nets, policies. Validate net structure. The value is clarity and formal structure.

**Stage 2 — Decision support:** Add the intelligence layer. The LLM assists humans at judgment points by consuming work orders, surfacing relevant policies and precedent, and structuring the decision. Decisions are still made by humans via `resolveJudgment()`.

**Stage 3 — Partial automation:** Agentic transitions run via LLM agents with tool access. Deterministic transitions run automatically. Judgment points still require human authority.

**Stage 4 — Delegated agency:** AI agents operate with institutional authority for defined transition types. Humans oversee and handle exceptions. All agent actions produce the same audit trail as human actions.

---

## 6. Key Technical Decisions

### Decided

- **Unified TypeScript architecture** — single language, strict mode, ESM
- **Coloured Petri Net execution model** — places, transitions, arcs, coloured tokens with payloads
- **SQLite persistence** — WAL mode, transactional writes, foreign key enforcement
- **Numeric authority levels** — flexible, composable authority model via role assignments
- **Hierarchical policy scoping** — `domain.transition` → `domain.*` → `*` with strength ordering
- **Cryptographic audit chaining** — SHA-256 hash chains for tamper evidence
- **Transactional firing** — token consumption, production, and audit are atomic
- **Work order contract** — 9-step structured context as the interface between engine and agents
- **pi-agent-core / pi-ai for future agent execution** — agent runtime uses pi libraries

### Open

- **Instance lifecycle:** When does an instance transition to `completed`, `stuck`, or `suspended`? Currently instances remain `running`.
- **Postcondition evaluation:** How are postconditions verified after agent execution? Deterministic checkers vs. LLM-as-judge.
- **Guard evaluation:** Guards are stored as strings but not yet evaluated. What is the guard expression language?
- **Concurrent firing:** Current model is sequential. When do we need concurrent transition firing?
- **Token merging:** When multiple tokens flow into a place, should payloads merge or remain separate?
- **Net composition:** How do sub-nets compose? Can a transition expand into a sub-net?

---

## 7. Package Structure

```
typescript/
├── package.json
├── tsconfig.json
├── vitest.config.ts
├── src/
│   ├── index.ts                   # Package entry — re-exports from core/
│   └── core/
│       ├── index.ts               # Barrel export
│       ├── types.ts               # Canonical types: CPN + institutional model
│       ├── db.ts                  # SQLite schema (12 tables) and connection
│       ├── engine.ts              # Engine class: ~15 operations
│       ├── audit.ts               # Hash-chained audit log
│       ├── context.ts             # Work order assembly (buildWorkOrder)
│       ├── validate.ts            # Net structural validation
│       ├── types.test.ts          # Type construction tests
│       ├── db.test.ts             # Schema creation tests
│       ├── engine.test.ts         # 27 tests: definition, runtime, firing, judgments
│       ├── audit.test.ts          # Hash chaining and verification tests
│       ├── context.test.ts        # Work order assembly tests
│       ├── validate.test.ts       # Validation rule tests
│       └── carta-de-agua.test.ts  # End-to-end: ASADA Carta de Agua process
```

### Archived code

```
archive/
├── iii-spike/    # Original Petri net spike (vendor onboarding scenario)
└── rust/         # Abandoned Rust implementation
```

---

## 8. Milestones

1. **✓ Spike: Petri net agent execution boundary.** Validated that an LLM agent can interpret transition definitions and execute them with tools. Proved the core execution model. (archived in `archive/iii-spike/`)

2. **✓ Core engine with institutional semantics.** Unified CPN + institutional model. SQLite persistence. Authority enforcement. Policy scoping. Cryptographic audit. Transactional firing. 43 tests passing.

3. **✓ First real workflow encoded.** The ASADA Carta de Agua (water availability letter) process: 9 places, 7 transitions, 7 policies, full end-to-end test with authority gating and evidence capture.

4. **✓ Agent runtime (phase 1).** AgentRunner with step/run loop, postcondition verification (deterministic + LLM fallback with confidence tracking), institutional context store, prompt construction. Mock executor for testing; real LLM integration deferred to phase 2.

5. **Next: Postcondition verification.** Deterministic verifiers mapped to postcondition strings, with LLM-as-judge fallback.

6. **Next: Instance lifecycle.** Detect terminal markings, update instance status, handle stuck/suspended states.

Each milestone validates the architecture at a different boundary. Prioritize the feedback loop over completeness.
