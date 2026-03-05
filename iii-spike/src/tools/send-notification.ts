import { Type } from "@mariozechner/pi-ai";
import type { AgentTool } from "@mariozechner/pi-agent-core";

export const sendNotification: AgentTool = {
  name: "send-notification",
  label: "Send Notification",
  description:
    "Send a notification to a specified recipient. Supports email and slack " +
    "channels. The notification is logged and considered delivered.",
  parameters: Type.Object({
    channel: Type.Union(
      [Type.Literal("email"), Type.Literal("slack"), Type.Literal("console")],
      { description: "The notification channel to use" },
    ),
    recipient: Type.String({
      description: "The recipient address (email address or Slack channel)",
    }),
    subject: Type.String({ description: "The notification subject line" }),
    body: Type.String({ description: "The notification body content" }),
  }),
  execute: async (_toolCallId, params) => {
    const { channel, recipient, subject, body } = params as {
      channel: string; recipient: string; subject: string; body: string;
    };
    console.log(`  [tool] send-notification via ${channel} to ${recipient}`);
    console.log(`         Subject: ${subject}`);
    console.log(
      `         Body: ${body.slice(0, 200)}${body.length > 200 ? "..." : ""}`,
    );

    // In a real system this would send via an API.
    // For the spike we log and return success.
    const timestamp = new Date().toISOString();
    const result = {
      success: true,
      channel,
      recipient,
      subject,
      sentAt: timestamp,
      messageId: `msg-${Date.now()}`,
    };
    return {
      content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
      details: result,
    };
  },
};
