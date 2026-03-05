# Spike: Petri Net Agent Execution Boundary

## Goal

Validate that an LLM agent can reliably interpret transition definitions from a Petri net and execute them to satisfy postconditions using real tools. The Petri net engine is intentionally minimal — the spike tests the **interface contract**, not the engine.

## What we're testing

1. Can an agent read a transition's `intent` + `context_sources` and figure out what to do?
2. Can the system verify `postconditions` after agent execution?
3. What's the right granularity for transition definitions?
4. Where does the agent fail, and are those failures recoverable via the net?

## Scenario: Vendor Onboarding

```
request-submitted ──→ [verify-vendor] ──→ vendor-verified
                                              │
vendor-verified ──→ [assess-risk] ──→ risk-assessed
                                          │
risk-assessed ──→ [notify-compliance] ──→ compliance-notified
                                              │
compliance-notified ──→ [approve-onboarding] ──→ onboarding-approved
```

Four transitions covering three types:
- **verify-vendor**: deterministic (API/tool call, no judgment)
- **assess-risk**: judgment (LLM reasons over data + policy)
- **notify-compliance**: agentic (real side effect — send notification)
- **approve-onboarding**: judgment + gate (LLM recommends, postcondition requires explicit approval)

---

## Project structure

```
iii-spike/
├── src/
│   ├── net/
│   │   ├── types.ts          # Zod schemas for Place, Transition, Net, Marking
│   │   ├── engine.ts         # Minimal execution loop (find enabled → fire → check postconditions)
│   │   └── vendor-onboarding.ts  # The net definition for the test scenario
│   ├── agent/
│   │   ├── executor.ts       # Takes a transition + context, returns execution result
│   │   └── postconditions.ts # Verifies postconditions against execution result
│   ├── tools/
│   │   ├── lookup-vendor.ts  # Mock or real vendor lookup
│   │   ├── send-notification.ts  # Sends actual notification (email/Slack/console)
│   │   └── generate-document.ts  # Generates a risk assessment summary
│   ├── context/
│   │   └── store.ts          # Simple key-value store for place data (tokens carry payloads)
│   └── index.ts              # CLI entry point: run the net end-to-end
├── fixtures/
│   ├── vendor-acme.json      # Test vendor data
│   └── risk-policy.md        # A short policy document the agent reads
├── package.json
├── tsconfig.json
└── .env                      # ANTHROPIC_API_KEY or OPENAI_API_KEY
```

---

## Phase 1: Define the type system (1-2 hours)

The core types. Everything else builds on these.

### `src/net/types.ts`

```typescript
import { z } from "zod";

export const PlaceSchema = z.object({
  id: z.string(),
  description: z.string(),
});

export const PostconditionSchema = z.object({
  /** Must all be true for the transition to be considered fired */
  required: z.array(z.string()),
  /** Ideal outcomes — logged but not blocking */
  desired: z.array(z.string()).optional(),
  /** If required postconditions aren't met within timeout, trigger these */
  escalation: z.array(z.string()).optional(),
});

export const TransitionSchema = z.object({
  id: z.string(),
  
  // -- Formal (what the net cares about) --
  consumes: z.array(z.string()),     // input place IDs — tokens removed
  produces: z.array(z.string()),     // output place IDs — tokens added
  guard: z.string().optional(),      // expression evaluated against marking

  // -- Semantic (what the agent cares about) --
  intent: z.string(),                // natural language goal
  context_sources: z.array(z.string()), // keys to look up in context store
  postconditions: PostconditionSchema,

  // -- Capabilities (what the agent can use) --
  available_tools: z.array(z.string()),
  
  // -- Execution mode --
  mode: z.enum(["deterministic", "judgment", "agentic"]),
});

export const NetSchema = z.object({
  id: z.string(),
  places: z.array(PlaceSchema),
  transitions: z.array(TransitionSchema),
});

/** A marking is a map from place ID → token count + optional payload */
export const TokenSchema = z.object({
  count: z.number().default(1),
  payload: z.record(z.unknown()).optional(),
});

export type Place = z.infer<typeof PlaceSchema>;
export type Transition = z.infer<typeof TransitionSchema>;
export type Net = z.infer<typeof NetSchema>;
export type Token = z.infer<typeof TokenSchema>;
export type Marking = Map<string, Token>;
```

### Key decision: tokens carry payloads

This is crucial. When `verify-vendor` fires and produces a token in `vendor-verified`, that token should carry the vendor data. The next transition (`assess-risk`) reads that payload as context. This is how data flows through the net without a separate data bus.

---

## Phase 2: Define the test net (30 min)

### `src/net/vendor-onboarding.ts`

Define the four transitions with real intent descriptions and postconditions. This is the part you'll iterate on most — the quality of these definitions directly determines whether the agent can execute them.

Write the intent strings as if you're explaining to a new hire what this step accomplishes institutionally. Not "call the vendor API" but "verify that the vendor exists as a registered entity, confirm their business registration is active, and retrieve their basic profile."

Write postconditions as observable facts, not implementation details. Not "API returned 200" but "vendor identity has been confirmed with registration number" and "vendor profile data is available for subsequent review."

---

## Phase 3: Minimal engine (1-2 hours)

### `src/net/engine.ts`

The engine does four things in a loop:

```
1. Find enabled transitions (all input places have tokens, guard passes)
2. Pick one (for now: first enabled, later: priority/policy)
3. Hand it to the agent executor
4. Check postconditions:
   - If met: consume input tokens, produce output tokens (with payload)
   - If not met: log failure, check escalation paths
5. Repeat until no transitions are enabled or target marking is reached
```

Keep this dead simple. No concurrency, no persistence, no optimization. A `while` loop with a `Map`. The engine's job in this spike is to be the **frame** that makes agent behavior observable — you want to see exactly what the agent tried, what postconditions it satisfied, and where it got stuck.

Add structured logging at every step — this is your primary observation tool.

```typescript
interface ExecutionLog {
  transitionId: string;
  markingBefore: Marking;
  agentActions: AgentAction[];    // what the agent actually did
  postconditionResults: Record<string, boolean>;
  markingAfter: Marking;
  status: "fired" | "failed" | "escalated";
  durationMs: number;
}
```

---

## Phase 4: Agent executor (2-3 hours)

This is the core of the spike. The executor receives a transition definition and returns an execution result.

### `src/agent/executor.ts`

Use Vercel AI SDK (`ai` package) with `generateText` + tools.

```typescript
import { generateText } from "ai";
import { anthropic } from "@ai-sdk/anthropic"; // or openai

async function executeTransition(
  transition: Transition,
  context: Record<string, unknown>,
  tools: Record<string, Tool>,
): Promise<ExecutionResult> {
  
  const systemPrompt = buildSystemPrompt(transition);
  const contextPrompt = buildContextPrompt(transition, context);
  
  const result = await generateText({
    model: anthropic("claude-sonnet-4-20250514"),
    system: systemPrompt,
    prompt: contextPrompt,
    tools,
    maxSteps: 10,  // let it loop over tools
  });
  
  return {
    actions: result.steps,       // what tools it called
    text: result.text,           // its reasoning/output
    toolResults: result.toolResults,
  };
}
```

### The system prompt is where the design lives

This is the most important part to get right. The system prompt needs to:

1. Explain what a transition is (you're executing one step in an institutional process)
2. Present the intent (what you're trying to accomplish)
3. Present the postconditions (what must be true when you're done)
4. Present available tools (what you can use)
5. Instruct the agent to reason about *which* tools to use and in *what order* to satisfy the postconditions
6. Ask for a structured report of what it did and what postconditions it believes it satisfied

Draft something like:

```
You are executing a single transition in an institutional workflow.

## Your task
{transition.intent}

## Context
{formatted context from input place payloads}

## Success criteria
The following must be true when you are done:
{transition.postconditions.required — as a checklist}

The following are desirable but not required:
{transition.postconditions.desired}

## Available tools
You have access to: {transition.available_tools}

## Instructions
1. Review the context and success criteria
2. Decide which tools to use and in what order
3. Execute your plan
4. Report which success criteria you satisfied and provide evidence
```

### Start with 2-3 real tools

Don't use MCP yet — just define AI SDK tools directly. Keep it simple:

- `lookup-vendor`: takes a vendor name, returns mock data from `fixtures/vendor-acme.json`
- `send-notification`: takes recipient + message, writes to console (or actually sends via an API if you want)
- `generate-summary`: takes data + instructions, returns formatted text (can be an LLM sub-call or template)

---

## Phase 5: Postcondition verification (1 hour)

### `src/agent/postconditions.ts`

This is the second key design question: **how do you verify postconditions?**

Three approaches to test, simplest first:

**A. Agent self-report (baseline).** Ask the agent to report which postconditions it satisfied. Parse its structured output. Cheapest, least reliable — the agent may hallucinate success.

**B. Tool-result inspection.** Check the actual tool results. If `send-notification` was called and returned success, "notification-sent" is satisfied. This is deterministic but only works for tool-observable postconditions.

**C. LLM-as-judge.** A second LLM call reviews the agent's actions and tool results against the postconditions. More reliable than self-report, more flexible than tool-result inspection.

**For the spike: implement B first, fall back to C.** Map postcondition strings to verification functions where possible, use LLM-as-judge for the rest.

```typescript
const postconditionVerifiers: Record<string, (result: ExecutionResult) => boolean> = {
  "notification-sent": (r) => r.toolResults.some(
    t => t.toolName === "send-notification" && t.result.success
  ),
  "vendor-identity-confirmed": (r) => r.toolResults.some(
    t => t.toolName === "lookup-vendor" && t.result.registrationStatus === "active"
  ),
  // Fall back to LLM-as-judge for anything not in this map
};
```

---

## Phase 6: Run it and observe (1-2 hours)

### `src/index.ts`

Wire it all together. Load the net, set the initial marking (`request-submitted` has a token with the vendor request payload), and run the engine loop.

```
$ npx tsx src/index.ts --vendor acme --verbose
```

### What to observe

Run it 5-10 times and log everything. You're looking for:

1. **Does the agent understand the intent?** Read its reasoning. Is it doing what you'd expect a competent new hire to do?
2. **Does it use the right tools?** For `verify-vendor`, does it call `lookup-vendor` without being told the specific function name — just from the intent?
3. **Are postconditions actually verified?** When the agent claims success, is it right?
4. **What happens when it fails?** Break something (make vendor lookup return nothing) and see if the net handles it correctly.
5. **Is the transition definition too vague or too specific?** This is the granularity question. You'll feel it when the agent flounders (too vague) or when you're basically writing pseudocode in the intent (too specific).

---

## Phase 7: Iterate on the boundary (ongoing)

Based on what you observe, adjust:

- **Transition definitions**: tune the intent language, add/remove postconditions
- **Context assembly**: does the agent get enough context? Too much?
- **Tool granularity**: are the tools at the right level of abstraction?
- **Postcondition verification**: where does self-report fail? Where is LLM-as-judge needed?

This is the actual research. The code is just the test harness.

---

## Dependencies

```json
{
  "dependencies": {
    "ai": "^4.0.0",
    "@ai-sdk/anthropic": "^1.0.0",
    "zod": "^3.23.0"
  },
  "devDependencies": {
    "tsx": "^4.0.0",
    "typescript": "^5.5.0"
  }
}
```

## Later (not in this spike)

- Evolve the spike engine into the production Petri net core
- Add MCP tools instead of hardcoded AI SDK tools
- Concurrent transition firing
- Persistent marking state
- Policy documents as context sources (RAG)
- Transition definition generation from policy docs
- Multiple scenarios beyond vendor onboarding