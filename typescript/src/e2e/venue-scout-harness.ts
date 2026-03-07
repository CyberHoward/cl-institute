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
