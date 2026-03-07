import { describe, it, expect, afterEach } from "vitest";
import { Engine } from "../core/engine.js";
import { validateNet } from "../core/validate.js";
import { buildVenueScoutNet } from "./venue-scout-net.js";

describe("buildVenueScoutNet", () => {
  let engine: Engine;

  afterEach(() => {
    engine.close();
  });

  it("creates a valid net with 4 places and 3 transitions", () => {
    engine = new Engine(":memory:");
    const { netId, institutionId, roles, actors } = buildVenueScoutNet(engine);

    const { places, transitions } = engine.getNetWithGraph(netId);
    expect(places).toHaveLength(4);
    expect(transitions).toHaveLength(3);

    // Check place IDs
    const placeIds = places.map((p) => p.id);
    expect(placeIds).toContain("event-submitted");
    expect(placeIds).toContain("proposals-ready");
    expect(placeIds).toContain("outreach-approved");
    expect(placeIds).toContain("outreach-sent");

    // Check transition IDs and modes
    const research = transitions.find((t) => t.id === "research-venues");
    expect(research?.mode).toBe("agentic");
    expect(research?.consumes).toEqual(["event-submitted"]);
    expect(research?.produces).toEqual(["proposals-ready"]);

    const review = transitions.find((t) => t.id === "review-proposals");
    expect(review?.mode).toBe("judgment");

    const send = transitions.find((t) => t.id === "send-outreach");
    expect(send?.mode).toBe("deterministic");

    // Validate net structure
    const validation = validateNet(engine, netId);
    expect(validation.is_valid).toBe(true);
  });

  it("returns actor IDs for the scout agent and human reviewer", () => {
    engine = new Engine(":memory:");
    const { actors } = buildVenueScoutNet(engine);

    expect(actors.scout).toBeTruthy();
    expect(actors.reviewer).toBeTruthy();
  });
});
