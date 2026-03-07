import * as readline from "node:readline/promises";
import { stdin, stdout } from "node:process";
import type { HumanPromptContext, HumanDecision, HumanInput } from "./types.js";
import type { VenueProposal } from "./types.js";

interface ProposalsShape {
  venues: VenueProposal[];
}

export class ReadlineHumanInput implements HumanInput {
  async prompt(context: HumanPromptContext): Promise<HumanDecision> {
    const rl = readline.createInterface({ input: stdin, output: stdout });

    try {
      console.log("\n" + "=".repeat(60));
      console.log(`📋 JUDGMENT REQUIRED: ${context.intent}`);
      console.log("=".repeat(60));

      // Display proposals
      const proposals = context.token_payloads[0] as unknown as ProposalsShape | undefined;
      const venues = proposals?.venues ?? [];

      if (venues.length === 0) {
        console.log("\nNo venues were found.");
        const notes = await rl.question("\nAny notes? (press Enter to skip): ");
        return {
          decision: { approved_venues: [], reviewer_notes: notes || undefined },
          reasoning: "No venues to approve",
        };
      }

      for (let i = 0; i < venues.length; i++) {
        const v = venues[i]!;
        console.log(`\n--- Venue ${i + 1}: ${v.name} ---`);
        console.log(`Why: ${v.why}`);
        console.log(`Capacity: ${v.capacity}`);
        console.log(`Price: ${v.price_range}`);
        console.log(`Contact: ${v.contact_email}`);
        console.log(`Website: ${v.website}`);
        console.log(`\nDraft email:\n${v.draft_email}`);
      }

      // Ask which to approve
      const indices = venues.map((_v, i) => i + 1).join(", ");
      const selection = await rl.question(
        `\nWhich venues to contact? (${indices}, comma-separated, or 'none'): `,
      );

      if (selection.trim().toLowerCase() === "none") {
        return {
          decision: { approved_venues: [], reviewer_notes: "Reviewer rejected all venues" },
          reasoning: "None selected",
        };
      }

      const selected = selection
        .split(",")
        .map((s) => parseInt(s.trim(), 10) - 1)
        .filter((i) => i >= 0 && i < venues.length);

      const approvedVenues: Array<{ name: string; final_email: string; contact_email: string }> = [];

      for (const idx of selected) {
        const v = venues[idx]!;
        const editChoice = await rl.question(
          `\nEdit email for ${v.name}? (y/N): `,
        );

        let finalEmail = v.draft_email;
        if (editChoice.trim().toLowerCase() === "y") {
          console.log("Enter new email text (end with a blank line):");
          const lines: string[] = [];
          while (true) {
            const line = await rl.question("");
            if (line === "") break;
            lines.push(line);
          }
          finalEmail = lines.join("\n");
        }

        approvedVenues.push({
          name: v.name,
          final_email: finalEmail,
          contact_email: v.contact_email,
        });
      }

      const notes = await rl.question("\nAny reviewer notes? (press Enter to skip): ");

      return {
        decision: {
          approved_venues: approvedVenues,
          reviewer_notes: notes || undefined,
        },
        reasoning: `Approved ${approvedVenues.length} of ${venues.length} venues`,
      };
    } finally {
      rl.close();
    }
  }
}
