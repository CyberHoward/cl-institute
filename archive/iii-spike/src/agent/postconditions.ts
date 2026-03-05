import type { ExecutionResult, Postcondition } from "../net/types.js";
import { generateText } from "./llm.js";

type Verifier = (result: ExecutionResult) => boolean;

/**
 * Deterministic postcondition verifiers.
 * Each maps a postcondition string to a function that inspects tool results.
 */
const deterministicVerifiers: Record<string, Verifier> = {
  "vendor-identity-confirmed": (r) =>
    r.toolResults.some(
      (t) =>
        t.toolName === "lookup-vendor" &&
        t.result.found === true &&
        t.result.registrationStatus === "active",
    ),

  "vendor-profile-retrieved": (r) =>
    r.toolResults.some(
      (t) => t.toolName === "lookup-vendor" && t.result.found === true,
    ),

  "vendor-certifications-listed": (r) =>
    r.toolResults.some(
      (t) =>
        t.toolName === "lookup-vendor" &&
        Array.isArray(t.result.certifications) &&
        (t.result.certifications as unknown[]).length > 0,
    ),

  "vendor-compliance-history-available": (r) =>
    r.toolResults.some(
      (t) =>
        t.toolName === "lookup-vendor" &&
        t.result.complianceHistory != null,
    ),

  "notification-sent": (r) =>
    r.toolResults.some(
      (t) => t.toolName === "send-notification" && t.result.success === true,
    ),

  "risk-assessment-documented": (r) =>
    r.toolResults.some(
      (t) =>
        t.toolName === "generate-document" &&
        t.result.success === true &&
        typeof t.result.content === "string" &&
        (t.result.content as string).length > 50,
    ),
};

/**
 * LLM-as-judge fallback for postconditions that can't be verified deterministically.
 */
async function llmJudge(
  postcondition: string,
  result: ExecutionResult,
): Promise<boolean> {
  console.log(`  [postcondition] LLM judge evaluating: "${postcondition}"`);

  const evidence = {
    agentText: result.text,
    toolsCalled: result.toolResults.map((t) => ({
      tool: t.toolName,
      resultKeys: Object.keys(t.result),
      resultSummary:
        JSON.stringify(t.result).slice(0, 500),
    })),
    agentPayload: result.payload,
  };

  const response = await generateText({
    system:
      "You are a postcondition verifier. Given evidence from an agent's execution, " +
      "determine whether a specific postcondition has been satisfied. Respond with " +
      'ONLY "true" or "false" followed by a one-sentence explanation.',
    prompt:
      `Postcondition to verify: "${postcondition}"\n\n` +
      `Evidence:\n${JSON.stringify(evidence, null, 2)}`,
  });

  const satisfied = response.toLowerCase().startsWith("true");
  console.log(
    `  [postcondition] LLM judge: ${postcondition} → ${satisfied} (${response.trim()})`,
  );
  return satisfied;
}

/**
 * Verify all postconditions for a transition's execution result.
 * Uses deterministic verifiers where available, falls back to LLM-as-judge.
 */
export async function verifyPostconditions(
  postconditions: Postcondition,
  result: ExecutionResult,
): Promise<{
  allRequiredMet: boolean;
  results: Record<string, boolean>;
}> {
  const results: Record<string, boolean> = {};

  // Check required postconditions
  for (const pc of postconditions.required) {
    if (deterministicVerifiers[pc]) {
      results[pc] = deterministicVerifiers[pc](result);
      console.log(
        `  [postcondition] deterministic: ${pc} → ${results[pc]}`,
      );
    } else {
      results[pc] = await llmJudge(pc, result);
    }
  }

  // Check desired postconditions (non-blocking)
  if (postconditions.desired) {
    for (const pc of postconditions.desired) {
      if (deterministicVerifiers[pc]) {
        results[pc] = deterministicVerifiers[pc](result);
        console.log(
          `  [postcondition] deterministic (desired): ${pc} → ${results[pc]}`,
        );
      } else {
        results[pc] = await llmJudge(pc, result);
      }
    }
  }

  const allRequiredMet = postconditions.required.every((pc) => results[pc]);

  return { allRequiredMet, results };
}
