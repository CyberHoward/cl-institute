import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { Marking } from "./net/types.js";
import { vendorOnboardingNet } from "./net/vendor-onboarding.js";
import { ContextStore } from "./context/store.js";
import { runNet } from "./net/engine.js";
import { lookupVendor } from "./tools/lookup-vendor.js";
import { sendNotification } from "./tools/send-notification.js";
import { generateDocument } from "./tools/generate-document.js";

// Parse CLI args
const args = process.argv.slice(2);
const vendorFlag = args.indexOf("--vendor");
const vendorName = vendorFlag !== -1 ? args[vendorFlag + 1] : "acme";
const verbose = args.includes("--verbose");

async function main() {
  console.log(`\nVendor Onboarding Spike`);
  console.log(`Vendor: ${vendorName}`);
  console.log(`Verbose: ${verbose}\n`);

  // Load risk policy as context
  const riskPolicy = readFileSync(
    resolve(import.meta.dirname, "../../fixtures/risk-policy.md"),
    "utf-8",
  );

  // Set up context store with initial data
  const contextStore = new ContextStore();
  contextStore.set("vendor-request", {
    vendorName,
    requestedBy: "procurement@example.com",
    requestDate: new Date().toISOString(),
    purpose: "Software development services",
  });
  contextStore.set("risk-policy", riskPolicy);

  // Set up initial marking: one token in "request-submitted"
  const initialMarking: Marking = new Map([
    [
      "request-submitted",
      {
        count: 1,
        payload: {
          vendorName,
          requestedBy: "procurement@example.com",
        },
      },
    ],
  ]);

  // Register all tools
  const tools = [lookupVendor, sendNotification, generateDocument];

  // Run the net
  const startTime = Date.now();
  const result = await runNet(vendorOnboardingNet, initialMarking, contextStore, {
    tools,
    maxSteps: 10,
    verbose,
  });
  const totalDuration = Date.now() - startTime;

  // Print summary
  console.log("\n========================================");
  console.log("EXECUTION SUMMARY");
  console.log("========================================");
  console.log(`Status: ${result.status}`);
  console.log(`Total duration: ${totalDuration}ms`);
  console.log(`Transitions fired: ${result.logs.filter((l) => l.status === "fired").length}`);
  console.log(`Transitions failed: ${result.logs.filter((l) => l.status === "failed").length}`);
  console.log(`\nFinal marking:`);
  for (const [place, token] of Object.entries(result.finalMarking)) {
    console.log(`  ${place}: ${token.count} token(s)`);
  }

  console.log(`\nTransition details:`);
  for (const log of result.logs) {
    console.log(`\n  ${log.transitionId} [${log.status}] (${log.durationMs}ms)`);
    console.log(`    Tools used: ${log.agentActions.map((a) => a.toolName).join(", ") || "none"}`);
    console.log(`    Postconditions:`);
    for (const [pc, met] of Object.entries(log.postconditionResults)) {
      console.log(`      ${met ? "✓" : "✗"} ${pc}`);
    }
  }

  // Dump final context for inspection
  if (verbose) {
    console.log(`\nFinal context store:`);
    console.log(JSON.stringify(contextStore.dump(), null, 2));
  }
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
