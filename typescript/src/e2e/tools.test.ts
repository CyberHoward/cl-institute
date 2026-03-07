import { describe, it, expect } from "vitest";
import { createWebSearchTool, createFetchPageContentTool } from "./tools.js";

describe("createWebSearchTool", () => {
  it("returns an AgentTool with correct name and parameters", () => {
    const tool = createWebSearchTool("/fake/path/search.js");
    expect(tool.name).toBe("web-search");
    expect(tool.label).toBe("Web Search");
    expect(tool.parameters).toBeDefined();
    expect(tool.execute).toBeTypeOf("function");
  });
});

describe("createFetchPageContentTool", () => {
  it("returns an AgentTool with correct name and parameters", () => {
    const tool = createFetchPageContentTool("/fake/path/content.js");
    expect(tool.name).toBe("fetch-page-content");
    expect(tool.label).toBe("Fetch Page Content");
    expect(tool.parameters).toBeDefined();
    expect(tool.execute).toBeTypeOf("function");
  });
});
