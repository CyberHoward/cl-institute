import { randomUUID } from "node:crypto";
import { DB } from "./db.js";
import type {
  Institution,
  Role,
  Actor,
  ActorType,
  Net,
  Place,
  Transition,
  TransitionMode,
  DecisionType,
  Postconditions,
  EvidenceRequirement,
  Arc,
  ArcDirection,
  JsonSchema,
  Token,
  WorkflowInstance,
  InstanceStatus,
  Policy,
  PolicyStrength,
  FiringResult,
  Evidence,
  PendingJudgment,
  AuditEntry,
} from "./types.js";

export interface TransitionDef {
  id: string;
  consumes: string[];
  produces: string[];
  guard?: string;
  intent: string;
  mode: TransitionMode;
  decision_type?: DecisionType;
  requires_authority: number;
  authorized_roles?: string[];
  input_schema?: JsonSchema;
  output_schema?: JsonSchema;
  context_sources: string[];
  postconditions: Postconditions;
  evidence_requirements: EvidenceRequirement[];
  available_tools: string[];
  timeout?: number;
}

export class Engine {
  private readonly db: DB;

  constructor(dbPath: string) {
    this.db = new DB(dbPath);
  }

  // -----------------------------------------------------------------------
  // Institution
  // -----------------------------------------------------------------------

  createInstitution(name: string, description?: string): Institution {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO institutions (id, name, description, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)`,
      )
      .run(id, name, description ?? null, now, now);
    return { id, name, description, created_at: now, updated_at: now };
  }

  getInstitution(id: string): Institution {
    const row = this.db.sqlite
      .prepare("SELECT * FROM institutions WHERE id = ?")
      .get(id) as Record<string, unknown> | undefined;
    if (!row) throw new Error(`Institution not found: ${id}`);
    return {
      id: row["id"] as string,
      name: row["name"] as string,
      description: (row["description"] as string) ?? undefined,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    };
  }

  // -----------------------------------------------------------------------
  // Roles
  // -----------------------------------------------------------------------

  createRole(
    institutionId: string,
    name: string,
    authorityLevel: number,
    description?: string,
  ): Role {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO roles (id, institution_id, name, description, authority_level, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(id, institutionId, name, description ?? null, authorityLevel, now, now);
    return {
      id,
      institution_id: institutionId,
      name,
      description,
      authority_level: authorityLevel,
      created_at: now,
      updated_at: now,
    };
  }

  // -----------------------------------------------------------------------
  // Actors
  // -----------------------------------------------------------------------

  createActor(institutionId: string, name: string, type: ActorType): Actor {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO actors (id, institution_id, name, type, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)`,
      )
      .run(id, institutionId, name, type, now, now);
    return {
      id,
      institution_id: institutionId,
      name,
      type,
      created_at: now,
      updated_at: now,
    };
  }

  listActors(institutionId: string): Actor[] {
    const rows = this.db.sqlite
      .prepare("SELECT * FROM actors WHERE institution_id = ?")
      .all(institutionId) as Array<Record<string, unknown>>;
    return rows.map((row) => ({
      id: row["id"] as string,
      institution_id: row["institution_id"] as string,
      name: row["name"] as string,
      type: row["type"] as ActorType,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    }));
  }

  assignRole(actorId: string, roleId: string): void {
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO actor_roles (actor_id, role_id, created_at)
         VALUES (?, ?, ?)`,
      )
      .run(actorId, roleId, now);
  }

  getActorRoles(actorId: string): Role[] {
    const rows = this.db.sqlite
      .prepare(
        `SELECT r.* FROM roles r
         JOIN actor_roles ar ON ar.role_id = r.id
         WHERE ar.actor_id = ?`,
      )
      .all(actorId) as Array<Record<string, unknown>>;
    return rows.map((row) => ({
      id: row["id"] as string,
      institution_id: row["institution_id"] as string,
      name: row["name"] as string,
      description: (row["description"] as string) ?? undefined,
      authority_level: row["authority_level"] as number,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    }));
  }

  getActorAuthority(actorId: string): number {
    const roles = this.getActorRoles(actorId);
    if (roles.length === 0) return 0;
    return Math.max(...roles.map((r) => r.authority_level));
  }

  // -----------------------------------------------------------------------
  // Net
  // -----------------------------------------------------------------------

  createNet(institutionId: string, name: string, domain?: string, description?: string): Net {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO nets (id, institution_id, domain, name, description, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(id, institutionId, domain ?? null, name, description ?? null, now, now);
    return { id, institution_id: institutionId, domain, name, description, created_at: now, updated_at: now };
  }

  addPlace(netId: string, id: string, description: string, schema?: JsonSchema): Place {
    this.db.sqlite
      .prepare(
        `INSERT INTO places (id, net_id, description, schema_json)
         VALUES (?, ?, ?, ?)`,
      )
      .run(id, netId, description, schema ? JSON.stringify(schema) : null);
    return { id, net_id: netId, description, schema };
  }

  addTransition(netId: string, def: TransitionDef): Transition {
    // Validate that all consumed and produced places exist
    const allPlaces = def.consumes.concat(def.produces);
    for (const placeId of allPlaces) {
      const exists = this.db.sqlite
        .prepare("SELECT 1 FROM places WHERE id = ? AND net_id = ?")
        .get(placeId, netId);
      if (!exists) {
        throw new Error(`Place '${placeId}' does not exist in net '${netId}'`);
      }
    }

    this.db.sqlite
      .prepare(
        `INSERT INTO transitions (
          id, net_id, guard, intent, mode, decision_type, requires_authority,
          authorized_roles_json, input_schema_json, output_schema_json,
          context_sources_json, postconditions_json, evidence_requirements_json,
          available_tools_json, timeout
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(
        def.id,
        netId,
        def.guard ?? null,
        def.intent,
        def.mode,
        def.decision_type ?? null,
        def.requires_authority,
        def.authorized_roles ? JSON.stringify(def.authorized_roles) : null,
        def.input_schema ? JSON.stringify(def.input_schema) : null,
        def.output_schema ? JSON.stringify(def.output_schema) : null,
        JSON.stringify(def.context_sources),
        JSON.stringify(def.postconditions),
        JSON.stringify(def.evidence_requirements),
        JSON.stringify(def.available_tools),
        def.timeout ?? null,
      );

    // Create arcs for consumes (place → transition)
    for (const placeId of def.consumes) {
      this.db.sqlite
        .prepare(
          `INSERT INTO arcs (id, net_id, place_id, transition_id, direction)
           VALUES (?, ?, ?, ?, ?)`,
        )
        .run(randomUUID(), netId, placeId, def.id, "place_to_transition");
    }

    // Create arcs for produces (transition → place)
    for (const placeId of def.produces) {
      this.db.sqlite
        .prepare(
          `INSERT INTO arcs (id, net_id, place_id, transition_id, direction)
           VALUES (?, ?, ?, ?, ?)`,
        )
        .run(randomUUID(), netId, placeId, def.id, "transition_to_place");
    }

    return {
      id: def.id,
      net_id: netId,
      consumes: def.consumes,
      produces: def.produces,
      guard: def.guard,
      intent: def.intent,
      mode: def.mode,
      decision_type: def.decision_type,
      requires_authority: def.requires_authority,
      authorized_roles: def.authorized_roles,
      input_schema: def.input_schema,
      output_schema: def.output_schema,
      context_sources: def.context_sources,
      postconditions: def.postconditions,
      evidence_requirements: def.evidence_requirements,
      available_tools: def.available_tools,
      timeout: def.timeout,
    };
  }

  getArcs(netId: string): Arc[] {
    const rows = this.db.sqlite
      .prepare("SELECT * FROM arcs WHERE net_id = ?")
      .all(netId) as Array<Record<string, unknown>>;
    return rows.map((row) => ({
      id: row["id"] as string,
      net_id: row["net_id"] as string,
      place_id: row["place_id"] as string,
      transition_id: row["transition_id"] as string,
      direction: row["direction"] as ArcDirection,
    }));
  }

  getNetWithGraph(netId: string): { net: Net; places: Place[]; transitions: Transition[] } {
    const netRow = this.db.sqlite
      .prepare("SELECT * FROM nets WHERE id = ?")
      .get(netId) as Record<string, unknown> | undefined;
    if (!netRow) throw new Error(`Net not found: ${netId}`);

    const net: Net = {
      id: netRow["id"] as string,
      institution_id: netRow["institution_id"] as string,
      domain: (netRow["domain"] as string) ?? undefined,
      name: netRow["name"] as string,
      description: (netRow["description"] as string) ?? undefined,
      created_at: netRow["created_at"] as string,
      updated_at: netRow["updated_at"] as string,
    };

    const placeRows = this.db.sqlite
      .prepare("SELECT * FROM places WHERE net_id = ?")
      .all(netId) as Array<Record<string, unknown>>;
    const places: Place[] = placeRows.map((row) => ({
      id: row["id"] as string,
      net_id: row["net_id"] as string,
      description: row["description"] as string,
      schema: row["schema_json"] ? JSON.parse(row["schema_json"] as string) : undefined,
    }));

    const transitionRows = this.db.sqlite
      .prepare("SELECT * FROM transitions WHERE net_id = ?")
      .all(netId) as Array<Record<string, unknown>>;

    const arcs = this.getArcs(netId);
    const transitions: Transition[] = transitionRows.map((row) => {
      const tId = row["id"] as string;
      const consumeArcs = arcs.filter(
        (a) => a.transition_id === tId && a.direction === "place_to_transition",
      );
      const produceArcs = arcs.filter(
        (a) => a.transition_id === tId && a.direction === "transition_to_place",
      );
      return {
        id: tId,
        net_id: row["net_id"] as string,
        consumes: consumeArcs.map((a) => a.place_id),
        produces: produceArcs.map((a) => a.place_id),
        guard: (row["guard"] as string) ?? undefined,
        intent: row["intent"] as string,
        mode: row["mode"] as TransitionMode,
        decision_type: (row["decision_type"] as DecisionType) ?? undefined,
        requires_authority: row["requires_authority"] as number,
        authorized_roles: row["authorized_roles_json"]
          ? JSON.parse(row["authorized_roles_json"] as string)
          : undefined,
        input_schema: row["input_schema_json"]
          ? JSON.parse(row["input_schema_json"] as string)
          : undefined,
        output_schema: row["output_schema_json"]
          ? JSON.parse(row["output_schema_json"] as string)
          : undefined,
        context_sources: JSON.parse(row["context_sources_json"] as string),
        postconditions: JSON.parse(row["postconditions_json"] as string),
        evidence_requirements: JSON.parse(row["evidence_requirements_json"] as string),
        available_tools: JSON.parse(row["available_tools_json"] as string),
        timeout: (row["timeout"] as number) ?? undefined,
      };
    });

    return { net, places, transitions };
  }

  // -----------------------------------------------------------------------
  // Policies
  // -----------------------------------------------------------------------

  attachPolicy(
    institutionId: string,
    scope: string,
    strength: PolicyStrength,
    text: string,
    metadata?: Record<string, unknown>,
  ): Policy {
    const id = randomUUID();
    const now = new Date().toISOString();
    this.db.sqlite
      .prepare(
        `INSERT INTO policies (id, institution_id, scope, strength, text, metadata_json, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(id, institutionId, scope, strength, text, metadata ? JSON.stringify(metadata) : null, now, now);
    return { id, institution_id: institutionId, scope, strength, text, metadata, created_at: now, updated_at: now };
  }

  getPolicies(scope: string): Policy[] {
    // Build list of matching scopes: exact, parent wildcards, global
    const matchScopes: string[] = [scope];
    const parts = scope.split(".");
    for (let i = parts.length - 1; i >= 1; i--) {
      matchScopes.push(parts.slice(0, i).join(".") + ".*");
    }
    matchScopes.push("*");

    const placeholders = matchScopes.map(() => "?").join(", ");
    const rows = this.db.sqlite
      .prepare(`SELECT * FROM policies WHERE scope IN (${placeholders})`)
      .all(...matchScopes) as Array<Record<string, unknown>>;

    const strengthOrder: Record<string, number> = {
      constraint: 0,
      procedure: 1,
      preference: 2,
      context: 3,
    };

    const scopeSpecificity = (s: string): number => {
      const idx = matchScopes.indexOf(s);
      return idx === -1 ? matchScopes.length : idx;
    };

    const policies: Policy[] = rows.map((row) => ({
      id: row["id"] as string,
      institution_id: row["institution_id"] as string,
      scope: row["scope"] as string,
      strength: row["strength"] as PolicyStrength,
      text: row["text"] as string,
      metadata: row["metadata_json"] ? JSON.parse(row["metadata_json"] as string) : undefined,
      created_at: row["created_at"] as string,
      updated_at: row["updated_at"] as string,
    }));

    policies.sort((a, b) => {
      const strengthDiff = (strengthOrder[a.strength] ?? 99) - (strengthOrder[b.strength] ?? 99);
      if (strengthDiff !== 0) return strengthDiff;
      return scopeSpecificity(a.scope) - scopeSpecificity(b.scope);
    });

    return policies;
  }

  // -----------------------------------------------------------------------
  // Lifecycle
  // -----------------------------------------------------------------------

  close(): void {
    this.db.close();
  }
}
