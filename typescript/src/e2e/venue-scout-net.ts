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
