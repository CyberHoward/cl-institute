import type { Engine } from "./engine.js";
import type {
  Policy,
  Postconditions,
  EvidenceRequirement,
  TransitionMode,
  DecisionType,
  JsonSchema,
} from "./types.js";

export interface WorkOrder {
  // Identity
  transition_id: string;
  instance_id: string;
  mode: TransitionMode;
  decision_type?: DecisionType;

  // 1. What am I trying to accomplish?
  intent: string;

  // 2. What case data do I have?
  token_payloads: Record<string, unknown>[];

  // 3. Do I have everything I need?
  input_schema?: JsonSchema;

  // 4. What rules govern this?
  policies: Policy[];

  // 5. What else should I consider?
  context_sources: string[];

  // 6. What data must I produce?
  output_schema?: JsonSchema;

  // 7. What must be true when I'm done?
  postconditions: Postconditions;

  // 8. What proof must I attach?
  evidence_requirements: EvidenceRequirement[];

  // 9. What can I use?
  available_tools: string[];
}

/**
 * Assemble a complete work order for an agent from the current net state.
 * This is the 9-step context assembly from the design doc.
 */
export function buildWorkOrder(
  engine: Engine,
  instanceId: string,
  transitionId: string,
): WorkOrder {
  const instance = engine.getInstance(instanceId);
  const { net, transitions } = engine.getNetWithGraph(instance.net_id);
  const transition = transitions.find((t) => t.id === transitionId);
  if (!transition) {
    throw new Error(`Transition '${transitionId}' not found in net '${instance.net_id}'`);
  }

  // 2. Gather token payloads from input places
  const marking = engine.getMarking(instanceId);
  const tokenPayloads: Record<string, unknown>[] = [];
  for (const placeId of transition.consumes) {
    const tokens = marking.get(placeId);
    if (tokens) {
      for (const token of tokens) {
        tokenPayloads.push(token.payload);
      }
    }
  }

  // 4. Resolve policies by scope
  const domain = net.domain ?? "";
  const scope = domain ? `${domain}.${transitionId}` : transitionId;
  const policies = engine.getPolicies(scope);

  return {
    transition_id: transitionId,
    instance_id: instanceId,
    mode: transition.mode,
    decision_type: transition.decision_type,
    intent: transition.intent,
    token_payloads: tokenPayloads,
    input_schema: transition.input_schema,
    policies,
    context_sources: transition.context_sources,
    output_schema: transition.output_schema,
    postconditions: transition.postconditions,
    evidence_requirements: transition.evidence_requirements,
    available_tools: transition.available_tools,
  };
}
