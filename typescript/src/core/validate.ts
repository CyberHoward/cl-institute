import type { Engine } from "./engine.js";
import type { ConstraintViolation, ValidationResult } from "./types.js";

export function validateNet(engine: Engine, netId: string): ValidationResult {
  const { net, places, transitions } = engine.getNetWithGraph(netId);
  const arcs = engine.getArcs(netId);
  const violations: ConstraintViolation[] = [];

  // Check for orphan places (places with no arcs)
  for (const place of places) {
    const hasArc = arcs.some((a) => a.place_id === place.id);
    if (!hasArc) {
      violations.push({
        constraint_name: "orphan_place",
        severity: "warning",
        message: `Place '${place.id}' has no connections to any transition`,
        location: `net:${netId}/place:${place.id}`,
        suggestion: "Connect this place to a transition or remove it",
      });
    }
  }

  // Check judgment transitions have policies
  for (const t of transitions) {
    if (t.mode === "judgment") {
      const domain = net.domain ?? "";
      const scope = domain ? `${domain}.${t.id}` : t.id;
      const policies = engine.getPolicies(scope);
      if (policies.length === 0) {
        violations.push({
          constraint_name: "judgment_without_policy",
          severity: "warning",
          message: `Judgment transition '${t.id}' has no governing policies (scope: ${scope})`,
          location: `net:${netId}/transition:${t.id}`,
          suggestion: `Attach policies to scope '${scope}' or a parent scope`,
        });
      }
    }
  }

  // Check transitions reference valid places
  const placeIds = new Set(places.map((p) => p.id));
  for (const t of transitions) {
    for (const placeId of t.consumes.concat(t.produces)) {
      if (!placeIds.has(placeId)) {
        violations.push({
          constraint_name: "invalid_place_reference",
          severity: "error",
          message: `Transition '${t.id}' references non-existent place '${placeId}'`,
          location: `net:${netId}/transition:${t.id}`,
        });
      }
    }
  }

  // Check for transitions with no input
  for (const t of transitions) {
    if (t.consumes.length === 0) {
      violations.push({
        constraint_name: "sourceless_transition",
        severity: "warning",
        message: `Transition '${t.id}' has no input places — it can fire at any time`,
        location: `net:${netId}/transition:${t.id}`,
        suggestion: "This may be intentional for event-triggered transitions",
      });
    }
  }

  const hasErrors = violations.some((v) => v.severity === "error");
  return { violations, is_valid: !hasErrors };
}
