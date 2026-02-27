# Intelligent Institution Initiative — Technical Architecture Blueprint

## Executive Summary

The Intelligent Institution Initiative is a system that makes institutions programmable by representing their structure, workflows, and decision-making processes as code. Rather than competing with existing workflow automation tools, the system targets the **judgment layer** — the points in institutional workflows where decisions require context, policy, precedent, and discretion.

The architecture is a hybrid Rust/TypeScript system. A Rust core owns the institutional model, invariants, and CLI interface. A TypeScript orchestration layer manages AI-driven policy interpretation, integration compilation, and agent operation. The boundary between them is the CLI's structured output, ensuring reproducibility, auditability, and language-agnostic extensibility.

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

### 2.4 Telescoping Abstraction

Workflows are defined at multiple resolutions simultaneously. A high-level business process decomposes into sub-workflows, which decompose into individual tasks. The level at which something is "atomic" versus "composite" is not fixed — it depends on the observer and the operational context. This mirrors function composition in code: abstraction layers with stable external interfaces and expandable internals.

---

## 3. Hybrid Architecture

### 3.1 Overview

```
┌─────────────────────────────────────────────────────┐
│                  TypeScript Layer                    │
│                                                     │
│  ┌─────────────┐ ┌──────────────┐ ┌──────────────┐  │
│  │     LLM     │ │   Policy     │ │ Integration  │  │
│  │Orchestration│ │ Interpreter  │ │  Compiler    │  │
│  └──────┬──────┘ └──────┬───────┘ └──────┬───────┘  │
│         │               │                │          │
│  ┌──────┴───────────────┴────────────────┴───────┐  │
│  │              Agent Runtime                    │  │
│  └──────────────────┬────────────────────────────┘  │
│                     │                               │
└─────────────────────┼───────────────────────────────┘
                      │  CLI (JSON protocol)
┌─────────────────────┼───────────────────────────────┐
│                     │        Rust Core               │
│  ┌──────────────────┴────────────────────────────┐  │
│  │                CLI Interface                   │  │
│  └──┬──────────┬──────────┬──────────┬───────────┘  │
│     │          │          │          │               │
│  ┌──┴───┐  ┌──┴───┐  ┌──┴───┐  ┌──┴────────────┐  │
│  │Graph │  │Const-│  │Audit │  │  Integration  │  │
│  │Engine│  │raint │  │  Log │  │   Registry    │  │
│  │      │  │Engine│  │      │  │               │  │
│  └──────┘  └──────┘  └──────┘  └───────────────┘  │
│                                                     │
│  ┌─────────────────────────────────────────────────┐│
│  │        Institution Project Directory            ││
│  │   (git-managed, file-based, human-readable)     ││
│  └─────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────┘
```

### 3.2 Rust Core — `libinstitution`

The Rust core is a library exposing both a programmatic API and a CLI. It owns everything where correctness, auditability, and invariant enforcement matter.

#### 3.2.1 Institutional Graph Engine

The source-of-truth representation of the institution: organizations, roles, authority models, decision points, edge specifications, policy attachments, and their relationships. All entities are strongly typed using Rust's algebraic data types (enums with associated data), ensuring that invalid institutional states are unrepresentable.

**Design guidance:**
- Use `serde` for serialization to TOML/YAML (human-authored config) and JSON (machine interchange).
- Use `String` and `serde_json::Value` liberally for the squishy parts (policies, context, LLM outputs) during early development. Tighten types as the ontology crystallizes.
- The graph is parameterized — the same workflow definition may produce different topologies based on context (e.g., procurement under $10k vs. over $100k).

#### 3.2.2 Constraint Engine

Evaluates Layer 1 (hard constraints) at both definition time and runtime. When a workflow is defined or modified, the constraint engine validates that all invariants hold: authority levels are sufficient, required inputs are available, policy scoping is valid. At runtime, it enforces that decisions are made by authorized actors and that edge preconditions are satisfied.

**Design guidance:**
- Constraints are typed predicates attached to the graph. Model them as trait implementations so new constraint types are extensible.
- Validation produces structured error types with full context, enabling the TypeScript layer to present meaningful guidance to the LLM or human operator.

#### 3.2.3 Audit Log

Append-only, structured log of every mutation to the institutional model and every decision made at runtime. Each entry captures: the action, the actor, the timestamp, the prior state, the new state, and the reasoning (if provided).

**Design guidance:**
- JSONL format for the log file — one JSON object per line, streamable and parseable.
- Cryptographic chaining (hash of previous entry included in each new entry) for tamper evidence. This is lightweight to implement and provides a strong auditability guarantee.
- Rust's ownership model ensures audit writes cannot be silently skipped or reordered.

#### 3.2.4 Integration Registry

A typed declaration of what external capabilities the institution has available. Each integration exposes a set of capabilities (e.g., DocuSign exposes `route_for_signature`, SAP exposes `create_purchase_order`) with defined input/output schemas.

**Design guidance:**
- Use Rust traits to define capability interfaces: `Notifier`, `DocumentStore`, `SignatureProvider`, etc.
- The registry lives in the institution project directory alongside the graph — it is part of the institutional definition.
- The registry does not contain credentials or connection details. Those are environment configuration, separate from the institutional model.

#### 3.2.5 CLI Interface

The primary interface for both humans and AI agents. Stateless — all state lives in the project directory. Every command is a transaction: validate, mutate, commit.

**Example commands:**

```bash
# Organizational structure
inst org define --name "Acme Foundation"
inst role create --name "compliance-officer" --authority-level 3

# Decision topology
inst decision define --type approval --domain procurement \
    --requires-authority 2 --output-schema approve_reject
inst edge define --from vendor-review --to contract-generation \
    --spec "Generate purchase order from approved template, route for signature"

# Policy management
inst policy attach --scope "procurement.*" \
    --strength preference \
    --text "Prefer vendors with existing relationships when cost delta < 15%"

# Validation and compilation
inst workflow validate
inst workflow compile --target n8n

# Querying
inst policy list --scope "procurement.vendor-selection" --format json
inst decision history --type vendor-review --last 20 --format json
inst graph export --format dot
```

**Design guidance:**
- Use `clap` for argument parsing with derive macros.
- All commands support `--format json` for machine consumption.
- Exit codes are meaningful and documented (0 = success, 1 = validation error, 2 = invariant violation, etc.).
- The CLI is the contract between the Rust core and everything else. Design it as a public API.

### 3.3 TypeScript Layer

A separate process (or set of processes) that consumes the Rust CLI and adds AI-driven intelligence. Communicates with the Rust core exclusively through CLI invocations and JSON parsing.

#### 3.3.1 LLM Orchestration

Manages the dialogue when an AI agent is programming an institution or supporting human decision-making. Constructs prompts, parses structured outputs, manages conversation state, and translates LLM intent into sequences of CLI commands.

**Design guidance:**
- Use the Anthropic SDK (or Vercel AI SDK) natively in TypeScript.
- Define a prompt template system for different interaction modes: institution definition, decision support, policy interpretation, edge specification authoring.
- Every LLM interaction that results in a mutation to the institutional model should produce a reviewable CLI command sequence before execution.

#### 3.3.2 Policy Interpreter

When a decision point is reached and policies need to inform the judgment, this module:

1. Queries the Rust core for relevant policies: `inst policy list --scope <decision-scope> --format json`
2. Retrieves precedent: `inst decision history --type <decision-type> --format json`
3. Assembles the decision context (inputs, policies, precedent, authority model) into a structured prompt.
4. Invokes the LLM to reason about what the policies imply for this specific case.
5. Returns a structured recommendation or decision, which is recorded back through the CLI.

**Design guidance:**
- Policy scoping (attaching policies to ontological structures rather than relying on retrieval) is the primary mechanism. Policies are *in context* because of their scope, not because of similarity search.
- For institutions with large policy sets, a secondary RAG layer may be needed, but start with scoping and add retrieval only when scoping proves insufficient.
- Policy interpretation results should include reasoning traces for the audit log.

#### 3.3.3 Integration Compiler

Takes an edge specification plus the integration registry and produces an executable automation for a target platform.

**Compilation flow:**
1. Read edge specification from Rust core (natural language intent + structured metadata).
2. Read available integrations: `inst integration list --format json`
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

1. Reads the decision graph to identify pending decisions.
2. Assembles context for each pending decision (inputs, policies, precedent).
3. Invokes the policy interpreter.
4. Makes or recommends decisions (depending on delegation level).
5. Records outcomes through the CLI.
6. Triggers edge execution when decisions are made.

**Design guidance:**
- The agent's authority is defined within the institutional model itself — it's a role with explicit permissions and constraints, same as any human actor.
- All agent actions are CLI commands, producing the same audit trail as human actions.
- The agent should be interruptible and resumable. Its state is derived from the institutional model, not held internally.

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
│       ├── workflow.toml          # Decision graph definition
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
- **Branching:** Proposed institutional changes can be developed on branches, validated with `inst workflow validate`, and merged when approved.
- **CI/CD:** Automated validation on every commit. The constraint engine runs in CI.
- **Rollback:** `git revert` undoes an institutional change with full traceability.

---

## 5. Progressive Adoption Path

The system supports incremental adoption. An institution does not need to automate everything on day one.

**Stage 1 — Codification:** Define the institutional model: roles, authority, decision points, policies. No automation, no AI. The value is clarity and version control. `inst workflow validate` ensures consistency. Edge specifications compile to human checklists.

**Stage 2 — Decision support:** Add the TypeScript layer. The LLM assists humans at decision points by assembling context, surfacing relevant policies and precedent, and structuring the decision. Decisions are still made by humans.

**Stage 3 — Partial automation:** Add integrations. Edge specifications compile to executable automations where tooling exists. Deterministic steps between decisions run automatically. Humans still make judgments.

**Stage 4 — Delegated agency:** The AI agent operates with institutional authority for defined decision types. Humans oversee and handle exceptions. The override mechanism moves decisions from procedure (Layer 2) to policy-level judgment (Layer 3), is logged, and can inform future workflow refinement.

---

## 6. Key Technical Decisions and Open Questions

### Decided

- **Hybrid Rust/TypeScript architecture** — Rust for correctness-critical core, TypeScript for AI orchestration.
- **CLI as the boundary** — all communication between layers is via CLI invocations with JSON output.
- **File-based, git-managed institutional model** — the institution is a directory of human-readable configuration files.
- **Decision topology over step-by-step workflow** — the system models judgment points and their relationships, not mechanical steps.
- **Stratified formality** — four layers from hard constraints to tacit knowledge, each with appropriate formalism.

### Open

- **Ontology of decision types:** The current list (approval, classification, prioritization, allocation, exception handling) is provisional. The real ontology will emerge from modeling actual institutions.
- **Edge specification grammar:** What is the formal structure of an edge specification? How much structure vs. natural language? This needs experimentation.
- **Telescoping depth:** How many levels of abstraction should a workflow support? Is there a natural depth limit?
- **Policy conflict resolution:** When multiple policies apply to a decision and conflict, what is the resolution mechanism? Priority ordering? LLM-mediated synthesis?
- **Multi-tenancy model:** Does each institution get its own project directory, or is there a higher-level structure for organizations with multiple institutional units?
- **Runtime execution model:** When an automated workflow is running and reaches a decision point, what is the execution environment? Does the agent runtime run as a persistent service, or is it invoked on-demand?
- **Serialization boundary optimization:** If CLI invocation overhead becomes a bottleneck, the fallback is exposing the Rust core as a WASM module or native Node module via `napi-rs`. When does this become necessary?

---

## 7. Initial Scaffold — Suggested Crate and Package Structure

### Rust Workspace

```
inst-core/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── inst-model/               # Core data types, graph, ontology
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── organization.rs
│   │   │   ├── role.rs
│   │   │   ├── decision.rs
│   │   │   ├── edge.rs
│   │   │   ├── policy.rs
│   │   │   ├── workflow.rs
│   │   │   └── integration.rs
│   │   └── Cargo.toml
│   ├── inst-constraint/          # Constraint engine, validation
│   │   └── Cargo.toml
│   ├── inst-audit/               # Audit log, cryptographic chaining
│   │   └── Cargo.toml
│   ├── inst-store/               # File-system persistence, serialization
│   │   └── Cargo.toml
│   └── inst-cli/                 # CLI binary
│       └── Cargo.toml
```

### TypeScript Package

```
inst-agent/
├── package.json
├── tsconfig.json
├── src/
│   ├── cli-bridge/               # CLI invocation, JSON parsing
│   ├── orchestration/            # LLM prompt management, conversation
│   ├── policy-interpreter/       # Policy assembly, LLM reasoning
│   ├── integration-compiler/     # Edge → automation compilation
│   ├── agent/                    # Autonomous agent runtime
│   └── targets/                  # Compilation target plugins
│       ├── n8n/
│       ├── api-direct/
│       └── human-checklist/
```

---

## 8. First Milestones

1. **Define `inst-model` types.** Start with `Organization`, `Role`, `Decision`, `Edge`, `Policy`. Use the type system to explore the ontology. Don't optimize — discover.

2. **Build `inst-cli` with `define` and `validate` commands.** Be able to define a simple workflow (2-3 decision points with edges and policies) and validate it. JSON output mode from day one.

3. **Model one real workflow.** Take an actual institutional process and attempt to represent it in the system. This will break the ontology in useful ways.

4. **Build the CLI bridge in TypeScript.** Invoke CLI commands, parse JSON output. Verify the boundary works.

5. **Build a minimal policy interpreter.** Given a decision context assembled from CLI output, have an LLM reason about applicable policies and produce a structured recommendation.

6. **Compile one edge.** Take a single edge specification and produce an executable automation for one target platform.

Each milestone validates the architecture at a different boundary. Prioritize the feedback loop over completeness.
