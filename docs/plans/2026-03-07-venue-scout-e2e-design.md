# Venue Scout — E2E Test Scenario Design

## Overview

An end-to-end test scenario that exercises the full agentic workflow loop: user input → agent search/reasoning → human-in-the-loop decision → dynamic notification. Built as a minimal Petri net that can run with mocks in CI (`vitest`) or live with real LLM, web search, human input, and Discord notifications (`tsx`).

## Scenario Narrative

A user wants to throw an event. They describe it (type, vibe, headcount, budget, location area, date). The workflow searches the web for matching venues, gathers details and contact info, proposes a shortlist with draft outreach emails, and — after human review and selection — sends the approved emails as Discord notifications.

---

## Petri Net

```
[event-submitted] → (research-venues) → [proposals-ready] → (review-proposals) → [outreach-approved] → (send-outreach) → [outreach-sent]
```

### Places

| Place | Description |
|-------|-------------|
| `event-submitted` | User has described their event — initial token |
| `proposals-ready` | Agent has found venues + drafted outreach emails |
| `outreach-approved` | Human has reviewed, selected venues, edited emails |
| `outreach-sent` | Notification(s) delivered |

### Transitions

| Transition | Mode | Authority | What happens |
|------------|------|-----------|-------------|
| `research-venues` | **agentic** | 1 | Searches the web for venues matching the event criteria. For each viable venue: name, why it fits, capacity, price range, contact info, website. Drafts a personalized outreach email per venue. |
| `review-proposals` | **judgment** | 1 | Human sees the shortlist + draft emails. Can approve all, drop venues, or edit email text. Returns the final set of approved venues with final email text. |
| `send-outreach` | **deterministic** | 1 | For each approved venue, sends the final email text as a notification via `NotificationSender`. |

### Policies

| Scope | Strength | Text |
|-------|----------|------|
| `venue-scout.*` | procedure | Each proposed venue must include: name, why it fits the event, estimated capacity, price range, contact information, and website link. |
| `venue-scout.research-venues` | preference | Prefer venues that have outdoor space when the event description mentions outdoor or nature. |
| `venue-scout.research-venues` | preference | Draft outreach emails should be warm and specific — reference the event details and explain why this venue caught your attention. |
| `venue-scout.send-outreach` | constraint | Only send outreach to venues explicitly approved by the human reviewer. |

---

## Token Payloads

### `event-submitted` (initial — from user input)

```typescript
interface EventSubmission {
  event_type: string;        // "birthday party", "corporate retreat", etc.
  vibe: string;              // "intimate, candlelit, jazz"
  headcount: number;         // 40
  budget: string;            // "$2000-5000"
  location_area: string;     // "Austin, TX"
  preferred_date: string;    // "2026-04-15"
  special_requirements: string; // "outdoor space preferred, needs AV for speeches"
}
```

### `proposals-ready` (after agent research)

```typescript
interface VenueProposal {
  name: string;
  why: string;               // Why it fits the event
  capacity: string;          // "30-80"
  price_range: string;       // "$3000-6000"
  contact_email: string;
  website: string;
  draft_email: string;       // Personalized outreach email text
}

interface ProposalsPayload {
  venues: VenueProposal[];
  search_summary: string;    // "Found 8 venues, shortlisted 3 based on..."
}
```

### `outreach-approved` (after human review)

```typescript
interface ApprovedVenue {
  name: string;
  final_email: string;       // Possibly edited by human
  contact_email: string;
}

interface OutreachApprovedPayload {
  approved_venues: ApprovedVenue[];
  reviewer_notes?: string;
}
```

### `outreach-sent` (after notifications)

```typescript
interface OutreachSentPayload {
  sent: Array<{
    venue_name: string;
    contact_email: string;
    notification_id: string;
    sent_at: string;
  }>;
}
```

---

## Pluggable Interfaces

### NotificationSender

Structural notifications — called by the workflow harness when the `send-outreach` transition fires.

```typescript
interface NotificationMessage {
  recipient: string;          // channel/contact identifier
  subject: string;
  body: string;
  metadata?: Record<string, unknown>;
}

interface NotificationResult {
  id: string;
  sent_at: string;
  success: boolean;
  error?: string;
}

interface NotificationSender {
  send(message: NotificationMessage): Promise<NotificationResult>;
}
```

**Implementations:**
- `DiscordWebhookSender` — POSTs to a Discord webhook URL. The `recipient` field is ignored (webhook is pre-configured to a channel). Subject becomes the embed title, body becomes embed description.
- `MockNotificationSender` — Captures messages in an array for assertions. Returns canned success results.

The interface is designed so it can later be wrapped as an `AgentTool` for agent-initiated notifications without changing the contract.

### HumanInput

Human-in-the-loop for judgment transitions.

```typescript
interface HumanPromptContext {
  transition_id: string;
  intent: string;
  token_payloads: Record<string, unknown>[];
  policies: Policy[];
}

interface HumanDecision {
  decision: Record<string, unknown>;  // The output payload for the judgment
  reasoning?: string;
}

interface HumanInput {
  prompt(context: HumanPromptContext): Promise<HumanDecision>;
}
```

**Implementations:**
- `ReadlineHumanInput` — Prints a formatted summary of the proposals to stdout, prompts via `readline` for venue selection and email edits.
- `MockHumanInput` — Returns a canned decision (approve all venues, no edits).

---

## Agent Tools

The agentic `research-venues` transition uses tools that wrap the existing Brave Search extension scripts (`~/.my-pi/skills/brave-search/`). This reuses proven search infrastructure and means improvements to the extension benefit all consumers.

### `web-search`

Wraps `search.js`.

```typescript
const webSearchTool: AgentTool = {
  name: "web-search",
  label: "Web Search",
  description: "Search the web for information. Returns titles, links, snippets, and optionally full page content as markdown.",
  parameters: Type.Object({
    query: Type.String({ description: "Search query" }),
    num_results: Type.Optional(Type.Number({ description: "Number of results, default 5, max 20" })),
    include_content: Type.Optional(Type.Boolean({ description: "Fetch full page content as markdown" })),
  }),
  execute: async (toolCallId, params) => {
    const args = [SEARCH_SCRIPT_PATH, params.query];
    if (params.num_results) args.push("-n", String(params.num_results));
    if (params.include_content) args.push("--content");
    const { stdout } = await execFile("node", args);
    return {
      content: [{ type: "text", text: stdout }],
      details: { raw: stdout },
    };
  },
};
```

### `fetch-page-content`

Wraps `content.js`.

```typescript
const fetchPageContentTool: AgentTool = {
  name: "fetch-page-content",
  label: "Fetch Page Content",
  description: "Fetch a URL and extract readable content as markdown. Use to get detailed info from a venue's website.",
  parameters: Type.Object({
    url: Type.String({ description: "URL to fetch" }),
  }),
  execute: async (toolCallId, params) => {
    const { stdout } = await execFile("node", [CONTENT_SCRIPT_PATH, params.url]);
    return {
      content: [{ type: "text", text: stdout }],
      details: { raw: stdout },
    };
  },
};
```

### Tool script path

The path to the brave-search scripts is configurable. Default: `~/.my-pi/skills/brave-search/`. The live runner resolves this at startup; mock tests don't use real tools.

---

## Bridging AgentRunner → pi-agent-core

The existing `TransitionExecutor` signature:

```typescript
type TransitionExecutor = (
  transitionId: string,
  systemPrompt: string,
  contextPrompt: string,
  tools: unknown[],
) => Promise<ExecutionEvidence>;
```

### Live executor (for `run.ts`)

Creates a pi-agent-core `Agent`, sends the context prompt, collects tool calls, extracts structured output:

```typescript
async function piAgentExecutor(
  transitionId: string,
  systemPrompt: string,
  contextPrompt: string,
  tools: AgentTool[],
): Promise<ExecutionEvidence> {
  const agent = new Agent({
    initialState: {
      systemPrompt,
      model: getModel("anthropic", "claude-sonnet-4-20250514"),
      tools,
      thinkingLevel: "low",
    },
  });

  const toolResults: Array<{ toolName: string; result: Record<string, unknown> }> = [];

  agent.subscribe((event) => {
    if (event.type === "tool_execution_end" && !event.isError) {
      toolResults.push({ toolName: event.toolName, result: event.result });
    }
  });

  await agent.prompt(contextPrompt);

  // Extract the final assistant message
  const messages = agent.state.messages;
  const lastAssistant = [...messages].reverse().find(
    (m) => "role" in m && m.role === "assistant"
  );
  const text = extractText(lastAssistant);
  const payload = extractJsonPayload(text);

  return { text, toolResults, payload };
}
```

### Mock executor (for `vitest`)

Returns canned `ExecutionEvidence` with pre-built venue data. Tools are never invoked.

---

## Execution Flow

### Live (`tsx src/e2e/run.ts`)

```
1. User types event description into terminal prompt
2. Engine instantiates net with event payload as initial token in "event-submitted"
3. AgentRunner.step() picks up "research-venues" (agentic)
   → pi-agent-core Agent calls web-search, fetch-page-content
   → Agent produces venue shortlist + draft emails
   → Postconditions verified, transition fires
   → Token moves to "proposals-ready"
4. Runner stops (no more agentic transitions)
5. Harness detects pending judgment "review-proposals"
   → ReadlineHumanInput shows venues + emails, asks for selection/edits
   → resolveJudgment() fires transition
   → Token moves to "outreach-approved"
6. Harness fires "send-outreach" (deterministic)
   → Reads approved venues from token payload
   → Calls DiscordWebhookSender.send() for each venue
   → Token moves to "outreach-sent"
7. Print audit trail summary
```

### CI (`vitest`)

```
1. Instantiate net with fixture event payload
2. AgentRunner.run() with mock executor → fires "research-venues" with canned venue data
3. Assert: token at "proposals-ready", correct payload shape
4. Resolve judgment with MockHumanInput → fires "review-proposals"
5. Assert: token at "outreach-approved"
6. Fire "send-outreach" with MockNotificationSender
7. Assert: token at "outreach-sent", mock captured correct messages
8. Assert: audit trail complete, hash chain valid
9. Assert: notification messages contain venue names + personalized email text
```

---

## File Structure

```
typescript/src/e2e/
  types.ts                    — NotificationSender, HumanInput interfaces, payload types
  tools.ts                    — web-search and fetch-page-content AgentTool wrappers
  venue-scout-net.ts          — Builds the Petri net (places, transitions, policies)
  venue-scout-harness.ts      — Orchestration loop: run agent → handle judgment → fire notifications
  venue-scout.test.ts         — vitest test with all mocks (CI-safe)
  run.ts                      — Live demo script (tsx), real LLM + search + readline + Discord
  discord-webhook.ts          — DiscordWebhookSender implementation
  readline-input.ts           — ReadlineHumanInput implementation
```

---

## Notification Architecture Decision

**For now:** Notifications are structural — part of the workflow definition. The `send-outreach` transition fires deterministically, and the harness calls `NotificationSender` as a side effect. This is correct for this scenario: the institution decided that approved outreach gets sent.

**Later:** The `NotificationSender` interface can be wrapped as an `AgentTool` for agent-initiated notifications during agentic transitions (e.g., "I can't find contact info, pinging a human for help"). The interface is the same; only the caller changes.

---

## Dependencies

- `@mariozechner/pi-agent-core` — Agent class, AgentTool types (already in package.json)
- `@mariozechner/pi-ai` — getModel, Model types (already in package.json)
- `@sinclair/typebox` — Tool parameter schemas (transitive dep of pi-ai)
- Brave Search extension scripts — shelled out via `execFile`, no new npm deps
- Discord webhook — plain `fetch` POST, no SDK needed
