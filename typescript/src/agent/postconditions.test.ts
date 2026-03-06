import { describe, it, expect } from "vitest";
import { PostconditionVerifier } from "./postconditions.js";
import type { Postconditions, PostconditionResult } from "../core/types.js";

// Fake execution result for testing
const makeResult = (toolResults: Array<{ toolName: string; result: Record<string, unknown> }>) => ({
  text: "Agent completed the task.",
  toolResults,
  payload: {},
});

describe("PostconditionVerifier", () => {
  it("uses deterministic verifier when registered", async () => {
    const verifier = new PostconditionVerifier(
      new Map([
        ["doc-generated", (r) => r.toolResults.some((t) => t.toolName === "generate-document" && t.result["success"] === true)],
      ]),
    );

    const result = makeResult([
      { toolName: "generate-document", result: { success: true } },
    ]);

    const postconditions: Postconditions = {
      required: ["doc-generated"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results).toHaveLength(1);
    expect(results[0]!.postcondition).toBe("doc-generated");
    expect(results[0]!.satisfied).toBe(true);
    expect(results[0]!.method).toBe("deterministic");
    expect(results[0]!.confidence).toBe(1.0);
  });

  it("reports deterministic failure correctly", async () => {
    const verifier = new PostconditionVerifier(
      new Map([
        ["doc-generated", (r) => r.toolResults.some((t) => t.toolName === "generate-document" && t.result["success"] === true)],
      ]),
    );

    const result = makeResult([]);

    const postconditions: Postconditions = {
      required: ["doc-generated"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results[0]!.satisfied).toBe(false);
    expect(results[0]!.method).toBe("deterministic");
    expect(results[0]!.confidence).toBe(1.0);
  });

  it("verifies both required and desired postconditions", async () => {
    const verifier = new PostconditionVerifier(
      new Map([
        ["required-thing", (): boolean => true],
        ["nice-to-have", (): boolean => false],
      ]),
    );

    const result = makeResult([]);
    const postconditions: Postconditions = {
      required: ["required-thing"],
      desired: ["nice-to-have"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results).toHaveLength(2);
    expect(results.find((r) => r.postcondition === "required-thing")!.satisfied).toBe(true);
    expect(results.find((r) => r.postcondition === "nice-to-have")!.satisfied).toBe(false);
  });

  it("checks allRequiredMet correctly", async () => {
    const verifier = new PostconditionVerifier(
      new Map([
        ["a", (): boolean => true],
        ["b", (): boolean => false],
      ]),
    );

    const result = makeResult([]);
    const postconditions: Postconditions = {
      required: ["a", "b"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(PostconditionVerifier.allRequiredMet(postconditions, results)).toBe(false);
  });

  it("falls back to llm judge when no deterministic verifier and judge provided", async () => {
    const mockJudge = async (_pc: string, _result: unknown): Promise<{ satisfied: boolean; confidence: number }> => {
      return { satisfied: true, confidence: 0.85 };
    };

    const verifier = new PostconditionVerifier(new Map(), mockJudge);

    const result = makeResult([]);
    const postconditions: Postconditions = {
      required: ["semantic-postcondition"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results[0]!.method).toBe("llm");
    expect(results[0]!.satisfied).toBe(true);
    expect(results[0]!.confidence).toBe(0.85);
  });

  it("fails postcondition when no verifier and no judge", async () => {
    const verifier = new PostconditionVerifier(new Map());

    const result = makeResult([]);
    const postconditions: Postconditions = {
      required: ["unverifiable-postcondition"],
    };

    const results = await verifier.verify(postconditions, result);
    expect(results[0]!.satisfied).toBe(false);
    expect(results[0]!.method).toBe("deterministic");
    expect(results[0]!.confidence).toBe(0);
  });
});
