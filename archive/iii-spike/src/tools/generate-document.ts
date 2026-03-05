import { Type } from "@mariozechner/pi-ai";
import type { AgentTool } from "@mariozechner/pi-agent-core";
import { generateText } from "../agent/llm.js";

export const generateDocument: AgentTool = {
  name: "generate-document",
  label: "Generate Document",
  description:
    "Generate a structured document (risk assessment, summary report, etc.) " +
    "based on provided data and instructions. Returns the formatted document text.",
  parameters: Type.Object({
    documentType: Type.String({
      description:
        "The type of document to generate (e.g., 'risk-assessment', 'summary')",
    }),
    data: Type.String({
      description: "JSON string of the data to base the document on",
    }),
    instructions: Type.String({
      description: "Specific instructions for how to structure the document",
    }),
  }),
  execute: async (_toolCallId, params) => {
    const { documentType, data, instructions } = params as {
      documentType: string; data: string; instructions: string;
    };
    console.log(`  [tool] generate-document: creating ${documentType}`);

    const text = await generateText({
      system:
        "You are a document generation assistant. Generate clear, professional " +
        "documents based on the provided data and instructions. Output only the " +
        "document content, no preamble.",
      prompt: `Generate a ${documentType} document.\n\nData:\n${data}\n\nInstructions:\n${instructions}`,
    });

    console.log(`  [tool] generate-document: generated ${text.length} chars`);

    const result = {
      success: true,
      documentType,
      content: text,
      generatedAt: new Date().toISOString(),
    };
    return {
      content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
      details: result,
    };
  },
};
