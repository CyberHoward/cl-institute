# Intelligent Institution Initiative — Technical Architecture Blueprint

## Executive Summary

The Intelligent Institution Initiative is a system that makes institutions programmable by representing their structure, workflows, and decision-making processes as code. Rather than competing with existing workflow automation tools, the system targets the **judgment layer** — the points in institutional workflows where decisions require context, policy, precedent, and discretion.

The architecture is a unified TypeScript system. A core model layer owns the institutional representation, invariants, and Petri net execution engine. An intelligence layer manages AI-driven policy interpretation, integration compilation, and agent operation. The system is file-based and git-managed, ensuring reproducibility, auditability, and extensibility.

---

## 1. Core Thesis

Institutions can be represented as formal systems expressed in code. This representation inherits the properties of code itself: versioning, diffability, composability, upgradability, and auditability. These are not features to be built — they emerge from the representation.

The system does not model every step in an institutional workflow. Deterministic, mechanical steps are a solved problem addressed by existing automation tools (n8n, Zapier, RPA platforms, agentic tooling). Instead, the system models the **decision topology** — where institutional judgment is required, what governs it, and how outcomes are recorded.

Natural language is the most effective medium for expressing institutional knowledge. Large language models serve as the bridge between natural language policy and formal execution, enabling a stratified representation where each layer uses the formalism appropriate to its nature.

---

## 2. Foundational Concepts

### 2.1 The Formality Spectrum

Institutional rules exist on a spectrum of formality. The system represents them in four stratified layers:

| Layer | Nature | Representation | Enforcement |
|-------|--------|----------------|-------------|
| **Constraints** | Hard rules, legal/regulatory mandates | Typed predicates, formal logic | Machine-enforced, compile-time and runtime |
| **Procedures** | Defined steps with authorized deviation | State machines, decision graphs | Executed deterministically with logged overrides |
| **Policies** | Intent-bearing guidance, preferences | Structured natural language with semantic metadata | LLM-interpreted at decision time |
| **Context** | Tacit knowledge, institutional culture | Unstructured natural language | LLM-referenced, advisory only |

Each layer uses the formalism appropriate to its nature. The LLM serves as the interpreter that bridges between layers.

### 2.2 The Decision Point Ontology

A **judgment point** (decision node) is the atomic unit of the system. Its components:

- **Decision type** — approval, classification, prioritization, allocation, exception handling
- **Inputs** — documents, data, prior decisions, contextual information required to decide
- **Governing policies** — scoped by domain and decision type, drawn from Layer 3
- **Authority model** — who can make this decision, delegation rules, escalation paths
- **Precedent** — how this type of decision has been made before (historical decision records)
- **Output schema** — what the decision produces (boolean, ranking, modified document, routing choice)
- **Accountability trail** — who decided, when, based on what, with what reasoning

### 2.3 Workflow as Decision Topology

A workflow is not modeled as a linear sequence of steps. It is modeled as a **graph of judgment points** connected by **edge specifications**. The edges describe *what* needs to happen between decisions (intent-level), not *how* (implementation-level). Implementation is delegated to external automation tools via the integration layer.

The execution model uses **Petri nets** as the formal foundation. Places represent states (conditions that hold), transitions represent actions (including agent-executed judgment), and tokens carry context through the net. This gives the system precise semantics for concurrency, synchronization, and state that informal workflow graphs lack.

### 2.4 Telescoping Abstraction

Workflows are defined at multiple resolutions simultaneously. A high-level business process decomposes into sub-workflows, which decompose into individual tasks. The level at which something is "atomic" versus "composite" is not fixed — it depends on the observer and the operational context. This mirrors function composition in code: abstraction layers with stable external interfaces and expandable internals.

---

## 3. Architecture

### 3.1 Overview

```
┌─────────────────────────────────────────────────────────┐
│                    TypeScript System                     │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │              Intelligence Layer                    │  │
│  │                                                   │  │
│  │  ┌─────────────┐ ┌──────────────┐ ┌───────────┐  │  │
│  │  │     LLM     │ │   Policy     │ │Integration│  │  │
│  │  │Orchestration│ │ Interpreter  │ │ Compiler  │  │  │
│  │  └──────┬──────┘ └──────┬───────┘ └─────┬─────┘  │  │
│  │         │               │               │        │  │
│  │  ┌──────┴───────────────┴───────────────┴─────┐  │  │
│  │  │              Agent Runtime                  │  │  │
│  │  └──────────────────┬──────────────────────────┘  │  │
│  └─────────────────────┼─────────────────────────────┘  │
│                        │                                │
│  ┌─────────────────────┼─────────────────────────────┐  │
│  │                     │       Core Layer             │  │
│  │  ┌──────────────────┴──────────────────────────┐  │  │
│  │  │           Model Access Layer                 │  │  │
│  │  └──┬──────────┬──────────┬──────────┬─────────┘  │  │
│  │     │          │          │          │            │  │
│  │  ┌──┴────┐ ┌──┴───┐  ┌──┴───┐  ┌──┴─────────┐  │  │
│  │  │Petri  │ │Const-│  │Audit │  │Integration │  │  │
│  │  │ Net   │ │raint │  │  Log │  │  Registry  │  │  │
│  │  │Engine │ │Engine│  │      │  │            │  │  │
│  │  └───────┘ └──────┘  └──────┘  └────────────┘  │  │
│  │                                                  │  │
│  │  ┌────────────────────────────────────────────┐  │  │
│  │  │     Institution Project Directory           │  │  │
│  │  │  (git-managed, file-based, human-readable)  │  │  │
│  │  └────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Core Layer

The core layer owns the institutional model, execution engine, invariants, and persistence. All components are implemented in TypeScript with Zod schemas for runtime validation.

#### 3.2.1 Petri Net Execution Engine

The execution engine implements a Petri net semantics for workflow execution. Places represent institutional states, transitions represent actions, and tokens carry context (payloads) through the net.

Each transition declares:
- **Consumes/produces** — which places it reads from and writes to (formal net semantics)
- **Intent** — natural language description of what the transition accomplishes
- **Context sources** — keys to look up in the context store for assembling the agent's working context
- **Postconditions** — required and desired outcomes, verified after execution
- **Available tools** — which tools the agent may use during execution
- **Execution mode** — `deterministic`, `judgment`, or `agentic`

The engine finds enabled transitions (all input places have tokens), hands them to the agent executor with the appropriate tools and context, verifies postconditions, and updates the marking.

**Design guidance:**
- Use Zod schemas (`NetSchema`, `TransitionSchema`, `PlaceSchema`) for runtime validation of net definitions.
- The context store (`ContextStore`) is a simple key-value map that accumulates data as tokens flow through the net. Transitions read from it via `context_sources` and write to it via execution payloads.
- Postcondition verification uses deterministic checkers where possible, with LLM-as-judge fallback for semantic postconditions.

#### 3.2.2 Constraint Engine

Evaluates Layer 1 (hard constraints) at both definition time and runtime. When a workflow is defined or modified, the constraint engine validates that all invariants hold: authority levels are sufficient, required inputs are available, policy scoping is valid. At runtime, it enforces that decisions are made by authorized actors and that edge preconditions are satisfied.

**Design guidance:**
- Constraints are typed predicates attached to the graph. Model them as functions so new constraint types are extensible.
- Validation produces structured error types with full context, enabling the intelligence layer to present meaningful guidance to the LLM or human operator.

#### 3.2.3 Audit Log

Append-only, structured log of every mutation to the institutional model and every decision made at runtime. Each entry captures: the action, the actor, the timestamp, the prior state, the new state, and the reasoning (if provided).

**Design guidance:**
- JSONL format for the log file — one JSON object per line, streamable and parseable.
- Cryptographic chaining (hash of previous entry included in each new entry) for tamper evidence.
- Every transition firing, postcondition verification, and agent action is logged with full context.

#### 3.2.4 Integration Registry

A typed declaration of what external capabilities the institution has available. Each integration exposes a set of capabilities (e.g., DocuSign exposes `route_for_signature`, SAP exposes `create_purchase_order`) with defined input/output schemas.

**Design guidance:**
- The registry lives in the institution project directory alongside the graph — it is part of the institutional definition.
- The registry does not contain credentials or connection details. Those are environment configuration, separate from the institutional model.
- Tools are registered with the engine at startup and filtered per-transition based on `available_tools`.

#### 3.2.5 Model Access Layer

The access layer provides typed operations for reading and mutating the institutional model. All other modules consume data through this layer's typed interfaces.

**Design guidance:**
- The access layer is stateless — configuration only. Each call reads from or writes to the project directory.
- All operations support JSON output for machine consumption.
- The access layer is the contract between the core and the intelligence layer.

### 3.3 Intelligence Layer

The intelligence layer manages AI-driven reasoning. It consumes the core layer's typed interfaces and adds LLM-powered intelligence.

#### 3.3.1 LLM Orchestration

Manages the dialogue when an AI agent is programming an institution or supporting human decision-making. Constructs prompts, parses structured outputs, manages conversation state, and translates LLM intent into sequences of model operations.

**Design guidance:**
- Use the pi-agent-core and pi-ai libraries for agent execution and model access.
- Define a prompt template system for different interaction modes: institution definition, decision support, policy interpretation, edge specification authoring.
- Every LLM interaction that results in a mutation to the institutional model should produce a reviewable operation sequence before execution.

#### 3.3.2 Policy Interpreter

When a decision point is reached and policies need to inform the judgment, this module:

1. Queries the core for relevant policies by scope.
2. Retrieves precedent from the audit log.
3. Assembles the decision context (inputs, policies, precedent, authority model) into a structured prompt.
4. Invokes the LLM to reason about what the policies imply for this specific case.
5. Returns a structured recommendation or decision, which is recorded back through the model.

**Design guidance:**
- Policy scoping (attaching policies to ontological structures rather than relying on retrieval) is the primary mechanism. Policies are *in context* because of their scope, not because of similarity search.
- For institutions with large policy sets, a secondary RAG layer may be needed, but start with scoping and add retrieval only when scoping proves insufficient.
- Policy interpretation results should include reasoning traces for the audit log.

#### 3.3.3 Integration Compiler

Takes an edge specification plus the integration registry and produces an executable automation for a target platform.

**Compilation flow:**
1. Read edge specification (natural language intent + structured metadata).
2. Read available integrations from the registry.
3. Construct a prompt: "Given these available capabilities, produce an executable plan for this edge specification."
4. LLM generates a platform-specific automation (n8n workflow JSON, API call sequence, human checklist).
5. Output is validated against the integration registry (all referenced capabilities must exist).
6. Compiled edge is stored alongside the institution definition.

**Design guidance:**
- Define compilation targets as pluggable modules. Start with one target (e.g., n8n or direct API calls) and add others incrementally.
- The compiled output is an artifact, not the source of truth. The edge specification is the source of truth. Recompilation should be idempotent given the same specification and registry.
- For edges where no automation exists, compile to a structured human checklist. This is the default and should be high quality.

#### 3.3.4 Agent Runtime

The autonomous agent that can operate the institution. It:

1. Reads the decision graph to identify pending decisions (enabled transitions in the Petri net).
2. Assembles context for each pending decision (inputs, policies, precedent) via the context store.
3. Invokes the policy interpreter.
4. Makes or recommends decisions (depending on delegation level and execution mode).
5. Records outcomes through the audit log.
6. Triggers edge execution when decisions are made and postconditions are verified.

**Design guidance:**
- The agent's authority is defined within the institutional model itself — it's a role with explicit permissions and constraints, same as any human actor.
- All agent actions produce the same audit trail as human actions.
- The agent should be interruptible and resumable. Its state is derived from the Petri net marking, not held internally.
- Execution modes (`deterministic`, `judgment`, `agentic`) control how much autonomy the agent has at each transition.

---

## 4. Institution as Code — Project Structure

An institution is a directory that lives in version control:

```
acme-foundation/
├── institution.toml              # Top-level metadata, name, version
├── roles/
│   ├── compliance-officer.toml
│   ├── procurement-lead.toml
│   └── board-member.toml
├── workflows/
│   └── procurement/
│       ├── workflow.toml          # Decision graph definition (Petri net)
│       ├── decisions/
│       │   ├── vendor-review.toml
│       │   └── budget-approval.toml
│       ├── edges/
│       │   ├── review-to-approval.toml
│       │   └── approval-to-contract.toml
│       └── policies/
│           ├── vendor-preference.md
│           └── budget-thresholds.md
├── integrations/
│   ├── registry.toml             # Available capabilities
│   ├── docusign.toml
│   └── sap.toml
├── compiled/                     # Generated automation artifacts
│   └── procurement/
│       └── n8n/
│           └── review-to-approval.json
└── audit/
    └── log.jsonl                 # Append-only audit trail
```

**Properties inherited from this representation:**
- **Versioning:** `git log` shows the complete history of institutional change.
- **Diffability:** A policy change is a diff to a markdown file. A workflow restructuring is a diff to a TOML graph definition.
- **Review:** A PR to change an authority model is reviewable by the people it affects.
- **Branching:** Proposed institutional changes can be developed on branches, validated, and merged when approved.
- **CI/CD:** Automated validation on every commit. The constraint engine runs in CI.
- **Rollback:** `git revert` undoes an institutional change with full traceability.

---

## 5. Progressive Adoption Path

The system supports incremental adoption. An institution does not need to automate everything on day one.

**Stage 1 — Codification:** Define the institutional model: roles, authority, decision points, policies. No automation, no AI. The value is clarity and version control. Validation ensures consistency. Edge specifications compile to human checklists.

**Stage 2 — Decision support:** Add the intelligence layer. The LLM assists humans at decision points by assembling context, surfacing relevant policies and precedent, and structuring the decision. Decisions are still made by humans.

**Stage 3 — Partial automation:** Add integrations. Edge specifications compile to executable automations where tooling exists. Deterministic transitions run automatically. Humans still make judgments.

**Stage 4 — Delegated agency:** The AI agent operates with institutional authority for defined decision types. Humans oversee and handle exceptions. The override mechanism moves decisions from procedure (Layer 2) to policy-level judgment (Layer 3), is logged, and can inform future workflow refinement.

---

## 6. Key Technical Decisions and Open Questions

### Decided

- **Unified TypeScript architecture** — a single language for the entire system, from model to intelligence layer. TypeScript with Zod provides sufficient type safety and runtime validation for the institutional model, while eliminating the complexity of a cross-language boundary.
- **Petri net execution model** — workflows are executed as Petri nets, giving precise formal semantics for state, concurrency, and synchronization. Transitions are the execution boundary where agents operate.
- **Postcondition-driven verification** — transitions declare required and desired postconditions. The engine verifies them after execution using deterministic checkers and LLM-as-judge fallback.
- **File-based, git-managed institutional model** — the institution is a directory of human-readable configuration files.
- **Decision topology over step-by-step workflow** — the system models judgment points and their relationships, not mechanical steps.
- **Stratified formality** — four layers from hard constraints to tacit knowledge, each with appropriate formalism.
- **pi-agent-core / pi-ai for agent execution** — the agent runtime uses pi libraries for LLM interaction and tool execution.

### Open

- **Ontology of decision types:** The current list (approval, classification, prioritization, allocation, exception handling) is provisional. The real ontology will emerge from modeling actual institutions.
- **Edge specification grammar:** What is the formal structure of an edge specification? How much structure vs. natural language? This needs experimentation.
- **Telescoping depth:** How many levels of abstraction should a workflow support? Is there a natural depth limit?
- **Policy conflict resolution:** When multiple policies apply to a decision and conflict, what is the resolution mechanism? Priority ordering? LLM-mediated synthesis?
- **Multi-tenancy model:** Does each institution get its own project directory, or is there a higher-level structure for organizations with multiple institutional units?
- **Runtime execution model:** When an automated workflow is running and reaches a decision point, what is the execution environment? Does the agent runtime run as a persistent service, or is it invoked on-demand?
- **Petri net extensions:** Do we need colored Petri nets (typed tokens), timed Petri nets (timeouts), or hierarchical Petri nets (sub-nets)? The spike uses simple place/transition nets with payload-carrying tokens — when does this prove insufficient?

---

## 7. Package Structure

```
typescript/
├── package.json
├── tsconfig.json
├── fixtures/                      # Test data (vendor profiles, policies)
│   ├── risk-policy.md
│   └── vendor-acme.json
├── src/
│   ├── index.ts                   # Package entry point
│   ├── types/                     # Core type definitions (Zod schemas + TS interfaces)
│   │   └── index.ts
│   ├── spike/                     # Petri net execution engine (spike)
│   │   ├── index.ts               # Spike entry point (vendor onboarding demo)
│   │   ├── net/
│   │   │   ├── types.ts           # Net, Place, Transition, Token schemas
│   │   │   ├── engine.ts          # Petri net execution engine
│   │   │   └── vendor-onboarding.ts  # Example net definition
│   │   ├── agent/
│   │   │   ├── executor.ts        # Transition → agent execution
│   │   │   ├── llm.ts            # One-shot text generation helper
│   │   │   └── postconditions.ts  # Postcondition verification
│   │   ├── context/
│   │   │   └── store.ts           # Key-value context store
│   │   └── tools/
│   │       ├── lookup-vendor.ts   # Vendor lookup tool
│   │       ├── generate-document.ts  # Document generation tool
│   │       └── send-notification.ts  # Notification tool
│   ├── cli-bridge/                # Model access layer
│   │   └── index.ts
│   ├── orchestration/             # LLM prompt management, conversation
│   │   └── index.ts
│   ├── policy-interpreter/        # Policy assembly, LLM reasoning
│   │   └── index.ts
│   ├── integration-compiler/      # Edge → automation compilation
│   │   └── index.ts
│   ├── agent/                     # Autonomous agent runtime
│   │   └── index.ts
│   └── targets/                   # Compilation target plugins
│       ├── n8n/
│       │   └── index.ts
│       └── human-checklist/
│           └── index.ts
```

---

## 8. First Milestones

1. **✓ Spike: Petri net agent execution boundary.** Defined a Petri net for vendor onboarding, executed transitions via LLM agents with tool access, verified postconditions with deterministic checkers and LLM-as-judge fallback. Validates the core execution model.

2. **Define core model types.** Consolidate `types/index.ts` (institutional model) with `spike/net/types.ts` (Petri net model). Establish a unified type system with Zod schemas for runtime validation.

3. **Build the model/store layer.** File-based persistence for institutional definitions. Read and write TOML/YAML config files. Wire the `cli-bridge` (now model access layer) to this store.

4. **Build the constraint engine.** Validate institutional invariants at definition time and runtime. Authority levels, policy scoping, required inputs.

5. **Model one real workflow.** Take an actual institutional process and represent it as a Petri net with policies, constraints, and integration specifications. This will break the ontology in useful ways.

6. **Build a minimal policy interpreter.** Given a decision context assembled from the model and context store, have an LLM reason about applicable policies and produce a structured recommendation.

7. **Compile one edge.** Take a single edge specification and produce an executable automation for one target platform.

Each milestone validates the architecture at a different boundary. Prioritize the feedback loop over completeness.
