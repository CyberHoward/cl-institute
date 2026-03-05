# Architecture Diagrams — Intelligent Institution Initiative

A working set of Mermaid diagrams for the system architecture. Each diagram isolates one architectural concern.

---

## 1. System Architecture

Unified TypeScript system with core engine and future intelligence layer.

```mermaid
graph TB
    subgraph ts["TypeScript System"]
        subgraph intelligence["Intelligence Layer (future)"]
            agent["Agent Runtime<br/><i>Consumes WorkOrders<br/>Fires transitions via Engine</i>"]
        end

        subgraph core["Core Layer (implemented)"]
            engine["Engine<br/><i>Definition, runtime, query operations<br/>Authority enforcement</i>"]
            db["SQLite DB<br/><i>12 tables, WAL mode<br/>FK enforcement</i>"]
            audit["Audit Log<br/><i>SHA-256 hash chaining<br/>Append-only</i>"]
            context["Context Assembly<br/><i>buildWorkOrder()<br/>9-step work order</i>"]
            validate["Validate<br/><i>Structural checks<br/>Policy coverage</i>"]

            engine --> db
            engine --> audit
            context --> engine
            validate --> engine
        end

        agent --> context
        agent --> engine
    end

    subgraph ext["External (future)"]
        claude["Claude API<br/><i>via pi-ai / pi-agent-core</i>"]
        tools["Domain Tools<br/><i>check-documents, send-notification, etc.</i>"]
    end

    agent --> claude
    agent --> tools

    style intelligence fill:#e8f4f8,stroke:#2196F3
    style core fill:#fce4ec,stroke:#e91e63
    style ext fill:#f5f5f5,stroke:#9e9e9e
```

---

## 2. Formality Spectrum

Four policy strengths from hard constraints to tacit knowledge.

```mermaid
graph LR
    subgraph layer1["Constraint"]
        c_nature["Hard rules<br/>Legal/regulatory mandates"]
        c_enforce["Machine-enforced<br/>Included first in work orders"]
    end

    subgraph layer2["Procedure"]
        p_nature["Defined steps<br/>Authorized deviation"]
        p_enforce["Included in work orders<br/>Logged overrides"]
    end

    subgraph layer3["Preference"]
        po_nature["Intent-bearing guidance"]
        po_enforce["Included in agent context"]
    end

    subgraph layer4["Context"]
        cx_nature["Tacit knowledge<br/>Institutional culture"]
        cx_enforce["Advisory only"]
    end

    layer1 -->|"governs"| layer2
    layer2 -->|"guided by"| layer3
    layer3 -->|"informed by"| layer4

    llm["LLM as Interpreter<br/>(future)"]
    llm -.->|"interprets"| layer3
    llm -.->|"references"| layer4

    style layer1 fill:#ffcdd2,stroke:#e91e63
    style layer2 fill:#ffe0b2,stroke:#ff9800
    style layer3 fill:#c8e6c9,stroke:#4caf50
    style layer4 fill:#bbdefb,stroke:#2196f3
    style llm fill:#f3e5f5,stroke:#9c27b0
```

---

## 3. Engine Firing Protocol

How `fireTransition` works: authority check → token check → atomic consume/produce/audit.

```mermaid
flowchart TD
    call["fireTransition(instanceId, transitionId, actorId, payload)"]

    call --> auth{"Actor authority<br/>≥ requires_authority?"}
    auth -->|"No"| reject_auth["Return: success=false<br/><i>Insufficient authority</i>"]
    auth -->|"Yes"| tokens{"All input places<br/>have tokens?"}
    tokens -->|"No"| reject_token["Return: success=false<br/><i>No token in input place</i>"]
    tokens -->|"Yes"| txn

    subgraph txn["SQLite Transaction (atomic)"]
        snapshot_before["Snapshot marking before"]
        snapshot_before --> consume["Consume tokens<br/><i>DELETE from input places</i>"]
        consume --> produce["Produce tokens<br/><i>INSERT to output places<br/>with output payload</i>"]
        produce --> snapshot_after["Snapshot marking after"]
        snapshot_after --> audit_write["Append audit entry<br/><i>Hash-chained, with evidence</i>"]
    end

    txn --> result["Return: FiringResult<br/><i>success=true, tokens consumed/produced,<br/>audit_entry_id</i>"]

    style txn fill:#f5f5f5,stroke:#9e9e9e
    style reject_auth fill:#ffcdd2,stroke:#e91e63
    style reject_token fill:#ffcdd2,stroke:#e91e63
    style result fill:#e8f5e9,stroke:#4caf50
```

---

## 4. Work Order Assembly

The 9-step context assembly from `buildWorkOrder()`.

```mermaid
flowchart TD
    trigger["buildWorkOrder(engine, instanceId, transitionId)"]

    trigger --> t["Load Transition definition"]
    t --> tokens["1. Read token payloads<br/>from consumed places"]
    tokens --> schema_in["2. Get input_schema"]
    schema_in --> policies["3. Resolve policies by scope<br/><i>domain.transitionId → domain.* → *<br/>Ordered: constraint → context</i>"]
    policies --> ctx["4. Get context_sources"]
    ctx --> schema_out["5. Get output_schema"]
    schema_out --> post["6. Get postconditions<br/><i>required, desired, escalation</i>"]
    post --> evidence["7. Get evidence_requirements"]
    evidence --> tools["8. Get available_tools"]
    tools --> wo["9. Return WorkOrder"]

    wo --> agent["→ Agent Runtime<br/><i>Structured context for<br/>LLM goal construction</i>"]

    style trigger fill:#fff3e0,stroke:#ff9800
    style wo fill:#e8f5e9,stroke:#4caf50
    style agent fill:#e8f4f8,stroke:#2196F3
```

---

## 5. Policy Scope Resolution

How policies are gathered and ordered at a judgment point.

```mermaid
flowchart TD
    query["getPolicies('carta-de-agua.board-decision')"]

    query --> scopes["Build scope list:<br/>1. carta-de-agua.board-decision (exact)<br/>2. carta-de-agua.* (parent)<br/>3. * (global)"]

    scopes --> fetch["SELECT * FROM policies<br/>WHERE scope IN (exact, parent, global)"]

    fetch --> order["Sort by:<br/>1. Strength: constraint → procedure → preference → context<br/>2. Specificity: exact → parent → global"]

    order --> result["Ordered Policy[]<br/><i>Constraints first, most specific first</i>"]

    style query fill:#fff3e0,stroke:#ff9800
    style result fill:#e8f5e9,stroke:#4caf50
```

---

## 6. Carta de Agua — Petri Net

The ASADA water availability letter process encoded as a CPN.

```mermaid
graph LR
    p1((("intake<br/>🔵")))
    t1["receive-request<br/><i>mode: deterministic<br/>authority: 2</i>"]
    p2((("documents-<br/>pending")))
    t2["check-completeness<br/><i>mode: agentic<br/>authority: 2</i>"]
    p3((("documents-<br/>complete")))
    t3["triage-case<br/><i>mode: judgment<br/>authority: 2</i>"]
    p4((("triaged")))
    t4["check-scarcity<br/><i>mode: deterministic<br/>authority: 2</i>"]
    p5((("technical-<br/>review-ready")))
    t5["compile-board-<br/>packet<br/><i>mode: agentic<br/>authority: 2</i>"]
    p6((("board-<br/>ready")))
    t6["board-decision<br/><i>mode: judgment<br/>authority: 4 🔒</i>"]
    p7((("decided")))
    t7["deliver-decision<br/><i>mode: agentic<br/>authority: 2</i>"]
    p8((("delivered<br/>🏁")))

    p1 --> t1 --> p2 --> t2 --> p3 --> t3 --> p4 --> t4 --> p5 --> t5 --> p6 --> t6 --> p7 --> t7 --> p8

    style p1 fill:#bbdefb,stroke:#2196f3
    style p8 fill:#e8f5e9,stroke:#4caf50
    style t1 fill:#fff3e0,stroke:#ff9800
    style t2 fill:#e8f4f8,stroke:#2196F3
    style t3 fill:#fce4ec,stroke:#e91e63
    style t4 fill:#fff3e0,stroke:#ff9800
    style t5 fill:#e8f4f8,stroke:#2196F3
    style t6 fill:#fce4ec,stroke:#e91e63
    style t7 fill:#e8f4f8,stroke:#2196F3
```

**Legend:** 🟠 deterministic | 🔵 agentic | 🔴 judgment | 🔒 board-only (authority 4)

---

## 7. Authority Model

How actors, roles, and authority levels gate transition firing.

```mermaid
graph TB
    subgraph institution["ASADA Playas de Nosara"]
        subgraph roles["Roles"]
            admin["administrator<br/><i>authority: 2</i>"]
            tech["technical-operator<br/><i>authority: 2</i>"]
            pres["president<br/><i>authority: 3</i>"]
            board["junta-directiva<br/><i>authority: 4</i>"]
        end

        subgraph actors["Actors"]
            carlos["Don Carlos<br/><i>type: human</i>"]
            bot["carta-agent<br/><i>type: agent</i>"]
            junta["Board<br/><i>type: human</i>"]
        end

        carlos -.->|"has role"| admin
        bot -.->|"has role"| admin
        junta -.->|"has role"| board
    end

    subgraph transitions["Transition Access"]
        t_admin["check-completeness<br/>compile-board-packet<br/>deliver-decision<br/><i>requires_authority: 2</i>"]
        t_board["board-decision<br/><i>requires_authority: 4</i>"]
    end

    carlos -->|"authority 2 ✅"| t_admin
    carlos -->|"authority 2 ❌"| t_board
    junta -->|"authority 4 ✅"| t_board

    style roles fill:#f5f5f5,stroke:#9e9e9e
    style actors fill:#e8f5e9,stroke:#4caf50
    style t_board fill:#fce4ec,stroke:#e91e63
```

---

## 8. Audit Chain

Cryptographic hash chaining for tamper-evident audit trail.

```mermaid
graph LR
    e1["Entry 1<br/><i>instance_created</i><br/>prev_hash: GENESIS<br/>hash: abc123..."]
    e2["Entry 2<br/><i>transition_fired</i><br/>prev_hash: abc123...<br/>hash: def456..."]
    e3["Entry 3<br/><i>transition_fired</i><br/>prev_hash: def456...<br/>hash: ghi789..."]

    e1 -->|"hash chain"| e2 -->|"hash chain"| e3

    verify["verifyChain()<br/><i>Recompute each hash<br/>Verify prev_hash links</i>"]
    verify -.-> e1
    verify -.-> e2
    verify -.-> e3

    style e1 fill:#bbdefb,stroke:#2196f3
    style e2 fill:#bbdefb,stroke:#2196f3
    style e3 fill:#bbdefb,stroke:#2196f3
    style verify fill:#e8f5e9,stroke:#4caf50
```

---

## 9. Decision Point Anatomy

The components of a judgment transition.

```mermaid
graph TB
    dp["Judgment Transition"]

    dp --> dt["decision_type<br/><i>approval | classification |<br/>prioritization | allocation |<br/>exception_handling</i>"]

    dp --> inputs["Token Payloads<br/><i>From consumed places</i>"]

    dp --> gov["Policies<br/><i>Resolved by scope<br/>Ordered: constraint → context</i>"]

    dp --> auth["requires_authority<br/><i>Numeric level<br/>Checked against actor roles</i>"]

    dp --> out["output_schema<br/><i>What the decision produces</i>"]

    dp --> evidence["evidence_requirements<br/><i>artifact | reference | attestation</i>"]

    dp --> post["postconditions<br/><i>required / desired / escalation</i>"]

    dp --> trail["Audit Entry<br/><i>Hash-chained, with evidence,<br/>reasoning, marking snapshots</i>"]

    style dp fill:#fff3e0,stroke:#ff9800
    style auth fill:#ffcdd2,stroke:#e91e63
    style gov fill:#c8e6c9,stroke:#4caf50
    style trail fill:#bbdefb,stroke:#2196f3
```

---

## 10. Component Dependency Map

```mermaid
graph BT
    subgraph core["Core Layer — src/core/"]
        types["types.ts<br/><i>25+ types</i>"]
        db["db.ts<br/><i>SQLite schema</i>"]
        engine["engine.ts<br/><i>Engine class</i>"]
        audit["audit.ts<br/><i>AuditLog class</i>"]
        context["context.ts<br/><i>buildWorkOrder()</i>"]
        validate["validate.ts<br/><i>validateNet()</i>"]
        idx["index.ts<br/><i>barrel export</i>"]

        engine --> db
        engine --> audit
        engine --> types
        audit --> db
        audit --> types
        context --> engine
        context --> types
        validate --> engine
        validate --> types
        db --> types
        idx --> engine
        idx --> audit
        idx --> context
        idx --> validate
    end

    subgraph future["Intelligence Layer (future)"]
        agent["Agent Runtime"]
    end

    agent --> context
    agent --> engine

    subgraph ext["External (future)"]
        claude["Claude API<br/><i>pi-ai / pi-agent-core</i>"]
    end

    agent --> claude

    style core fill:#fce4ec,stroke:#e91e63
    style future fill:#e8f4f8,stroke:#2196F3
    style ext fill:#f5f5f5,stroke:#9e9e9e
```
