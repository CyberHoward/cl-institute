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
