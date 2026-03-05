import { z } from "zod";

export const PlaceSchema = z.object({
  id: z.string(),
  description: z.string(),
});

export const PostconditionSchema = z.object({
  /** Must all be true for the transition to be considered fired */
  required: z.array(z.string()),
  /** Ideal outcomes — logged but not blocking */
  desired: z.array(z.string()).optional(),
  /** If required postconditions aren't met within timeout, trigger these */
  escalation: z.array(z.string()).optional(),
});

export const TransitionSchema = z.object({
  id: z.string(),

  // -- Formal (what the net cares about) --
  consumes: z.array(z.string()), // input place IDs — tokens removed
  produces: z.array(z.string()), // output place IDs — tokens added
  guard: z.string().optional(), // expression evaluated against marking

  // -- Semantic (what the agent cares about) --
  intent: z.string(), // natural language goal
  context_sources: z.array(z.string()), // keys to look up in context store
  postconditions: PostconditionSchema,

  // -- Capabilities (what the agent can use) --
  available_tools: z.array(z.string()),

  // -- Execution mode --
  mode: z.enum(["deterministic", "judgment", "agentic"]),
});

export const NetSchema = z.object({
  id: z.string(),
  places: z.array(PlaceSchema),
  transitions: z.array(TransitionSchema),
});

/** A marking is a map from place ID → token count + optional payload */
export const TokenSchema = z.object({
  count: z.number().default(1),
  payload: z.record(z.unknown()).optional(),
});

export type Place = z.infer<typeof PlaceSchema>;
export type Postcondition = z.infer<typeof PostconditionSchema>;
export type Transition = z.infer<typeof TransitionSchema>;
export type Net = z.infer<typeof NetSchema>;
export type Token = z.infer<typeof TokenSchema>;
export type Marking = Map<string, Token>;

export interface AgentAction {
  toolName: string;
  args: Record<string, unknown>;
  result: unknown;
}

export interface ExecutionResult {
  actions: AgentAction[];
  text: string;
  toolResults: Array<{
    toolName: string;
    result: Record<string, unknown>;
  }>;
  payload: Record<string, unknown>;
}

export interface ExecutionLog {
  transitionId: string;
  markingBefore: Record<string, Token>;
  agentActions: AgentAction[];
  postconditionResults: Record<string, boolean>;
  markingAfter: Record<string, Token>;
  status: "fired" | "failed" | "escalated";
  durationMs: number;
}
