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
