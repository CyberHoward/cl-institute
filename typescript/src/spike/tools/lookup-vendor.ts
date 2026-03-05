import { Type } from "@mariozechner/pi-ai";
import type { AgentTool } from "@mariozechner/pi-agent-core";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const FIXTURES_DIR = resolve(import.meta.dirname, "../../../fixtures");

export const lookupVendor: AgentTool = {
  name: "lookup-vendor",
  label: "Lookup Vendor",
  description:
    "Look up a vendor by name. Returns the vendor's registration details, " +
    "compliance history, certifications, and contact information.",
  parameters: Type.Object({
    vendorName: Type.String({ description: "The name of the vendor to look up" }),
  }),
  execute: async (_toolCallId, params) => {
    const { vendorName } = params as { vendorName: string };
    console.log(`  [tool] lookup-vendor: searching for "${vendorName}"`);

    // Normalize for fixture matching
    const normalized = vendorName.toLowerCase().replace(/\s+/g, "-");
    const fixturePath = resolve(FIXTURES_DIR, `vendor-${normalized}.json`);

    try {
      const raw = readFileSync(fixturePath, "utf-8");
      const data = JSON.parse(raw) as Record<string, unknown>;
      console.log(`  [tool] lookup-vendor: found ${data.vendorName}`);
      const result = { success: true, found: true, ...data };
      return {
        content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        details: result,
      };
    } catch {
      console.log(`  [tool] lookup-vendor: vendor not found`);
      const result = {
        success: true,
        found: false,
        message: `No vendor record found for "${vendorName}"`,
      };
      return {
        content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        details: result,
      };
    }
  },
};
