# Architecture Diagrams — Intelligent Institution Initiative

A working set of Mermaid diagrams for refining the system architecture. Each diagram isolates one architectural concern.

---

## 1. System Architecture

Unified TypeScript system with core and intelligence layers.

```mermaid
graph TB
    subgraph ts["TypeScript System"]
        subgraph intelligence["Intelligence Layer"]
            orch["LLM Orchestration"]
            pi["Policy Interpreter"]
            ic["Integration Compiler"]
            agent["Agent Runtime"]

            orch --> agent
            pi --> agent
            ic --> agent
        end

        subgraph core["Core Layer"]
            access["Model Access Layer<br/><i>stateless, typed</i>"]
            graph_eng["Petri Net Engine<br/><i>institutional model</i>"]
            constraint["Constraint Engine<br/><i>invariant enforcement</i>"]
            audit["Audit Log<br/><i>cryptographic chaining</i>"]
            registry["Integration Registry<br/><i>capability declarations</i>"]
            store["Store<br/><i>file-system persistence</i>"]

            access --> graph_eng
            access --> constraint
            access --> audit
            access --> registry
            graph_eng --> store
            audit --> store
        end

        agent --> access
        pi --> access
        ic --> access
    end

    subgraph fs["Institution Project Directory — Git Managed"]
        toml["roles/*.toml<br/>workflows/*.toml<br/>integrations/*.toml"]
        policies["policies/*.md"]
        compiled["compiled/*.json"]
        auditlog["audit/log.jsonl"]
    end

    store --> fs

    style intelligence fill:#e8f4f8,stroke:#2196F3
    style core fill:#fce4ec,stroke:#e91e63
    style fs fill:#e8f5e9,stroke:#4caf50
```

---

## 2. Formality Spectrum

Four stratified layers from hard constraints to tacit knowledge.

```mermaid
graph LR
    subgraph layer1["Layer 1: Constraints"]
        c_nature["Hard rules<br/>Legal/regulatory mandates"]
        c_repr["Typed predicates<br/>Formal logic"]
        c_enforce["Machine-enforced<br/>Compile-time + runtime"]
    end

    subgraph layer2["Layer 2: Procedures"]
        p_nature["Defined steps<br/>Authorized deviation"]
        p_repr["State machines<br/>Decision graphs"]
        p_enforce["Deterministic execution<br/>Logged overrides"]
    end

    subgraph layer3["Layer 3: Policies"]
        po_nature["Intent-bearing guidance<br/>Preferences"]
        po_repr["Structured natural language<br/>Semantic metadata"]
        po_enforce["LLM-interpreted<br/>At decision time"]
    end

    subgraph layer4["Layer 4: Context"]
        cx_nature["Tacit knowledge<br/>Institutional culture"]
        cx_repr["Unstructured<br/>natural language"]
        cx_enforce["LLM-referenced<br/>Advisory only"]
    end

    layer1 -->|"governs"| layer2
    layer2 -->|"guided by"| layer3
    layer3 -->|"informed by"| layer4

    llm["LLM as Interpreter"]
    llm -.->|"bridges"| layer3
    llm -.->|"bridges"| layer4
    llm -.->|"reads"| layer2

    style layer1 fill:#ffcdd2,stroke:#e91e63
    style layer2 fill:#ffe0b2,stroke:#ff9800
    style layer3 fill:#c8e6c9,stroke:#4caf50
    style layer4 fill:#bbdefb,stroke:#2196f3
    style llm fill:#f3e5f5,stroke:#9c27b0
```

---

## 3. Petri Net Execution Loop

The core engine cycle: find enabled transitions, fire via agent, verify postconditions.

```mermaid
flowchart TD
    start([Start: Initial Marking]) --> find

    find["Find enabled transitions<br/><i>All input places have tokens<br/>Guard evaluates true</i>"]

    find -->|"None enabled"| check_terminal
    find -->|"Transitions found"| select

    check_terminal{"Terminal<br/>marking?"}
    check_terminal -->|"Yes"| completed([Status: Completed])
    check_terminal -->|"No"| stuck([Status: Stuck / Deadlocked])

    select["Select transition<br/><i>Priority / FIFO</i>"]

    select --> gather["Gather context<br/><i>Read token payloads from input places<br/>Resolve context_sources from store</i>"]

    gather --> execute["Agent Executor<br/><i>LLM + tools</i>"]

    execute --> verify{"Verify<br/>postconditions"}

    verify -->|"All required met"| fire["Fire transition<br/><i>Consume input tokens<br/>Produce output tokens with payload<br/>Merge payload into context store</i>"]
    verify -->|"Required not met"| escalation{"Escalation<br/>path?"}

    escalation -->|"Yes"| escalate["Trigger escalation<br/><i>Log failure + escalation event</i>"]
    escalation -->|"No"| fail["Log failure<br/><i>Transition did not fire</i>"]

    fire --> log["Append to execution log<br/><i>transition, marking before/after,<br/>agent actions, postcondition results</i>"]

    log --> max_check{"Max steps<br/>reached?"}
    max_check -->|"No"| find
    max_check -->|"Yes"| maxsteps([Status: Max Steps])

    escalate --> find
    fail --> find

    style start fill:#e8f5e9,stroke:#4caf50
    style completed fill:#e8f5e9,stroke:#4caf50
    style stuck fill:#ffcdd2,stroke:#e91e63
    style maxsteps fill:#ffe0b2,stroke:#ff9800
    style execute fill:#e8f4f8,stroke:#2196F3
    style verify fill:#fff3e0,stroke:#ff9800
```

---

## 4. Agent Transition Execution

What happens inside the Agent Executor when a transition fires.

```mermaid
sequenceDiagram
    participant Engine as Petri Net Engine
    participant Executor as Agent Executor
    participant LLM as Claude (Sonnet)
    participant Tools as Tool Registry
    participant Verify as Postcondition Verifier

    Engine->>Executor: executeTransition(transition, context, tools)

    Note over Executor: Build system prompt from:<br/>- transition.intent<br/>- transition.postconditions<br/>- transition.mode

    Note over Executor: Build context prompt from:<br/>- input token payloads<br/>- context_sources lookups

    Executor->>LLM: generateText(system, context, tools)

    loop Tool use loop (max 10 steps)
        LLM->>Tools: call tool(params)
        Tools-->>LLM: tool result
        Note over LLM: Reason about next step<br/>given tool results
    end

    LLM-->>Executor: final text + all tool results

    Executor->>Verify: verify(postconditions, executionResult)

    alt Deterministic verifier exists
        Note over Verify: Inspect tool results directly<br/>e.g., lookup-vendor returned active
    else No deterministic verifier
        Verify->>LLM: LLM-as-judge:<br/>"Is this postcondition satisfied?"
        LLM-->>Verify: true / false
    end

    Verify-->>Executor: postcondition results map
    Executor-->>Engine: ExecutionResult + postcondition results + payload
```

---

## 5. Postcondition Verification Strategy

Two-tier verification: deterministic checks first, LLM-as-judge fallback.

```mermaid
flowchart LR
    pc["Postcondition<br/>to verify"] --> lookup{"Deterministic<br/>verifier exists?"}

    lookup -->|"Yes"| det["Deterministic Check<br/><i>Inspect tool results directly</i>"]
    lookup -->|"No"| judge["LLM-as-Judge<br/><i>Send evidence to Claude</i>"]

    det --> result_det{"Tool result<br/>matches criteria?"}
    result_det -->|"Yes"| satisfied["Postcondition: SATISFIED"]
    result_det -->|"No"| unsatisfied["Postcondition: NOT SATISFIED"]

    judge --> prompt["Assemble evidence:<br/>- Agent reasoning<br/>- Tool calls + results<br/>- Postcondition text"]
    prompt --> ask["Ask: 'Given this evidence,<br/>is the postcondition satisfied?'"]
    ask --> parse["Parse response"]
    parse --> result_judge{"true / false?"}
    result_judge -->|"true"| satisfied
    result_judge -->|"false"| unsatisfied

    subgraph examples["Deterministic Verifier Examples"]
        ex1["vendor-identity-confirmed:<br/>lookup-vendor returned found=true<br/>& registrationStatus=active"]
        ex2["notification-sent:<br/>send-notification returned success=true"]
    end

    style satisfied fill:#e8f5e9,stroke:#4caf50
    style unsatisfied fill:#ffcdd2,stroke:#e91e63
    style det fill:#c8e6c9,stroke:#4caf50
    style judge fill:#bbdefb,stroke:#2196f3
    style examples fill:#f5f5f5,stroke:#9e9e9e
```

---

## 6. Policy Interpretation Flow

How policies are gathered, scoped, and applied at a judgment point.

```mermaid
flowchart TD
    trigger["Decision point reached<br/><i>e.g., procurement.vendor-selection</i>"]

    trigger --> scope["Scope Resolution<br/><i>Walk hierarchy:</i><br/>procurement.vendor-selection<br/>→ procurement.*<br/>→ global"]

    scope --> query["Query model<br/><i>policy list by scope</i>"]
    query --> precedent["Fetch precedent<br/><i>decision history by type</i>"]

    precedent --> order["Order policies<br/><i>1. By strength: constraint → procedure → preference → context</i><br/><i>2. By specificity: exact scope → parent → global</i>"]

    order --> assemble["Assemble decision context:<br/>- Case inputs / documents<br/>- Applicable policies (ordered)<br/>- Historical precedent<br/>- Authority model<br/>- Available integrations"]

    assemble --> llm["LLM Reasoning<br/><i>Claude interprets policies<br/>against specific case</i>"]

    llm --> rec["Structured Recommendation"]

    rec --> fields["Fields:<br/>- action: approve / reject / escalate / request-info<br/>- confidence: 0.0 – 1.0<br/>- reasoning: chain of reasoning<br/>- contributing_policies: which policies applied<br/>- binding_constraints: hard constraints that must hold<br/>- suggested_conditions: if conditional approval"]

    fields --> record["Record decision<br/><i>Audit-logged with full trace</i>"]

    style trigger fill:#fff3e0,stroke:#ff9800
    style llm fill:#e8f4f8,stroke:#2196F3
    style rec fill:#e8f5e9,stroke:#4caf50
```

---

## 7. Edge Compilation Pipeline

How natural-language edge specs become executable automations.

```mermaid
flowchart LR
    edge["Edge Specification<br/><i>Natural language intent:<br/>'Generate PO from approved template,<br/>route for signature'</i>"]

    edge --> read_reg["Read integration registry<br/><code>inst integration list --format json</code>"]

    read_reg --> match["LLM Capability Matching<br/><i>Map intent → available capabilities</i><br/><i>e.g., docusign.route_for_signature,<br/>sap.create_purchase_order</i>"]

    match --> resolve["Resolved Capabilities<br/><i>Each with confidence score</i>"]

    resolve --> target{"Compilation Target"}

    target -->|"n8n"| n8n["N8n Plugin<br/><i>Generate workflow JSON</i><br/>Trigger → Integration → Conditional → Error"]
    target -->|"api-direct"| api["API Direct Plugin<br/><i>Generate API call sequence</i>"]
    target -->|"human-checklist"| checklist["Checklist Plugin<br/><i>Generate Markdown checklist</i><br/>Steps, policy reminders, sign-off"]

    n8n --> validate["Validate output<br/><i>All referenced capabilities exist<br/>in integration registry</i>"]
    api --> validate
    checklist --> validate

    validate --> store["Store compiled artifact<br/><i>compiled/procurement/n8n/...</i><br/><i>Artifact, not source of truth</i>"]

    style edge fill:#fff3e0,stroke:#ff9800
    style match fill:#e8f4f8,stroke:#2196F3
    style n8n fill:#e8f5e9,stroke:#4caf50
    style api fill:#e8f5e9,stroke:#4caf50
    style checklist fill:#e8f5e9,stroke:#4caf50
```

---

## 8. Vendor Onboarding — Petri Net

The concrete spike scenario as a Petri net with transition modes and tool bindings.

```mermaid
graph LR
    p1((("request-<br/>submitted<br/>🔵")))

    t1["verify-vendor<br/><i>mode: deterministic</i><br/><i>tools: lookup-vendor</i>"]

    p2((("vendor-<br/>verified")))

    t2["assess-risk<br/><i>mode: judgment</i><br/><i>tools: generate-document</i>"]

    p3((("risk-<br/>assessed")))

    t3["notify-compliance<br/><i>mode: agentic</i><br/><i>tools: send-notification</i>"]

    p4((("compliance-<br/>notified")))

    t4["approve-onboarding<br/><i>mode: judgment</i><br/><i>tools: generate-document</i>"]

    p5((("onboarding-<br/>approved<br/>🏁")))

    p1 -->|"consumes"| t1
    t1 -->|"produces<br/>+ vendor profile payload"| p2
    p2 -->|"consumes"| t2
    t2 -->|"produces<br/>+ risk assessment payload"| p3
    p3 -->|"consumes"| t3
    t3 -->|"produces<br/>+ notification confirmation"| p4
    p4 -->|"consumes"| t4
    t4 -->|"produces<br/>+ approval decision payload"| p5

    style p1 fill:#bbdefb,stroke:#2196f3
    style p2 fill:#f5f5f5,stroke:#9e9e9e
    style p3 fill:#f5f5f5,stroke:#9e9e9e
    style p4 fill:#f5f5f5,stroke:#9e9e9e
    style p5 fill:#e8f5e9,stroke:#4caf50
    style t1 fill:#fff3e0,stroke:#ff9800
    style t2 fill:#fce4ec,stroke:#e91e63
    style t3 fill:#e8f4f8,stroke:#2196F3
    style t4 fill:#fce4ec,stroke:#e91e63
```

---

## 9. Token Payload Data Flow

How data propagates through the net via colored tokens.

```mermaid
sequenceDiagram
    participant P1 as request-submitted
    participant T1 as verify-vendor
    participant P2 as vendor-verified
    participant T2 as assess-risk
    participant P3 as risk-assessed
    participant T3 as notify-compliance
    participant P4 as compliance-notified
    participant T4 as approve-onboarding
    participant P5 as onboarding-approved
    participant CS as Context Store

    Note over P1: Token: { vendorName: "Acme Corp",<br/>requestedBy: "procurement-lead" }

    P1->>T1: consume token
    T1->>T1: lookup-vendor("Acme Corp")
    T1->>P2: produce token + payload
    T1->>CS: merge { vendor: { name, regNumber, status, certs } }

    Note over P2: Token: { vendor: { name: "Acme Corp",<br/>regNumber: "REG-2024-1234",<br/>status: "active", certs: [...] } }

    P2->>T2: consume token
    Note over T2: context_sources: ["vendor", "risk-policy"]
    T2->>CS: read vendor data + risk-policy.md
    T2->>T2: generate-document(risk assessment)
    T2->>P3: produce token + payload
    T2->>CS: merge { riskLevel: "medium", riskFactors: [...] }

    Note over P3: Token: { riskLevel: "medium",<br/>riskFactors: [...],<br/>assessmentDoc: "..." }

    P3->>T3: consume token
    Note over T3: Reads riskLevel to choose channel
    T3->>T3: send-notification(Slack, medium risk)
    T3->>P4: produce token + payload
    T3->>CS: merge { notificationChannel: "slack", notifiedAt: "..." }

    P4->>T4: consume token
    Note over T4: context_sources: ["vendor", "riskLevel", "riskFactors"]
    T4->>CS: read all accumulated context
    T4->>T4: generate-document(approval recommendation)
    T4->>P5: produce token + payload

    Note over P5: Token: { decision: "conditional-approval",<br/>conditions: [...], rationale: "..." }
```

---

## 10. Progressive Adoption Stages

The four stages from codification to delegated agency.

```mermaid
graph TD
    subgraph s1["Stage 1: Codification"]
        s1a["Define institutional model<br/><i>Roles, authority, decisions, policies</i>"]
        s1b["Version control with git"]
        s1c["inst workflow validate"]
        s1d["Edge specs → human checklists"]
        s1val["Value: Clarity + traceability"]
    end

    subgraph s2["Stage 2: Decision Support"]
        s2a["Add TypeScript + LLM layer"]
        s2b["Context assembly at decision points"]
        s2c["Policy + precedent surfacing"]
        s2d["Structured recommendations"]
        s2val["Value: Better-informed human decisions"]
    end

    subgraph s3["Stage 3: Partial Automation"]
        s3a["Add integration registry"]
        s3b["Edge specs → n8n / API automations"]
        s3c["Deterministic steps run automatically"]
        s3d["Humans still make judgments"]
        s3val["Value: Reduced mechanical work"]
    end

    subgraph s4["Stage 4: Delegated Agency"]
        s4a["Agent operates with institutional authority"]
        s4b["Defined decision types delegated to AI"]
        s4c["Human oversight for exceptions"]
        s4d["Overrides logged + inform refinement"]
        s4val["Value: Scalable institutional judgment"]
    end

    s1 -->|"+ TypeScript layer"| s2
    s2 -->|"+ Integrations"| s3
    s3 -->|"+ Agent authority"| s4

    style s1 fill:#e8f5e9,stroke:#4caf50
    style s2 fill:#c8e6c9,stroke:#4caf50
    style s3 fill:#bbdefb,stroke:#2196f3
    style s4 fill:#e8f4f8,stroke:#2196F3
```

---

## 11. Decision Point Anatomy

The components of a judgment point and their relationships.

```mermaid
graph TB
    dp["Judgment Point<br/><i>(Decision Node)</i>"]

    dp --> dt["Decision Type<br/><i>approval | classification |<br/>prioritization | allocation |<br/>exception_handling</i>"]

    dp --> inputs["Inputs<br/><i>Documents, data,<br/>prior decisions, context</i>"]

    dp --> gov["Governing Policies<br/><i>Scoped by domain + type</i><br/><i>Ordered: constraint → context</i>"]

    dp --> auth["Authority Model<br/><i>Required level<br/>Delegation rules<br/>Escalation paths</i>"]

    dp --> prec["Precedent<br/><i>Historical decisions<br/>of same type</i>"]

    dp --> out["Output Schema<br/><i>approve/reject | ranking |<br/>modified doc | routing choice</i>"]

    dp --> trail["Accountability Trail<br/><i>Who decided, when,<br/>based on what, reasoning</i>"]

    gov --> l1["Layer 1: Constraints<br/><i>Must hold — blocks if violated</i>"]
    gov --> l2["Layer 2: Procedures<br/><i>Follow unless overridden</i>"]
    gov --> l3["Layer 3: Policies<br/><i>LLM interprets guidance</i>"]
    gov --> l4["Layer 4: Context<br/><i>Advisory — institutional culture</i>"]

    trail --> audit_log["Audit Log<br/><i>Cryptographically chained<br/>Append-only</i>"]

    style dp fill:#fff3e0,stroke:#ff9800
    style auth fill:#ffcdd2,stroke:#e91e63
    style gov fill:#c8e6c9,stroke:#4caf50
    style audit_log fill:#bbdefb,stroke:#2196f3
```

---

## 12. CPN Formal Execution — Agenda-Based Strategy

The formal workflow execution model from the CPN specification.

```mermaid
flowchart TD
    arrive["Token arrives at Place P"]

    arrive --> enum["For each transition T<br/>where P is an input place"]

    enum --> bind["Enumerate candidate bindings<br/><i>Constraint join over input arcs<br/>Ordered by selectivity</i>"]

    bind --> check_inputs{"All input places<br/>of T satisfied?"}
    check_inputs -->|"No"| skip["Skip this binding"]
    check_inputs -->|"Yes"| guard{"Guard evaluates<br/>true?"}
    guard -->|"No"| skip
    guard -->|"Yes"| enqueue["Enqueue (T, binding)<br/>onto agenda"]

    subgraph firing["Firing Protocol"]
        dequeue["Dequeue (T, σ) from agenda"]
        dequeue --> revalidate{"Binding still<br/>valid?<br/><i>(tokens may have been<br/>consumed since enqueue)</i>"}
        revalidate -->|"No"| discard["Discard binding"]
        revalidate -->|"Yes"| is_judgment{"Judgment<br/>point?"}
        is_judgment -->|"No"| fire["Fire: atomically update marking<br/><i>Subtract input arcs (evaluated under σ)<br/>Add output arcs (evaluated under σ)</i>"]
        is_judgment -->|"Yes"| suspend["Emit PendingJudgment<br/>Suspend in waiting state"]
        suspend --> external["Await external resolution<br/><i>Human / Agent / Policy layer</i>"]
        external --> fire
        fire --> propagate["Run token arrival protocol<br/>for each newly produced token"]
    end

    enqueue --> dequeue

    subgraph conflict["Conflict Resolution"]
        note["Optimistic: no pre-locking<br/>Re-validate at fire time<br/>Silently discard invalid bindings"]
    end

    style arrive fill:#e8f5e9,stroke:#4caf50
    style firing fill:#f5f5f5,stroke:#9e9e9e
    style conflict fill:#fff3e0,stroke:#ff9800
    style suspend fill:#fce4ec,stroke:#e91e63
```

---

## 13. Component Dependency Map

How the TypeScript modules depend on each other.

```mermaid
graph BT
    subgraph core["Core Layer"]
        access["Model Access Layer<br/><i>Typed model operations</i>"]
        engine["Petri Net Engine<br/><i>Execution, marking</i>"]
    end

    subgraph ts["Intelligence Layer"]
        orch["LLM Orchestration<br/><i>Prompt construction,<br/>conversation state</i>"]
        pi["Policy Interpreter<br/><i>Scope resolution,<br/>LLM reasoning</i>"]
        ic["Integration Compiler<br/><i>Edge → automation</i>"]
        agent["Agent Runtime<br/><i>Decision loop,<br/>authority checks</i>"]

        subgraph targets["Compilation Targets"]
            n8n["n8n"]
            api["API Direct"]
            checklist["Human Checklist"]
        end
    end

    orch --> access
    pi --> access
    pi --> orch
    ic --> access
    ic --> orch
    ic --> targets
    agent --> access
    agent --> engine
    agent --> orch
    agent --> pi

    subgraph ext["External"]
        claude["Claude API<br/><i>via pi-ai / pi-agent-core</i>"]
        integrations["External Integrations<br/><i>DocuSign, SAP, Slack, ...</i>"]
    end

    orch --> claude
    n8n --> integrations
    api --> integrations

    style core fill:#fce4ec,stroke:#e91e63
    style ts fill:#e8f4f8,stroke:#2196F3
    style targets fill:#e8f5e9,stroke:#4caf50
    style ext fill:#f5f5f5,stroke:#9e9e9e
```

---

## Notes for Refinement

**Open questions these diagrams surface:**

1. **Diagram 3 (Execution Loop)**: The current spike uses simple FIFO selection. The formal CPN spec (Diagram 12) uses agenda-based execution. When does the spike engine evolve to agenda-based execution?

2. **Diagram 6 (Policy Interpretation)**: Policy conflict resolution is shown as simple ordering. Should there be an explicit conflict detection + LLM-mediated synthesis step?

3. **Diagram 7 (Edge Compilation)**: The "validate" step checks capabilities exist. Should it also check that the generated automation satisfies the edge's postconditions?

4. **Diagram 8 (Vendor Onboarding)**: This is a linear net. What does a branching scenario look like? (e.g., high-risk vendor triggers additional review path)

5. **Diagram 12 (CPN Formal Execution)**: Judgment points suspend and await external resolution. How does this interact with the Agent Runtime? Is the agent "external" from the engine's perspective?

6. **Missing diagram**: Multi-workflow interaction — how do tokens or decisions in one workflow affect another?
