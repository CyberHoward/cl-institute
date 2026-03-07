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
      token_payloads: [],
      policies: [],
    };

    const result = await human.prompt(context);
    expect(result.decision).toEqual(cannedDecision.decision);
    expect(result.reasoning).toBe("Looks good");
  });
});
