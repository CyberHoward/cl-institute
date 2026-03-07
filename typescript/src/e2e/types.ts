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
