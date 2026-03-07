import { execFile as execFileCb } from "node:child_process";
import { promisify } from "node:util";
import { Type } from "@mariozechner/pi-ai";
import type { AgentTool } from "@mariozechner/pi-agent-core";

const execFile = promisify(execFileCb);

export function createWebSearchTool(searchScriptPath: string): AgentTool {
  return {
    name: "web-search",
    label: "Web Search",
    description:
      "Search the web for information. Returns titles, links, snippets, and optionally full page content as markdown.",
    parameters: Type.Object({
      query: Type.String({ description: "Search query" }),
      num_results: Type.Optional(
        Type.Number({ description: "Number of results, default 5, max 20" }),
      ),
      include_content: Type.Optional(
        Type.Boolean({ description: "Fetch full page content as markdown" }),
      ),
    }),
    execute: async (_toolCallId, params) => {
      const args = [searchScriptPath, params.query];
      if (params.num_results) args.push("-n", String(params.num_results));
      if (params.include_content) args.push("--content");
      const { stdout } = await execFile("node", args, {
        env: { ...process.env },
        timeout: 30_000,
      });
      return {
        content: [{ type: "text" as const, text: stdout }],
        details: { raw: stdout },
      };
    },
  };
}

export function createFetchPageContentTool(contentScriptPath: string): AgentTool {
  return {
    name: "fetch-page-content",
    label: "Fetch Page Content",
    description:
      "Fetch a URL and extract readable content as markdown. Use to get detailed info from a venue's website.",
    parameters: Type.Object({
      url: Type.String({ description: "URL to fetch" }),
    }),
    execute: async (_toolCallId, params) => {
      const { stdout } = await execFile("node", [contentScriptPath, params.url], {
        env: { ...process.env },
        timeout: 15_000,
      });
      return {
        content: [{ type: "text" as const, text: stdout }],
        details: { raw: stdout },
      };
    },
  };
}
