# Agent Runner Design — Intelligence Layer Phase 1

## Overview

A standalone `AgentRunner` class that sits above the core engine, driving agentic transition execution. It consumes `WorkOrder` objects, invokes LLM agents with tools, verifies postconditions, and fires transitions through the engine API.

## Architecture

```
AgentRunner (new)
  ├── observes Engine for enabled agentic transitions
  ├── builds WorkOrders via buildWorkOrder()
  ├── assembles agent context from 3 layers:
  │     1. Token payloads (case data, from work order)
  │     2. Institutional context store (standing facts, key-value)
  │     3. Tools (live data, fetched on demand)
  ├── constructs LLM agent with system prompt + context + scoped tools
  ├── verifies postconditions (deterministic registry + LLM-as-judge fallback)
  ├── classifies failures:
  │     Runtime error → retry (up to 3 attempts)
  │     Substantive failure → escalate to human
  └── fires transition via engine.fireTransition() on success
```

## Components

### 1. AgentRunner

- Initialized with: `Engine`, tool registry (`Map<string, AgentTool>`), institutional context store, postcondition verifier registry, config (retry limit, model, etc.)
- `step(instanceId, actorId)` — executes one enabled agentic transition, returns result
- `run(instanceId, actorId)` — loops `step()` until no agentic transitions remain
- Sequential execution — one transition at a time
- Skips `judgment` and `deterministic` mode transitions (only fires `agentic`)

### 2. InstitutionalContextStore

- Key-value store scoped to an institution
- Keys are namespaced strings (e.g., `contacts.secretary.email`, `config.sms_gateway`)
- Transitions reference keys via `context_sources: string[]`
- Persisted in SQLite (new table: `context_entries` with `institution_id`, `key`, `value`)

### 3. PostconditionVerifier

- Deterministic verifier registry: `Map<string, (result) => boolean>`
- LLM-as-judge fallback for unregistered postconditions
- Returns per-postcondition results with:
  - `satisfied: boolean`
  - `method: "deterministic" | "llm"`
  - `confidence: number` (1.0 for deterministic, 0-1 for LLM)
- Verification details included in audit entries

### 4. Failure Handling

- Runtime errors (tool exceptions, LLM failures) → retry up to 3 times
- Substantive failures (postconditions not met) → escalate as pending judgment
- Escalation surfaces via existing `getPendingJudgments()` mechanism

### 5. Prompt Construction

- System prompt built from work order: intent, policies, postconditions, evidence requirements, output schema
- Context prompt assembled from: token payloads + institutional context (resolved from `context_sources`)
- Tool scoping: only tools listed in `available_tools` are provided to the agent
- Generalized from spike's `buildSystemPrompt` / `buildContextPrompt`

## Data Flow — Single Step

```
1.  engine.getEnabledTransitions(instanceId, actorId)
2.  Filter to mode === "agentic", pick first
3.  buildWorkOrder(engine, instanceId, transitionId)
4.  Resolve context_sources from institutional context store
5.  Construct scoped tools from registry
6.  Build system prompt + context prompt from work order
7.  Create pi-agent-core Agent, invoke with prompt + tools
8.  Collect execution result (text, tool results, payload)
9.  Verify postconditions (deterministic → LLM fallback)
10. If runtime error → retry (up to 3)
11. If postconditions fail → escalate
12. If success → engine.fireTransition() with output payload + evidence + audit metadata
13. Return step result
```

## Context Layers

The agent receives context from three distinct sources:

| Layer | Contains | Source | Example |
|-------|----------|--------|---------|
| Token payloads | Case data flowing through the net | Work order (already implemented) | Applicant name, request date |
| Institutional context | Standing facts about the institution | Key-value store (new, SQLite-persisted) | Secretary's email, SMS gateway config |
| Tools | Live data fetched on demand | Tool execution at runtime | Current vendor registration status |

## Open Decisions from Architecture Doc

These remain open and are out of scope for this phase:

- Instance lifecycle (completed/stuck/suspended detection)
- Guard expression evaluation
- Concurrent transition firing
- Token merging semantics
- Net composition / sub-nets

## TODOs

- [ ] AI-verified postconditions should be flagged with verification method (deterministic vs LLM) and confidence level — surface this in the audit trail
