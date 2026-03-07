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
