# Venue Scout E2E Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build an end-to-end test scenario that exercises user input → agentic web search → human-in-the-loop judgment → dynamic Discord notification, runnable as both a CI test (mocked) and a live demo.

**Architecture:** New `src/e2e/` module with pluggable interfaces for notifications and human input. The Petri net is a minimal 4-place, 3-transition workflow. Agent tools wrap the existing Brave Search extension scripts via `child_process.execFile`. The live runner uses pi-agent-core's `Agent` class; the CI test uses mock executors.

**Tech Stack:** Engine (existing), AgentRunner (existing), pi-agent-core Agent, pi-ai getModel, @sinclair/typebox for tool schemas, Brave Search CLI scripts, Discord webhooks via fetch, Node readline.

**Design doc:** `docs/plans/2026-03-07-venue-scout-e2e-design.md`

---

### Task 1: E2E Types — Interfaces and Payload Types

**Files:**
- Create: `typescript/src/e2e/types.ts`

**Step 1: Create the types file**

```typescript
import type { Policy } from "../core/types.js";

// ---------------------------------------------------------------------------
// Token payloads
// ---------------------------------------------------------------------------

export interface EventSubmission {
  event_type: string;
  vibe: string;
  headcount: number;
  budget: string;
  location_area: string;
  preferred_date: string;
  special_requirements: string;
}

export interface VenueProposal {
  name: string;
  why: string;
  capacity: string;
  price_range: string;
  contact_email: string;
  website: string;
  draft_email: string;
}

export interface ProposalsPayload {
  venues: VenueProposal[];
  search_summary: string;
}

export interface ApprovedVenue {
  name: string;
  final_email: string;
  contact_email: string;
}

export interface OutreachApprovedPayload {
  approved_venues: ApprovedVenue[];
  reviewer_notes?: string | undefined;
}

export interface OutreachSentPayload {
  sent: Array<{
    venue_name: string;
    contact_email: string;
    notification_id: string;
    sent_at: string;
  }>;
}

// ---------------------------------------------------------------------------
// NotificationSender
// ---------------------------------------------------------------------------

export interface NotificationMessage {
  recipient: string;
  subject: string;
  body: string;
  metadata?: Record<string, unknown> | undefined;
}

export interface NotificationResult {
  id: string;
  sent_at: string;
  success: boolean;
  error?: string | undefined;
}

export interface NotificationSender {
  send(message: NotificationMessage): Promise<NotificationResult>;
}

// ---------------------------------------------------------------------------
// HumanInput
// ---------------------------------------------------------------------------

export interface HumanPromptContext {
  transition_id: string;
  intent: string;
  token_payloads: Record<string, unknown>[];
  policies: Policy[];
}

export interface HumanDecision {
  decision: Record<string, unknown>;
  reasoning?: string | undefined;
}

export interface HumanInput {
  prompt(context: HumanPromptContext): Promise<HumanDecision>;
}
```

**Step 2: Typecheck**

Run: `cd typescript && npx tsc --noEmit`
Expected: No errors.

**Step 3: Commit**

```bash
git add typescript/src/e2e/types.ts
git commit -m "feat(e2e): notification, human input, and payload type definitions"
```

---

### Task 2: Mock Implementations — MockNotificationSender and MockHumanInput

**Files:**
- Create: `typescript/src/e2e/mocks.ts`
- Create: `typescript/src/e2e/mocks.test.ts`

**Step 1: Write the failing tests**

```typescript
import { describe, it, expect } from "vitest";
import { MockNotificationSender, MockHumanInput } from "./mocks.js";
import type { HumanPromptContext, OutreachApprovedPayload } from "./types.js";

describe("MockNotificationSender", () => {
  it("captures sent messages and returns success", async () => {
    const sender = new MockNotificationSender();
    const result = await sender.send({
      recipient: "events@driskill.com",
      subject: "Event inquiry",
      body: "Dear Driskill team...",
    });

    expect(result.success).toBe(true);
    expect(result.id).toBeTruthy();
    expect(sender.messages).toHaveLength(1);
    expect(sender.messages[0]!.recipient).toBe("events@driskill.com");
    expect(sender.messages[0]!.body).toContain("Driskill");
  });

  it("captures multiple messages", async () => {
    const sender = new MockNotificationSender();
    await sender.send({ recipient: "a@b.com", subject: "S1", body: "B1" });
    await sender.send({ recipient: "c@d.com", subject: "S2", body: "B2" });
    expect(sender.messages).toHaveLength(2);
  });
});

describe("MockHumanInput", () => {
  it("returns the canned decision", async () => {
    const cannedDecision = {
      decision: {
        approved_venues: [
          { name: "Venue A", final_email: "Dear A...", contact_email: "a@a.com" },
        ],
      } satisfies Record<string, unknown>,
      reasoning: "Looks good",
    };
    const human = new MockHumanInput(cannedDecision);

    const context: HumanPromptContext = {
      transition_id: "review-proposals",
      intent: "Review venues",
      token_payloads: [{ venues: [] }],
      policies: [],
    };

    const result = await human.prompt(context);
    expect(result.decision).toEqual(cannedDecision.decision);
    expect(result.reasoning).toBe("Looks good");
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `cd typescript && npx vitest run src/e2e/mocks.test.ts`
Expected: FAIL — cannot find `./mocks.js`

**Step 3: Write the implementations**

```typescript
import { randomUUID } from "node:crypto";
import type {
  NotificationMessage,
  NotificationResult,
  NotificationSender,
  HumanPromptContext,
  HumanDecision,
  HumanInput,
} from "./types.js";

export class MockNotificationSender implements NotificationSender {
  readonly messages: NotificationMessage[] = [];

  async send(message: NotificationMessage): Promise<NotificationResult> {
    this.messages.push(message);
    return {
      id: randomUUID(),
      sent_at: new Date().toISOString(),
      success: true,
    };
  }
}

export class MockHumanInput implements HumanInput {
  constructor(private readonly cannedDecision: HumanDecision) {}

  async prompt(_context: HumanPromptContext): Promise<HumanDecision> {
    return this.cannedDecision;
  }
}
```

**Step 4: Run tests to verify they pass**

Run: `cd typescript && npx vitest run src/e2e/mocks.test.ts`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add typescript/src/e2e/mocks.ts typescript/src/e2e/mocks.test.ts
git commit -m "feat(e2e): mock notification sender and human input"
```

---

### Task 3: Venue Scout Net Builder

**Files:**
- Create: `typescript/src/e2e/venue-scout-net.ts`
- Create: `typescript/src/e2e/venue-scout-net.test.ts`

**Step 1: Write the failing test**

```typescript
import { describe, it, expect, afterEach } from "vitest";
import { Engine } from "../core/engine.js";
import { validateNet } from "../core/validate.js";
import { buildVenueScoutNet } from "./venue-scout-net.js";

describe("buildVenueScoutNet", () => {
  let engine: Engine;

  afterEach(() => {
    engine.close();
  });

  it("creates a valid net with 4 places and 3 transitions", () => {
    engine = new Engine(":memory:");
    const { netId, institutionId, roles, actors } = buildVenueScoutNet(engine);

    const { places, transitions } = engine.getNetWithGraph(netId);
    expect(places).toHaveLength(4);
    expect(transitions).toHaveLength(3);

    // Check place IDs
    const placeIds = places.map((p) => p.id);
    expect(placeIds).toContain("event-submitted");
    expect(placeIds).toContain("proposals-ready");
    expect(placeIds).toContain("outreach-approved");
    expect(placeIds).toContain("outreach-sent");

    // Check transition IDs and modes
    const research = transitions.find((t) => t.id === "research-venues");
    expect(research?.mode).toBe("agentic");
    expect(research?.consumes).toEqual(["event-submitted"]);
    expect(research?.produces).toEqual(["proposals-ready"]);

    const review = transitions.find((t) => t.id === "review-proposals");
    expect(review?.mode).toBe("judgment");

    const send = transitions.find((t) => t.id === "send-outreach");
    expect(send?.mode).toBe("deterministic");

    // Validate net structure
    const validation = validateNet(engine, netId);
    expect(validation.is_valid).toBe(true);
  });

  it("returns actor IDs for the scout agent and human reviewer", () => {
    engine = new Engine(":memory:");
    const { actors } = buildVenueScoutNet(engine);

    expect(actors.scout).toBeTruthy();
    expect(actors.reviewer).toBeTruthy();
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `cd typescript && npx vitest run src/e2e/venue-scout-net.test.ts`
Expected: FAIL — cannot find `./venue-scout-net.js`

**Step 3: Write the net builder**

```typescript
import type { Engine } from "../core/engine.js";

export interface VenueScoutNet {
  institutionId: string;
  netId: string;
  roles: { scout: string; reviewer: string };
  actors: { scout: string; reviewer: string };
}

export function buildVenueScoutNet(engine: Engine): VenueScoutNet {
  const inst = engine.createInstitution("Venue Scout", "Event venue research service");

  const scoutRole = engine.createRole(inst.id, "scout-agent", 1, "AI agent that researches venues");
  const reviewerRole = engine.createRole(inst.id, "reviewer", 2, "Human who reviews and approves outreach");

  const scout = engine.createActor(inst.id, "Scout Agent", "agent");
  engine.assignRole(scout.id, scoutRole.id);

  const reviewer = engine.createActor(inst.id, "Human Reviewer", "human");
  engine.assignRole(reviewer.id, reviewerRole.id);

  const net = engine.createNet(inst.id, "Venue Scout", "venue-scout", "Find and contact event venues");
  const netId = net.id;

  // Places
  engine.addPlace(netId, "event-submitted", "User has described their event — initial token");
  engine.addPlace(netId, "proposals-ready", "Agent has found venues and drafted outreach emails");
  engine.addPlace(netId, "outreach-approved", "Human has reviewed and approved venues to contact");
  engine.addPlace(netId, "outreach-sent", "Outreach notifications delivered");

  // Transitions
  engine.addTransition(netId, {
    id: "research-venues",
    consumes: ["event-submitted"],
    produces: ["proposals-ready"],
    intent: "Search the web for event venues matching the user's criteria. For each viable venue, gather: name, why it fits the event, capacity, price range, contact email, and website. Draft a personalized outreach email for each venue.",
    mode: "agentic",
    requires_authority: 1,
    context_sources: [],
    postconditions: {
      required: ["venues-found", "emails-drafted"],
      desired: ["at-least-3-venues"],
    },
    evidence_requirements: [
      { id: "search-queries", description: "Web search queries used", type: "artifact", required: true },
      { id: "venue-list", description: "Complete list of venues considered", type: "artifact", required: true },
    ],
    available_tools: ["web-search", "fetch-page-content"],
    output_schema: {
      type: "object",
      properties: {
        venues: {
          type: "array",
          items: {
            type: "object",
            properties: {
              name: { type: "string" },
              why: { type: "string" },
              capacity: { type: "string" },
              price_range: { type: "string" },
              contact_email: { type: "string" },
              website: { type: "string" },
              draft_email: { type: "string" },
            },
          },
        },
        search_summary: { type: "string" },
      },
    },
  });

  engine.addTransition(netId, {
    id: "review-proposals",
    consumes: ["proposals-ready"],
    produces: ["outreach-approved"],
    intent: "Review the proposed venues and draft outreach emails. Select which venues to contact, edit email text if needed.",
    mode: "judgment",
    decision_type: "approval",
    requires_authority: 1,
    context_sources: [],
    postconditions: { required: ["venues-selected"] },
    evidence_requirements: [],
    available_tools: [],
    output_schema: {
      type: "object",
      properties: {
        approved_venues: {
          type: "array",
          items: {
            type: "object",
            properties: {
              name: { type: "string" },
              final_email: { type: "string" },
              contact_email: { type: "string" },
            },
          },
        },
        reviewer_notes: { type: "string" },
      },
    },
  });

  engine.addTransition(netId, {
    id: "send-outreach",
    consumes: ["outreach-approved"],
    produces: ["outreach-sent"],
    intent: "Send the approved outreach emails as notifications to each selected venue.",
    mode: "deterministic",
    requires_authority: 1,
    context_sources: [],
    postconditions: { required: ["notifications-sent"] },
    evidence_requirements: [
      { id: "delivery-confirmations", description: "Confirmation of each notification sent", type: "artifact", required: true },
    ],
    available_tools: [],
  });

  // Policies
  engine.attachPolicy(inst.id, "venue-scout.*", "procedure",
    "Each proposed venue must include: name, why it fits the event, estimated capacity, price range, contact information, and website link.");
  engine.attachPolicy(inst.id, "venue-scout.research-venues", "preference",
    "Prefer venues that have outdoor space when the event description mentions outdoor or nature.");
  engine.attachPolicy(inst.id, "venue-scout.research-venues", "preference",
    "Draft outreach emails should be warm and specific — reference the event details and explain why this venue caught your attention.");
  engine.attachPolicy(inst.id, "venue-scout.send-outreach", "constraint",
    "Only send outreach to venues explicitly approved by the human reviewer.");

  return {
    institutionId: inst.id,
    netId,
    roles: { scout: scoutRole.id, reviewer: reviewerRole.id },
    actors: { scout: scout.id, reviewer: reviewer.id },
  };
}
```

**Step 4: Run tests to verify they pass**

Run: `cd typescript && npx vitest run src/e2e/venue-scout-net.test.ts`
Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add typescript/src/e2e/venue-scout-net.ts typescript/src/e2e/venue-scout-net.test.ts
git commit -m "feat(e2e): venue scout Petri net builder"
```

---

### Task 4: Agent Tools — web-search and fetch-page-content

**Files:**
- Create: `typescript/src/e2e/tools.ts`
- Create: `typescript/src/e2e/tools.test.ts`

**Step 1: Write the failing tests**

The tools shell out to external scripts, so the CI test verifies tool definition shape and mocks `execFile`. We test the real scripts exist but don't call Brave API in CI.

```typescript
import { describe, it, expect } from "vitest";
import { createWebSearchTool, createFetchPageContentTool } from "./tools.js";

describe("createWebSearchTool", () => {
  it("returns an AgentTool with correct name and parameters", () => {
    const tool = createWebSearchTool("/fake/path/search.js");
    expect(tool.name).toBe("web-search");
    expect(tool.label).toBe("Web Search");
    expect(tool.parameters).toBeDefined();
    expect(tool.execute).toBeTypeOf("function");
  });
});

describe("createFetchPageContentTool", () => {
  it("returns an AgentTool with correct name and parameters", () => {
    const tool = createFetchPageContentTool("/fake/path/content.js");
    expect(tool.name).toBe("fetch-page-content");
    expect(tool.label).toBe("Fetch Page Content");
    expect(tool.parameters).toBeDefined();
    expect(tool.execute).toBeTypeOf("function");
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `cd typescript && npx vitest run src/e2e/tools.test.ts`
Expected: FAIL — cannot find `./tools.js`

**Step 3: Write the tool factories**

```typescript
import { execFile as execFileCb } from "node:child_process";
import { promisify } from "node:util";
import { Type } from "@mariozechner/pi-ai";
import type { AgentTool } from "@mariozechner/pi-agent-core";

const execFile = promisify(execFileCb);

export function createWebSearchTool(searchScriptPath: string): AgentTool {
  return {
    name: "web-search",
    label: "Web Search",
    description:
      "Search the web for information. Returns titles, links, snippets, and optionally full page content as markdown.",
    parameters: Type.Object({
      query: Type.String({ description: "Search query" }),
      num_results: Type.Optional(
        Type.Number({ description: "Number of results, default 5, max 20" }),
      ),
      include_content: Type.Optional(
        Type.Boolean({ description: "Fetch full page content as markdown" }),
      ),
    }),
    execute: async (_toolCallId, params) => {
      const args = [searchScriptPath, params.query];
      if (params.num_results) args.push("-n", String(params.num_results));
      if (params.include_content) args.push("--content");
      const { stdout } = await execFile("node", args, {
        env: { ...process.env },
        timeout: 30_000,
      });
      return {
        content: [{ type: "text" as const, text: stdout }],
        details: { raw: stdout },
      };
    },
  };
}

export function createFetchPageContentTool(contentScriptPath: string): AgentTool {
  return {
    name: "fetch-page-content",
    label: "Fetch Page Content",
    description:
      "Fetch a URL and extract readable content as markdown. Use to get detailed info from a venue's website.",
    parameters: Type.Object({
      url: Type.String({ description: "URL to fetch" }),
    }),
    execute: async (_toolCallId, params) => {
      const { stdout } = await execFile("node", [contentScriptPath, params.url], {
        env: { ...process.env },
        timeout: 15_000,
      });
      return {
        content: [{ type: "text" as const, text: stdout }],
        details: { raw: stdout },
      };
    },
  };
}
```

**Step 4: Run tests to verify they pass**

Run: `cd typescript && npx vitest run src/e2e/tools.test.ts`
Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add typescript/src/e2e/tools.ts typescript/src/e2e/tools.test.ts
git commit -m "feat(e2e): web-search and fetch-page-content agent tool wrappers"
```

---

### Task 5: Venue Scout Harness — Orchestration Loop

**Files:**
- Create: `typescript/src/e2e/venue-scout-harness.ts`
- Create: `typescript/src/e2e/venue-scout-harness.test.ts`

The harness is the glue: it runs the AgentRunner for agentic steps, handles judgment transitions via `HumanInput`, and fires the deterministic notification transition with `NotificationSender`.

**Step 1: Write the failing test**

```typescript
import { describe, it, expect, afterEach } from "vitest";
import { Engine } from "../core/engine.js";
import { PostconditionVerifier } from "../agent/postconditions.js";
import { AgentRunner } from "../agent/runner.js";
import { buildVenueScoutNet } from "./venue-scout-net.js";
import { MockNotificationSender, MockHumanInput } from "./mocks.js";
import { runVenueScout } from "./venue-scout-harness.js";
import type { EventSubmission, ProposalsPayload, OutreachApprovedPayload } from "./types.js";

describe("runVenueScout", () => {
  let engine: Engine;

  afterEach(() => {
    engine.close();
  });

  it("runs the full workflow: agent research → human review → send notifications", async () => {
    engine = new Engine(":memory:");
    const net = buildVenueScoutNet(engine);

    const eventPayload: EventSubmission = {
      event_type: "birthday party",
      vibe: "intimate, candlelit, jazz",
      headcount: 40,
      budget: "$2000-5000",
      location_area: "Austin, TX",
      preferred_date: "2026-04-15",
      special_requirements: "outdoor space preferred",
    };

    const mockProposals: ProposalsPayload = {
      venues: [
        {
          name: "The Driskill Hotel",
          why: "Historic downtown venue with intimate event rooms",
          capacity: "30-80",
          price_range: "$3000-6000",
          contact_email: "events@driskill.com",
          website: "https://driskill.com",
          draft_email: "Dear Driskill Events Team,\n\nI'm planning an intimate birthday celebration...",
        },
        {
          name: "Hotel San José",
          why: "Boutique SoCo hotel with courtyard space",
          capacity: "20-50",
          price_range: "$2000-4000",
          contact_email: "events@sanjose.com",
          website: "https://sanjosehotel.com",
          draft_email: "Dear San José team,\n\nI'm looking for a warm, intimate space...",
        },
      ],
      search_summary: "Found 6 venues, shortlisted 2 based on capacity and vibe",
    };

    const approvedPayload: OutreachApprovedPayload = {
      approved_venues: [
        {
          name: "The Driskill Hotel",
          final_email: "Dear Driskill Events Team,\n\nI'm planning an intimate birthday celebration...",
          contact_email: "events@driskill.com",
        },
      ],
      reviewer_notes: "Dropped San José — over budget concerns",
    };

    // Mock executor returns canned venue data
    const executor = async () => ({
      text: "Found venues",
      toolResults: [{ toolName: "web-search", result: { raw: "search results" } }],
      payload: mockProposals as unknown as Record<string, unknown>,
    });

    const verifier = new PostconditionVerifier(
      new Map<string, (evidence: { text: string; toolResults: Array<{ toolName: string; result: Record<string, unknown> }>; payload: Record<string, unknown> }) => boolean>([
        ["venues-found", (e) => {
          const p = e.payload as unknown as ProposalsPayload;
          return Array.isArray(p.venues) && p.venues.length > 0;
        }],
        ["emails-drafted", (e) => {
          const p = e.payload as unknown as ProposalsPayload;
          return p.venues.every((v) => v.draft_email.length > 0);
        }],
        ["at-least-3-venues", () => false], // desired, not required — ok to fail
      ]),
    );

    const runner = new AgentRunner(engine, verifier, { executor });

    const mockHuman = new MockHumanInput({
      decision: approvedPayload as unknown as Record<string, unknown>,
      reasoning: "Dropped San José — over budget concerns",
    });

    const mockSender = new MockNotificationSender();

    const result = await runVenueScout({
      engine,
      runner,
      net,
      eventPayload: eventPayload as unknown as Record<string, unknown>,
      humanInput: mockHuman,
      notificationSender: mockSender,
    });

    // Workflow completed
    expect(result.success).toBe(true);

    // Token should be at outreach-sent
    const marking = engine.getMarking(result.instanceId);
    expect(marking.has("outreach-sent")).toBe(true);

    // Notification was sent for the one approved venue
    expect(mockSender.messages).toHaveLength(1);
    expect(mockSender.messages[0]!.recipient).toBe("events@driskill.com");
    expect(mockSender.messages[0]!.body).toContain("Driskill");

    // Audit trail is complete
    const history = engine.getHistory(result.instanceId);
    const fired = history.filter((e) => e.action === "transition_fired");
    expect(fired).toHaveLength(3); // research-venues, review-proposals, send-outreach
  });

  it("handles zero approved venues gracefully", async () => {
    engine = new Engine(":memory:");
    const net = buildVenueScoutNet(engine);

    const executor = async () => ({
      text: "Found venues",
      toolResults: [],
      payload: {
        venues: [{ name: "A", why: "x", capacity: "10", price_range: "$1k", contact_email: "a@a.com", website: "http://a.com", draft_email: "Hi" }],
        search_summary: "1 found",
      } as Record<string, unknown>,
    });

    const verifier = new PostconditionVerifier(
      new Map<string, () => boolean>([
        ["venues-found", () => true],
        ["emails-drafted", () => true],
      ]),
    );

    const runner = new AgentRunner(engine, verifier, { executor });

    const mockHuman = new MockHumanInput({
      decision: { approved_venues: [], reviewer_notes: "None suitable" },
    });

    const mockSender = new MockNotificationSender();

    const result = await runVenueScout({
      engine,
      runner,
      net,
      eventPayload: { event_type: "test", vibe: "test", headcount: 10, budget: "$100", location_area: "Test", preferred_date: "2026-01-01", special_requirements: "" } as unknown as Record<string, unknown>,
      humanInput: mockHuman,
      notificationSender: mockSender,
    });

    expect(result.success).toBe(true);
    expect(mockSender.messages).toHaveLength(0);
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `cd typescript && npx vitest run src/e2e/venue-scout-harness.test.ts`
Expected: FAIL — cannot find `./venue-scout-harness.js`

**Step 3: Write the harness**

```typescript
import { randomUUID } from "node:crypto";
import type { Engine } from "../core/engine.js";
import type { AgentRunner } from "../agent/runner.js";
import type { VenueScoutNet } from "./venue-scout-net.js";
import type {
  NotificationSender,
  HumanInput,
  OutreachApprovedPayload,
  OutreachSentPayload,
} from "./types.js";

export interface VenueScoutOptions {
  engine: Engine;
  runner: AgentRunner;
  net: VenueScoutNet;
  eventPayload: Record<string, unknown>;
  humanInput: HumanInput;
  notificationSender: NotificationSender;
}

export interface VenueScoutResult {
  success: boolean;
  instanceId: string;
  error?: string | undefined;
}

export async function runVenueScout(options: VenueScoutOptions): Promise<VenueScoutResult> {
  const { engine, runner, net, eventPayload, humanInput, notificationSender } = options;

  // 1. Instantiate workflow with user's event description
  const instance = engine.instantiate(net.netId, "event-submitted", eventPayload);

  // 2. Run agentic transitions (research-venues)
  const runResult = await runner.run(instance.id, net.actors.scout);

  if (runResult.final_outcome === "error") {
    return {
      success: false,
      instanceId: instance.id,
      error: `Agent runner error: ${runResult.steps.at(-1)?.error}`,
    };
  }

  if (runResult.final_outcome === "escalated") {
    return {
      success: false,
      instanceId: instance.id,
      error: "Agent escalated — postconditions not met",
    };
  }

  // 3. Handle judgment transition (review-proposals)
  const pending = engine.getPendingJudgments(instance.id);
  const reviewJudgment = pending.find((j) => j.transition_id === "review-proposals");

  if (!reviewJudgment) {
    return {
      success: false,
      instanceId: instance.id,
      error: "No pending review-proposals judgment found",
    };
  }

  const humanDecision = await humanInput.prompt({
    transition_id: reviewJudgment.transition_id,
    intent: reviewJudgment.transition_intent,
    token_payloads: reviewJudgment.token_payloads,
    policies: reviewJudgment.policies,
  });

  const judgmentResult = engine.resolveJudgment(
    instance.id,
    "review-proposals",
    net.actors.reviewer,
    humanDecision.decision,
    humanDecision.reasoning,
  );

  if (!judgmentResult.success) {
    return {
      success: false,
      instanceId: instance.id,
      error: `Judgment failed: ${judgmentResult.error}`,
    };
  }

  // 4. Fire deterministic send-outreach transition
  //    Read the approved venues from the token payload and send notifications
  const marking = engine.getMarking(instance.id);
  const approvedTokens = marking.get("outreach-approved");
  const approvedPayload = approvedTokens?.[0]?.payload as unknown as OutreachApprovedPayload | undefined;
  const approvedVenues = approvedPayload?.approved_venues ?? [];

  // Send notifications
  const sent: OutreachSentPayload["sent"] = [];

  for (const venue of approvedVenues) {
    const result = await notificationSender.send({
      recipient: venue.contact_email,
      subject: `Event venue inquiry — ${venue.name}`,
      body: venue.final_email,
    });

    sent.push({
      venue_name: venue.name,
      contact_email: venue.contact_email,
      notification_id: result.id,
      sent_at: result.sent_at,
    });
  }

  const outreachPayload: OutreachSentPayload = { sent };

  const fireResult = engine.fireTransition(
    instance.id,
    "send-outreach",
    net.actors.reviewer,
    outreachPayload as unknown as Record<string, unknown>,
  );

  if (!fireResult.success) {
    return {
      success: false,
      instanceId: instance.id,
      error: `send-outreach failed: ${fireResult.error}`,
    };
  }

  return { success: true, instanceId: instance.id };
}
```

**Step 4: Run tests to verify they pass**

Run: `cd typescript && npx vitest run src/e2e/venue-scout-harness.test.ts`
Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add typescript/src/e2e/venue-scout-harness.ts typescript/src/e2e/venue-scout-harness.test.ts
git commit -m "feat(e2e): venue scout orchestration harness"
```

---

### Task 6: Discord Webhook Sender

**Files:**
- Create: `typescript/src/e2e/discord-webhook.ts`
- Create: `typescript/src/e2e/discord-webhook.test.ts`

**Step 1: Write the failing test**

We test the message formatting logic, not the actual HTTP call. The real webhook is tested via the live runner.

```typescript
import { describe, it, expect } from "vitest";
import { DiscordWebhookSender, buildDiscordPayload } from "./discord-webhook.js";

describe("buildDiscordPayload", () => {
  it("formats a notification as a Discord embed", () => {
    const payload = buildDiscordPayload({
      recipient: "events@driskill.com",
      subject: "Event venue inquiry — The Driskill Hotel",
      body: "Dear Driskill Events Team,\n\nI'm planning an intimate birthday celebration for about 40 guests...",
    });

    expect(payload.embeds).toHaveLength(1);
    expect(payload.embeds[0]!.title).toBe("Event venue inquiry — The Driskill Hotel");
    expect(payload.embeds[0]!.description).toContain("intimate birthday");
    expect(payload.embeds[0]!.footer!.text).toContain("events@driskill.com");
  });
});

describe("DiscordWebhookSender", () => {
  it("throws if webhook URL is missing", () => {
    expect(() => new DiscordWebhookSender("")).toThrow("webhook URL");
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `cd typescript && npx vitest run src/e2e/discord-webhook.test.ts`
Expected: FAIL — cannot find `./discord-webhook.js`

**Step 3: Write the implementation**

```typescript
import { randomUUID } from "node:crypto";
import type { NotificationMessage, NotificationResult, NotificationSender } from "./types.js";

export interface DiscordEmbed {
  title: string;
  description: string;
  color?: number | undefined;
  footer?: { text: string } | undefined;
}

export interface DiscordWebhookPayload {
  embeds: DiscordEmbed[];
}

export function buildDiscordPayload(message: NotificationMessage): DiscordWebhookPayload {
  return {
    embeds: [
      {
        title: message.subject,
        description: message.body,
        color: 0x5865f2, // Discord blurple
        footer: { text: `To: ${message.recipient}` },
      },
    ],
  };
}

export class DiscordWebhookSender implements NotificationSender {
  constructor(private readonly webhookUrl: string) {
    if (!webhookUrl) {
      throw new Error("DiscordWebhookSender requires a webhook URL");
    }
  }

  async send(message: NotificationMessage): Promise<NotificationResult> {
    const payload = buildDiscordPayload(message);

    const response = await fetch(this.webhookUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    const now = new Date().toISOString();

    if (!response.ok) {
      const errorText = await response.text();
      return {
        id: randomUUID(),
        sent_at: now,
        success: false,
        error: `Discord webhook failed: HTTP ${response.status} — ${errorText}`,
      };
    }

    return {
      id: randomUUID(),
      sent_at: now,
      success: true,
    };
  }
}
```

**Step 4: Run tests to verify they pass**

Run: `cd typescript && npx vitest run src/e2e/discord-webhook.test.ts`
Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add typescript/src/e2e/discord-webhook.ts typescript/src/e2e/discord-webhook.test.ts
git commit -m "feat(e2e): Discord webhook notification sender"
```

---

### Task 7: Readline Human Input

**Files:**
- Create: `typescript/src/e2e/readline-input.ts`

No test for this — it's a terminal I/O adapter only used by the live runner. Testing would require mocking stdin, which adds complexity for little value.

**Step 1: Write the implementation**

```typescript
import * as readline from "node:readline/promises";
import { stdin, stdout } from "node:process";
import type { HumanPromptContext, HumanDecision, HumanInput } from "./types.js";
import type { ProposalsPayload, ApprovedVenue } from "./types.js";

export class ReadlineHumanInput implements HumanInput {
  async prompt(context: HumanPromptContext): Promise<HumanDecision> {
    const rl = readline.createInterface({ input: stdin, output: stdout });

    try {
      console.log("\n" + "=".repeat(60));
      console.log(`📋 JUDGMENT REQUIRED: ${context.intent}`);
      console.log("=".repeat(60));

      // Display proposals
      const proposals = context.token_payloads[0] as unknown as ProposalsPayload | undefined;
      const venues = proposals?.venues ?? [];

      if (venues.length === 0) {
        console.log("\nNo venues were found.");
        const notes = await rl.question("\nAny notes? (press Enter to skip): ");
        return {
          decision: { approved_venues: [], reviewer_notes: notes || undefined },
          reasoning: "No venues to approve",
        };
      }

      for (let i = 0; i < venues.length; i++) {
        const v = venues[i]!;
        console.log(`\n--- Venue ${i + 1}: ${v.name} ---`);
        console.log(`Why: ${v.why}`);
        console.log(`Capacity: ${v.capacity}`);
        console.log(`Price: ${v.price_range}`);
        console.log(`Contact: ${v.contact_email}`);
        console.log(`Website: ${v.website}`);
        console.log(`\nDraft email:\n${v.draft_email}`);
      }

      // Ask which to approve
      const indices = venues.map((_v, i) => i + 1).join(", ");
      const selection = await rl.question(
        `\nWhich venues to contact? (${indices}, comma-separated, or 'none'): `,
      );

      if (selection.trim().toLowerCase() === "none") {
        return {
          decision: { approved_venues: [], reviewer_notes: "Reviewer rejected all venues" },
          reasoning: "None selected",
        };
      }

      const selected = selection
        .split(",")
        .map((s) => parseInt(s.trim(), 10) - 1)
        .filter((i) => i >= 0 && i < venues.length);

      const approvedVenues: ApprovedVenue[] = [];

      for (const idx of selected) {
        const v = venues[idx]!;
        const editChoice = await rl.question(
          `\nEdit email for ${v.name}? (y/N): `,
        );

        let finalEmail = v.draft_email;
        if (editChoice.trim().toLowerCase() === "y") {
          console.log("Enter new email text (end with a blank line):");
          const lines: string[] = [];
          while (true) {
            const line = await rl.question("");
            if (line === "") break;
            lines.push(line);
          }
          finalEmail = lines.join("\n");
        }

        approvedVenues.push({
          name: v.name,
          final_email: finalEmail,
          contact_email: v.contact_email,
        });
      }

      const notes = await rl.question("\nAny reviewer notes? (press Enter to skip): ");

      return {
        decision: {
          approved_venues: approvedVenues,
          reviewer_notes: notes || undefined,
        },
        reasoning: `Approved ${approvedVenues.length} of ${venues.length} venues`,
      };
    } finally {
      rl.close();
    }
  }
}
```

**Step 2: Typecheck**

Run: `cd typescript && npx tsc --noEmit`
Expected: No errors.

**Step 3: Commit**

```bash
git add typescript/src/e2e/readline-input.ts
git commit -m "feat(e2e): readline human input for live demo"
```

---

### Task 8: Full CI Test — venue-scout.test.ts

**Files:**
- Create: `typescript/src/e2e/venue-scout.test.ts`

This is the comprehensive E2E test with all mocks. It validates the entire flow including audit trail integrity.

**Step 1: Write the test**

```typescript
import { describe, it, expect, afterEach } from "vitest";
import { Engine } from "../core/engine.js";
import { PostconditionVerifier } from "../agent/postconditions.js";
import { AgentRunner } from "../agent/runner.js";
import { buildVenueScoutNet } from "./venue-scout-net.js";
import { MockNotificationSender, MockHumanInput } from "./mocks.js";
import { runVenueScout } from "./venue-scout-harness.js";
import { AuditLog } from "../core/audit.js";
import type { ProposalsPayload } from "./types.js";

describe("Venue Scout — E2E (mocked)", () => {
  let engine: Engine;

  afterEach(() => {
    engine.close();
  });

  it("full flow: user input → agent research → human review → Discord notifications", async () => {
    engine = new Engine(":memory:");
    const net = buildVenueScoutNet(engine);

    // -- Mock agent: returns 3 venues --
    const mockProposals: ProposalsPayload = {
      venues: [
        {
          name: "The Driskill Hotel",
          why: "Historic charm, intimate event rooms, downtown Austin",
          capacity: "30-80",
          price_range: "$3000-6000",
          contact_email: "events@driskill.com",
          website: "https://driskill.com",
          draft_email: "Dear Driskill Events Team,\n\nWe're planning a candlelit birthday celebration for 40 guests with live jazz...",
        },
        {
          name: "Hotel San José",
          why: "Boutique SoCo vibe, courtyard perfect for intimate gatherings",
          capacity: "20-50",
          price_range: "$2000-4000",
          contact_email: "events@sanjose.com",
          website: "https://sanjosehotel.com",
          draft_email: "Dear San José Events,\n\nYour courtyard space caught our eye for a 40-person birthday...",
        },
        {
          name: "Jacoby's Restaurant",
          why: "Rustic outdoor space on the river, great for a relaxed vibe",
          capacity: "30-60",
          price_range: "$1500-3500",
          contact_email: "events@jacobys.com",
          website: "https://jacobysaustin.com",
          draft_email: "Hi Jacoby's team,\n\nWe love your riverside space for a 40-person birthday party...",
        },
      ],
      search_summary: "Found 8 venues in Austin, shortlisted 3 based on capacity (30-50), budget ($2000-5000), and intimate/jazz vibe",
    };

    const executor = async () => ({
      text: "Searched for venues and drafted emails",
      toolResults: [
        { toolName: "web-search", result: { raw: "austin intimate event venues..." } },
        { toolName: "web-search", result: { raw: "austin jazz venues private events..." } },
        { toolName: "fetch-page-content", result: { raw: "driskill.com content..." } },
      ],
      payload: mockProposals as unknown as Record<string, unknown>,
    });

    const verifier = new PostconditionVerifier(
      new Map<string, (e: { text: string; toolResults: Array<{ toolName: string; result: Record<string, unknown> }>; payload: Record<string, unknown> }) => boolean>([
        ["venues-found", (e) => {
          const p = e.payload as unknown as ProposalsPayload;
          return Array.isArray(p.venues) && p.venues.length > 0;
        }],
        ["emails-drafted", (e) => {
          const p = e.payload as unknown as ProposalsPayload;
          return p.venues.every((v) => v.draft_email.length > 0);
        }],
        ["at-least-3-venues", (e) => {
          const p = e.payload as unknown as ProposalsPayload;
          return p.venues.length >= 3;
        }],
      ]),
    );

    const runner = new AgentRunner(engine, verifier, { executor });

    // -- Mock human: approves Driskill and Jacoby's, drops San José --
    const mockHuman = new MockHumanInput({
      decision: {
        approved_venues: [
          {
            name: "The Driskill Hotel",
            final_email: "Dear Driskill Events Team,\n\nWe're planning a candlelit birthday celebration for 40 guests with live jazz...",
            contact_email: "events@driskill.com",
          },
          {
            name: "Jacoby's Restaurant",
            final_email: "Hi Jacoby's team,\n\nWe love your riverside space for a 40-person birthday party...",
            contact_email: "events@jacobys.com",
          },
        ],
        reviewer_notes: "Dropped San José — courtyard feels too small for 40 people",
      },
      reasoning: "Selected 2 of 3 venues",
    });

    const mockSender = new MockNotificationSender();

    // -- Run --
    const result = await runVenueScout({
      engine,
      runner,
      net,
      eventPayload: {
        event_type: "birthday party",
        vibe: "intimate, candlelit, jazz",
        headcount: 40,
        budget: "$2000-5000",
        location_area: "Austin, TX",
        preferred_date: "2026-04-15",
        special_requirements: "outdoor space preferred, needs AV for speeches",
      },
      humanInput: mockHuman,
      notificationSender: mockSender,
    });

    // -- Assertions --

    // 1. Workflow completed successfully
    expect(result.success).toBe(true);

    // 2. Final marking: token at outreach-sent, nowhere else
    const marking = engine.getMarking(result.instanceId);
    expect(marking.has("outreach-sent")).toBe(true);
    expect(marking.size).toBe(1);

    // 3. Notifications sent for exactly the 2 approved venues
    expect(mockSender.messages).toHaveLength(2);

    const driskillMsg = mockSender.messages.find((m) => m.recipient === "events@driskill.com");
    expect(driskillMsg).toBeDefined();
    expect(driskillMsg!.subject).toContain("Driskill");
    expect(driskillMsg!.body).toContain("candlelit birthday");

    const jacobysMsg = mockSender.messages.find((m) => m.recipient === "events@jacobys.com");
    expect(jacobysMsg).toBeDefined();
    expect(jacobysMsg!.subject).toContain("Jacoby");
    expect(jacobysMsg!.body).toContain("riverside");

    // 4. No notification for dropped venue
    expect(mockSender.messages.find((m) => m.recipient === "events@sanjose.com")).toBeUndefined();

    // 5. Audit trail: instance_created + 3 transition firings
    const history = engine.getHistory(result.instanceId);
    expect(history.find((e) => e.action === "instance_created")).toBeDefined();
    const firedTransitions = history
      .filter((e) => e.action === "transition_fired")
      .map((e) => e.transition_id);
    expect(firedTransitions).toContain("research-venues");
    expect(firedTransitions).toContain("review-proposals");
    expect(firedTransitions).toContain("send-outreach");

    // 6. Audit chain integrity
    const auditLog = new AuditLog(engine["db"]);
    const chainValid = auditLog.verifyChain(result.instanceId);
    expect(chainValid).toBe(true);
  });
});
```

**Step 2: Run tests to verify they pass**

Run: `cd typescript && npx vitest run src/e2e/venue-scout.test.ts`
Expected: PASS (1 test)

Note: this test depends on Tasks 2, 3, and 5 being complete. It should pass if those are done.

**Step 3: Run the full test suite to check for regressions**

Run: `cd typescript && npx vitest run`
Expected: All tests pass (71 existing + new tests).

**Step 4: Commit**

```bash
git add typescript/src/e2e/venue-scout.test.ts
git commit -m "test(e2e): full venue scout E2E test with mocks"
```

---

### Task 9: Live Runner Script

**Files:**
- Create: `typescript/src/e2e/run.ts`

This is the live demo script. Not tested in CI — run manually with `npx tsx src/e2e/run.ts`.

**Step 1: Write the script**

```typescript
import * as readline from "node:readline/promises";
import { stdin, stdout } from "node:process";
import { resolve } from "node:path";
import { homedir } from "node:os";
import { Engine } from "../core/engine.js";
import { PostconditionVerifier } from "../agent/postconditions.js";
import { AgentRunner } from "../agent/runner.js";
import { Agent } from "@mariozechner/pi-agent-core";
import { getModel } from "@mariozechner/pi-ai";
import { buildVenueScoutNet } from "./venue-scout-net.js";
import { runVenueScout } from "./venue-scout-harness.js";
import { DiscordWebhookSender } from "./discord-webhook.js";
import { ReadlineHumanInput } from "./readline-input.js";
import { createWebSearchTool, createFetchPageContentTool } from "./tools.js";
import type { ExecutionEvidence } from "../agent/postconditions.js";
import type { AgentTool, AgentMessage } from "@mariozechner/pi-agent-core";
import type { AssistantMessage, Message, TextContent, ToolCall } from "@mariozechner/pi-ai";
import type { ProposalsPayload } from "./types.js";

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const BRAVE_SEARCH_DIR = resolve(homedir(), ".my-pi/skills/brave-search");
const SEARCH_SCRIPT = resolve(BRAVE_SEARCH_DIR, "search.js");
const CONTENT_SCRIPT = resolve(BRAVE_SEARCH_DIR, "content.js");
const DB_PATH = ":memory:";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function extractText(messages: AgentMessage[]): string {
  const parts: string[] = [];
  for (const msg of messages) {
    if ("role" in msg && msg.role === "assistant") {
      for (const c of msg.content) {
        if (c.type === "text") parts.push(c.text);
      }
    }
  }
  return parts.join("\n");
}

function extractJsonPayload(text: string): Record<string, unknown> {
  // Try to find JSON block in the text
  const jsonMatch = text.match(/```json\s*([\s\S]*?)\s*```/);
  if (jsonMatch?.[1]) {
    try {
      return JSON.parse(jsonMatch[1]) as Record<string, unknown>;
    } catch { /* fall through */ }
  }

  // Try to find raw JSON object
  const braceMatch = text.match(/\{[\s\S]*\}/);
  if (braceMatch?.[0]) {
    try {
      return JSON.parse(braceMatch[0]) as Record<string, unknown>;
    } catch { /* fall through */ }
  }

  return {};
}

// ---------------------------------------------------------------------------
// Live executor using pi-agent-core
// ---------------------------------------------------------------------------

function createLiveExecutor(tools: AgentTool[]): (
  transitionId: string,
  systemPrompt: string,
  contextPrompt: string,
  agentTools: unknown[],
) => Promise<ExecutionEvidence> {
  return async (transitionId, systemPrompt, contextPrompt, _agentTools) => {
    console.log(`\n🤖 Agent executing: ${transitionId}`);
    console.log("   Calling LLM with tools:", tools.map((t) => t.name).join(", "));

    const model = getModel("anthropic", "claude-sonnet-4-20250514");

    const convertToLlm = (messages: AgentMessage[]): Message[] => {
      return messages.filter(
        (m): m is Message => "role" in m && ["user", "assistant", "toolResult"].includes(m.role),
      );
    };

    const agent = new Agent({
      initialState: {
        systemPrompt: systemPrompt + "\n\nIMPORTANT: After completing your research, output your final result as a JSON object inside a ```json code block. The JSON must have 'venues' (array) and 'search_summary' (string) fields.",
        model,
        tools,
        thinkingLevel: "low",
        messages: [],
      },
      convertToLlm,
    });

    const toolResults: Array<{ toolName: string; result: Record<string, unknown> }> = [];

    agent.subscribe((event) => {
      if (event.type === "tool_execution_start") {
        console.log(`   🔧 Tool: ${event.toolName}(${JSON.stringify(event.args).slice(0, 100)}...)`);
      }
      if (event.type === "tool_execution_end") {
        if (!event.isError) {
          toolResults.push({
            toolName: event.toolName,
            result: (typeof event.result === "object" ? event.result : { raw: event.result }) as Record<string, unknown>,
          });
        }
        console.log(`   ${event.isError ? "❌" : "✅"} ${event.toolName} done`);
      }
    });

    await agent.prompt(contextPrompt);

    const text = extractText(agent.state.messages);
    const payload = extractJsonPayload(text);

    console.log(`   📦 Agent produced ${Object.keys(payload).length} payload keys`);
    if ("venues" in payload && Array.isArray(payload.venues)) {
      console.log(`   📍 Found ${payload.venues.length} venues`);
    }

    return { text, toolResults, payload };
  };
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const discordWebhookUrl = process.env["DISCORD_WEBHOOK_URL"];
  if (!discordWebhookUrl) {
    console.error("❌ Set DISCORD_WEBHOOK_URL environment variable");
    console.error("   Create a webhook in Discord: Server Settings → Integrations → Webhooks");
    process.exit(1);
  }

  if (!process.env["BRAVE_API_KEY"]) {
    console.error("❌ Set BRAVE_API_KEY environment variable");
    process.exit(1);
  }

  const rl = readline.createInterface({ input: stdin, output: stdout });

  console.log("🎉 Venue Scout — Live Demo");
  console.log("=".repeat(40));
  console.log("Describe your event and I'll find venues for you.\n");

  const eventType = await rl.question("Event type (e.g., birthday party, corporate retreat): ");
  const vibe = await rl.question("Vibe (e.g., intimate, candlelit, jazz): ");
  const headcountStr = await rl.question("Headcount: ");
  const budget = await rl.question("Budget range (e.g., $2000-5000): ");
  const locationArea = await rl.question("Location area (e.g., Austin, TX): ");
  const preferredDate = await rl.question("Preferred date (e.g., 2026-04-15): ");
  const specialReqs = await rl.question("Special requirements (or press Enter): ");

  rl.close();

  const eventPayload = {
    event_type: eventType,
    vibe,
    headcount: parseInt(headcountStr, 10) || 20,
    budget,
    location_area: locationArea,
    preferred_date: preferredDate,
    special_requirements: specialReqs,
  };

  console.log("\n🚀 Starting workflow...\n");

  const engine = new Engine(DB_PATH);

  try {
    const net = buildVenueScoutNet(engine);

    // Tools
    const searchTool = createWebSearchTool(SEARCH_SCRIPT);
    const contentTool = createFetchPageContentTool(CONTENT_SCRIPT);

    // Executor
    const executor = createLiveExecutor([searchTool, contentTool]);

    // Postcondition verifiers
    const verifier = new PostconditionVerifier(
      new Map<string, (e: ExecutionEvidence) => boolean>([
        ["venues-found", (e) => {
          const p = e.payload as unknown as ProposalsPayload;
          return Array.isArray(p?.venues) && p.venues.length > 0;
        }],
        ["emails-drafted", (e) => {
          const p = e.payload as unknown as ProposalsPayload;
          return Array.isArray(p?.venues) && p.venues.every((v) => v.draft_email?.length > 0);
        }],
        ["at-least-3-venues", (e) => {
          const p = e.payload as unknown as ProposalsPayload;
          return Array.isArray(p?.venues) && p.venues.length >= 3;
        }],
      ]),
    );

    const runner = new AgentRunner(engine, verifier, {
      executor,
      toolRegistry: new Map([
        ["web-search", searchTool],
        ["fetch-page-content", contentTool],
      ]),
    });

    const humanInput = new ReadlineHumanInput();
    const notificationSender = new DiscordWebhookSender(discordWebhookUrl);

    const result = await runVenueScout({
      engine,
      runner,
      net,
      eventPayload: eventPayload as unknown as Record<string, unknown>,
      humanInput,
      notificationSender,
    });

    if (result.success) {
      console.log("\n✅ Workflow complete!");

      // Print audit summary
      const history = engine.getHistory(result.instanceId);
      console.log(`\n📜 Audit trail: ${history.length} entries`);
      for (const entry of history) {
        const transition = entry.transition_id ? ` [${entry.transition_id}]` : "";
        console.log(`   ${entry.action}${transition} — ${entry.timestamp}`);
      }
    } else {
      console.error(`\n❌ Workflow failed: ${result.error}`);
    }
  } finally {
    engine.close();
  }
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
```

**Step 2: Typecheck**

Run: `cd typescript && npx tsc --noEmit`
Expected: No errors.

**Step 3: Add a script to package.json**

Add to `scripts` in `typescript/package.json`:
```json
"e2e:live": "tsx --env-file=.env src/e2e/run.ts"
```

**Step 4: Commit**

```bash
git add typescript/src/e2e/run.ts typescript/package.json
git commit -m "feat(e2e): live venue scout demo script"
```

---

### Task 10: Final Verification

**Step 1: Run full test suite**

Run: `cd typescript && npx vitest run`
Expected: All tests pass — existing 71 + new tests from tasks 2, 3, 5, 6, 8.

**Step 2: Typecheck entire project**

Run: `cd typescript && npx tsc --noEmit`
Expected: No errors.

**Step 3: Verify file structure**

Run: `find typescript/src/e2e -type f | sort`
Expected:
```
typescript/src/e2e/discord-webhook.test.ts
typescript/src/e2e/discord-webhook.ts
typescript/src/e2e/mocks.test.ts
typescript/src/e2e/mocks.ts
typescript/src/e2e/readline-input.ts
typescript/src/e2e/run.ts
typescript/src/e2e/tools.test.ts
typescript/src/e2e/tools.ts
typescript/src/e2e/types.ts
typescript/src/e2e/venue-scout-harness.test.ts
typescript/src/e2e/venue-scout-harness.ts
typescript/src/e2e/venue-scout-net.test.ts
typescript/src/e2e/venue-scout-net.ts
typescript/src/e2e/venue-scout.test.ts
```

**Step 4: Commit any final adjustments**

```bash
git add -A
git commit -m "chore(e2e): final verification — all tests pass"
```
