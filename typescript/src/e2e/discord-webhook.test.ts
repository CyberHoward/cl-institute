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
