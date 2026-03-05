# Workflow Execution Specification

## Overview

Workflows are modeled as **Colored Petri Nets (CPNs)**: a formal computational model where
state is represented as tokens flowing through a graph of places and transitions. Unlike
sequential workflow engines, this model natively represents concurrency, non-determinism,
and data-parameterized branching without duplicating structure.

---

## Core Abstractions

### Color Sets

Every place has an associated **color set** — a type constraining the tokens it may hold.
Color sets are defined compositionally:

- Primitive: `INT`, `STRING`, `BOOL`, `UNIT`
- Product: `(A × B)` — ordered pairs
- Sum: `A | B` — tagged union
- List: `LIST(A)`
- Named record: `{ field: A, field: B, ... }`

In practice, the color set of a place represents the *shape of context* flowing through that
point in the workflow — document state, role assignments, policy parameters, case identifiers.

### Multi-sets

Places hold **multi-sets** (bags) of tokens — unordered collections where duplicates are
meaningful. A place may simultaneously hold multiple tokens of the same or different colors.

Notation: `2'a ++ 1'b` means two tokens of value `a` and one of value `b`.

Formally, a multi-set over color set `C` is a function `C → ℕ`.

### Marking

A **marking** is a snapshot of the full workflow state: a map from every place to its
current multi-set of tokens.

```
Marking = Map<PlaceId, MultiSet<Token>>
```

A workflow instance begins at an **initial marking** (typically a single token in a
designated start place) and terminates when a token reaches a designated end place or
a defined terminal condition holds.

### Arc Inscriptions

Arcs carry **expressions** that evaluate to multi-sets. These expressions may reference
the transition's free variables, binding them to token values at runtime.

- Input arc inscription: declares what tokens are *consumed* from the source place
- Output arc inscription: declares what tokens are *produced* into the target place

Example: an arc inscription `(caseId, status)` consumes or produces a token that is the
pair of the bound values of variables `caseId` and `status`.

### Guards

Transitions carry an optional **guard** — a boolean expression over the transition's
free variables. A transition may only fire for a variable binding where the guard
evaluates to `true`.

Guards encode policy and precondition logic directly in the model:

```
guard: role == "approver" && amount < limit
```

---

## Firing Semantics

Execution proceeds via **bindings**: assignments of concrete values to a transition's
free variables.

A binding `(T, σ)` — transition `T` with variable assignment `σ` — is **enabled** iff:

1. For every input arc of `T`, evaluating the arc inscription under `σ` yields a
   multi-set that is *contained in* the current multi-set of that input place.
2. The guard of `T` evaluates to `true` under `σ`.

When a binding fires:

1. For each input arc: subtract the arc inscription (evaluated under `σ`) from the
   source place's multi-set.
2. For each output arc: add the arc inscription (evaluated under `σ`) to the target
   place's multi-set.

Both steps are **atomic** — no intermediate state is observable.

---

## Execution Strategy

### Rationale

The CPN formalism is declarative: it defines *what can happen*, not *in what order to
check*. Implementations choose an execution strategy. For workflow execution (as opposed
to exhaustive state-space verification), an **agenda-based event-driven strategy** is
used. This is appropriate because institutional workflows are sparse — at any moment,
only a small fraction of places hold tokens.

### Agenda

The executor maintains an **agenda**: a queue of candidate bindings awaiting evaluation.

```
Agenda = Queue<(TransitionId, Binding)>
```

### Token Arrival Protocol

When a token arrives at place `P`:

1. For each transition `T` in the output set of `P` (i.e., `P` is an input place of `T`):
   a. Enumerate candidate bindings for `T` that are *consistent with the new token*
   b. For each candidate binding, check whether all other input places of `T` are
      also satisfied
   c. Evaluate the guard
   d. If fully enabled, enqueue `(T, binding)` onto the agenda

### Firing Protocol

The executor dequeues `(T, σ)` from the agenda and:

1. **Validates** the binding is still enabled (tokens may have been consumed by a
   conflicting binding that fired since enqueue time)
2. If invalid: discard and continue
3. If valid: fire the binding — atomically update the marking, then run the
   token arrival protocol for each newly produced token

### Conflict Resolution

Two bindings *conflict* if they require overlapping tokens from the same place.
Conflicts are resolved **optimistically**: bindings are not pre-locked. Instead,
validity is re-checked at fire time (step 1 above). A binding that fails validation
is silently discarded; the token arrival protocol will have already enqueued
non-conflicting alternatives if any exist.

For deterministic workflows (where conflicts should not arise), conflict detection
can be promoted to a runtime assertion.

---

## Binding Search

Binding search is the process of finding all variable assignments `σ` that enable
a given transition. It is implemented as a **constraint join** over the transition's
input places.

### Algorithm

```
function find_bindings(T, marking):
    // Order input arcs by selectivity (fewest tokens first)
    arcs = sort_by_selectivity(T.input_arcs, marking)

    bindings = [{}]  // start with one empty partial binding

    for arc in arcs:
        place_tokens = marking[arc.place]
        bindings = [
            extend(partial, match)
            for partial in bindings
            for match in unify(arc.inscription, place_tokens, partial)
        ]
        if bindings is empty: return []

    return [σ for σ in bindings if T.guard.eval(σ)]
```

The early-exit on empty `bindings` and arc ordering by selectivity prune the search
space significantly. For workflows with bounded, finite color sets, binding search is
always terminating and typically fast.

---

## Concurrency Model

Multiple bindings may be **concurrently enabled** if they involve disjoint sets of
tokens. The execution model supports two modes:

- **Interleaved**: bindings fire one at a time (sequential simulation of concurrency).
  Sufficient for most workflow execution; simpler to implement and reason about.
- **Maximal concurrent step**: all non-conflicting enabled bindings fire simultaneously.
  Useful for bulk processing and faithful simulation of parallel institutional processes.

The default is interleaved execution. Concurrent steps are opt-in per workflow definition.

---

## Judgment Points

A **judgment point** is a transition that cannot be automatically resolved by the
execution engine — it requires human discretion, external input, or policy evaluation
that is not fully encoded in the guard.

Judgment points are modeled as transitions with:

- A guard encoding necessary (but not sufficient) preconditions
- A designated **resolution type**: `approval`, `assignment`, `discretionary`, `escalation`
- An **actor binding**: the color set variable that resolves to the responsible role or agent

When a judgment point becomes enabled, the executor:

1. Does **not** automatically fire it
2. Emits a `PendingJudgment` event with the binding context
3. Suspends the binding in a `waiting` state
4. Resumes firing when an external resolution is received

This preserves the formal execution model while delegating the actual decision to
the appropriate human or policy layer.

---

## State Representation

```
WorkflowInstance {
    id:           InstanceId
    definition:   WorkflowDefinition   // the CPN structure
    marking:      Marking              // current token distribution
    agenda:       Queue<PendingBinding>
    waiting:      Map<BindingId, PendingJudgment>
    history:      Vec<FiringEvent>     // append-only audit log
    status:       Running | Completed | Faulted | Suspended
}

FiringEvent {
    transition:   TransitionId
    binding:      Binding
    consumed:     Map<PlaceId, MultiSet<Token>>
    produced:     Map<PlaceId, MultiSet<Token>>
    timestamp:    Instant
    actor:        Option<ActorId>      // set for judgment points
}
```

The history is an append-only log of all firing events. Full marking history is
reconstructible by replaying from the initial marking.

---

## Termination and Liveness

A workflow instance **terminates** when a token reaches a designated terminal place
and the agenda is empty.

Standard liveness properties verifiable against the model:

- **Reachability**: can a specific marking (e.g., approval granted) be reached?
- **Deadlock freedom**: is there always at least one enabled binding from every
  reachable non-terminal marking?
- **Boundedness**: does any place accumulate unboundedly many tokens?
- **Fairness**: can any transition be permanently starved?

These properties are checked offline against the workflow definition, not at runtime.
Runtime execution assumes a verified definition.