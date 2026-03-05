import { Agent } from "@mariozechner/pi-agent-core";
import { getModel } from "@mariozechner/pi-ai";

/**
 * Simple one-shot text generation using pi-agent.
 * Used by the document generation tool and the LLM-as-judge postcondition verifier.
 */
export async function generateText(opts: {
  system: string;
  prompt: string;
}): Promise<string> {
  const agent = new Agent({
    initialState: {
      systemPrompt: opts.system,
      model: getModel("anthropic", "claude-sonnet-4-20250514"),
      tools: [],
    },
  });

  let text = "";
  agent.subscribe((event) => {
    if (
      event.type === "message_update" &&
      event.assistantMessageEvent.type === "text_delta"
    ) {
      text += event.assistantMessageEvent.delta;
    }
  });

  await agent.prompt(opts.prompt);
  await agent.waitForIdle();
  return text;
}
