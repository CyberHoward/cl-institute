import type { Postconditions, PostconditionResult } from "../core/types.js";

/** The execution evidence passed to verifiers. */
export interface ExecutionEvidence {
  text: string;
  toolResults: Array<{ toolName: string; result: Record<string, unknown> }>;
  payload: Record<string, unknown>;
}

/** A deterministic verifier: inspects execution evidence, returns boolean. */
export type DeterministicVerifier = (evidence: ExecutionEvidence) => boolean;

/** LLM-as-judge function signature. */
export type LlmJudge = (
  postcondition: string,
  evidence: ExecutionEvidence,
) => Promise<{ satisfied: boolean; confidence: number }>;

/**
 * Verifies postconditions using deterministic verifiers with LLM-as-judge fallback.
 * Tracks verification method and confidence for audit trail transparency.
 */
export class PostconditionVerifier {
  constructor(
    private readonly verifiers: Map<string, DeterministicVerifier>,
    private readonly llmJudge?: LlmJudge | undefined,
  ) {}

  /**
   * Verify all postconditions (required + desired) against execution evidence.
   */
  async verify(
    postconditions: Postconditions,
    evidence: ExecutionEvidence,
  ): Promise<PostconditionResult[]> {
    const allPcs = [
      ...postconditions.required,
      ...(postconditions.desired ?? []),
    ];

    const results: PostconditionResult[] = [];

    for (const pc of allPcs) {
      const deterministicFn = this.verifiers.get(pc);

      if (deterministicFn) {
        results.push({
          postcondition: pc,
          satisfied: deterministicFn(evidence),
          method: "deterministic",
          confidence: 1.0,
        });
      } else if (this.llmJudge) {
        const judgment = await this.llmJudge(pc, evidence);
        results.push({
          postcondition: pc,
          satisfied: judgment.satisfied,
          method: "llm",
          confidence: judgment.confidence,
        });
      } else {
        // No verifier available — fail safely
        results.push({
          postcondition: pc,
          satisfied: false,
          method: "deterministic",
          confidence: 0,
        });
      }
    }

    return results;
  }

  /** Check if all required postconditions are satisfied. */
  static allRequiredMet(
    postconditions: Postconditions,
    results: PostconditionResult[],
  ): boolean {
    return postconditions.required.every((pc) => {
      const result = results.find((r) => r.postcondition === pc);
      return result?.satisfied === true;
    });
  }
}
