import { Agent, type AgentTool } from "@mariozechner/pi-agent-core";
import { getModel } from "@mariozechner/pi-ai";
import type { Transition, ExecutionResult } from "../net/types.js";

function buildSystemPrompt(transition: Transition): string {
  const requiredChecklist = transition.postconditions.required
    .map((pc) => `- [ ] ${pc}`)
    .join("\n");

  const desiredChecklist = transition.postconditions.desired
    ?.map((pc) => `- [ ] ${pc}`)
    .join("\n");

  return `You are executing a single transition in an institutional workflow.

## Execution mode
${transition.mode}

## Your task
${transition.intent}

## Success criteria
The following MUST be true when you are done:
${requiredChecklist}

${
  desiredChecklist
    ? `The following are desirable but not required:\n${desiredChecklist}`
    : ""
}

## Available tools
You have access to: ${transition.available_tools.join(", ")}

## Instructions
1. Review the context and success criteria carefully
2. Decide which tools to use and in what order
3. Execute your plan by calling the appropriate tools
4. After using tools, summarize what you accomplished

## Output format
After completing your work, provide a brief structured summary:
- Which success criteria you satisfied and the evidence
- Any issues encountered
- Data or artifacts produced (these will be passed to the next step)

IMPORTANT: You must actually call the tools to accomplish the task. Do not just describe what you would do.`;
}

function buildContextPrompt(
  transition: Transition,
  context: Record<string, unknown>,
): string {
  const contextEntries = Object.entries(context)
    .map(([key, value]) => {
      const formatted =
        typeof value === "string" ? value : JSON.stringify(value, null, 2);
      return `### ${key}\n${formatted}`;
    })
    .join("\n\n");

  return `## Transition: ${transition.id}

## Context
${contextEntries || "No additional context available."}

Please proceed with executing this transition.`;
}

/**
 * Execute a single transition by handing its definition to an LLM agent
 * with the appropriate tools and context.
 */
export async function executeTransition(
  transition: Transition,
  context: Record<string, unknown>,
  tools: AgentTool[],
): Promise<ExecutionResult> {
  console.log(`\n--- Executing transition: ${transition.id} (${transition.mode}) ---`);

  const systemPrompt = buildSystemPrompt(transition);
  const contextPrompt = buildContextPrompt(transition, context);

  const agent = new Agent({
    initialState: {
      systemPrompt,
      model: getModel("anthropic", "claude-sonnet-4-20250514"),
      tools,
    },
  });

  // Collect results from agent events
  const toolResults: ExecutionResult["toolResults"] = [];
  const actions: ExecutionResult["actions"] = [];
  let text = "";

  agent.subscribe((event) => {
    switch (event.type) {
      case "message_update":
        if (event.assistantMessageEvent.type === "text_delta") {
          text += event.assistantMessageEvent.delta;
        }
        break;
      case "tool_execution_end":
        toolResults.push({
          toolName: event.toolName,
          result: (event.result.details ?? {}) as Record<string, unknown>,
        });
        actions.push({
          toolName: event.toolName,
          args: {},
          result: (event.result.details ?? {}) as Record<string, unknown>,
        });
        break;
    }
  });

  await agent.prompt(contextPrompt);
  await agent.waitForIdle();

  console.log(`  Agent used ${toolResults.length} tool(s)`);
  console.log(`  Agent response: ${text.slice(0, 200)}...`);

  return {
    actions,
    text,
    toolResults,
    payload: extractPayload(transition, toolResults),
  };
}

/**
 * Extract the payload that should be carried forward by the output token.
 * This assembles relevant data from tool results into a structured payload.
 */
function extractPayload(
  transition: Transition,
  toolResults: ExecutionResult["toolResults"],
): Record<string, unknown> {
  const payload: Record<string, unknown> = {};

  for (const tr of toolResults) {
    switch (tr.toolName) {
      case "lookup-vendor":
        if (tr.result.found) {
          payload["vendor-profile"] = tr.result;
        }
        break;
      case "generate-document":
        if (tr.result.success) {
          payload[`document-${tr.result.documentType}`] = tr.result.content;
          // Specifically capture risk assessment data
          if (
            typeof tr.result.documentType === "string" &&
            tr.result.documentType.includes("risk")
          ) {
            payload["risk-assessment"] = tr.result.content;
          }
        }
        break;
      case "send-notification":
        if (tr.result.success) {
          payload["compliance-notification"] = {
            channel: tr.result.channel,
            recipient: tr.result.recipient,
            sentAt: tr.result.sentAt,
            messageId: tr.result.messageId,
          };
        }
        break;
    }
  }

  return payload;
}
