import type { WorkOrder } from "../core/context.js";

/**
 * Build the system prompt for an agentic transition.
 * Contains: task intent, policies, postconditions, evidence requirements,
 * output schema, and available tools.
 */
export function buildSystemPrompt(workOrder: WorkOrder): string {
  const requiredPcs = workOrder.postconditions.required
    .map((pc) => `- [ ] ${pc}`)
    .join("\n");

  const desiredPcs = workOrder.postconditions.desired
    ?.map((pc) => `- [ ] ${pc}`)
    .join("\n");

  const policySection = workOrder.policies
    .map((p) => `[${p.strength.toUpperCase()}] ${p.text}`)
    .join("\n");

  const evidenceSection = workOrder.evidence_requirements
    .map((e) => `- ${e.description} (${e.type}${e.required ? ", required" : ""})`)
    .join("\n");

  const outputSchemaSection = workOrder.output_schema
    ? `\n## Output Schema\nYour output must conform to:\n\`\`\`json\n${JSON.stringify(workOrder.output_schema, null, 2)}\n\`\`\``
    : "";

  return `You are executing a single transition in an institutional workflow.

## Your Task
${workOrder.intent}

## Governing Policies
${policySection || "No specific policies."}

## Success Criteria
The following MUST be true when you are done:
${requiredPcs}
${desiredPcs ? `\nThe following are desirable but not required:\n${desiredPcs}` : ""}

## Evidence Requirements
${evidenceSection || "No evidence requirements."}
${outputSchemaSection}

## Available Tools
${workOrder.available_tools.join(", ") || "None"}

## Instructions
1. Review the context, policies, and success criteria carefully.
2. Use the available tools to accomplish the task.
3. Ensure all required postconditions are satisfied.
4. Capture required evidence.

After completing your work, provide a structured summary:
- Which success criteria you satisfied and the evidence.
- Any issues encountered.
- Data or artifacts produced.

IMPORTANT: You must actually call the tools. Do not just describe what you would do.`;
}

/**
 * Build the context prompt with case data and institutional context.
 */
export function buildContextPrompt(
  workOrder: WorkOrder,
  institutionalContext: Record<string, unknown>,
): string {
  const tokenSection = workOrder.token_payloads
    .map((p, i) => `### Token ${i + 1}\n${JSON.stringify(p, null, 2)}`)
    .join("\n\n");

  const contextEntries = Object.entries(institutionalContext);
  const contextSection = contextEntries.length > 0
    ? contextEntries
        .map(([key, value]) => {
          const formatted = typeof value === "string" ? value : JSON.stringify(value, null, 2);
          return `### ${key}\n${formatted}`;
        })
        .join("\n\n")
    : "No additional institutional context.";

  return `## Case Data
${tokenSection || "No token data."}

## Institutional Context
${contextSection}

Please proceed with executing this transition.`;
}
