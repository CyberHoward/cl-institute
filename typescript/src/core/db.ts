import Database from "better-sqlite3";

const SCHEMA = `
  CREATE TABLE IF NOT EXISTS institutions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    name TEXT NOT NULL,
    description TEXT,
    authority_level INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(institution_id, name)
  );

  CREATE TABLE IF NOT EXISTS actors (
    id TEXT PRIMARY KEY,
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('human', 'agent')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS actor_roles (
    actor_id TEXT NOT NULL REFERENCES actors(id),
    role_id TEXT NOT NULL REFERENCES roles(id),
    created_at TEXT NOT NULL,
    PRIMARY KEY (actor_id, role_id)
  );

  CREATE TABLE IF NOT EXISTS nets (
    id TEXT PRIMARY KEY,
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    domain TEXT,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS places (
    id TEXT PRIMARY KEY,
    net_id TEXT NOT NULL REFERENCES nets(id),
    description TEXT NOT NULL,
    schema_json TEXT
  );

  CREATE TABLE IF NOT EXISTS transitions (
    id TEXT PRIMARY KEY,
    net_id TEXT NOT NULL REFERENCES nets(id),

    -- CPN core
    guard TEXT,

    -- Institutional semantics
    intent TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('deterministic', 'judgment', 'agentic')),
    decision_type TEXT,
    requires_authority INTEGER NOT NULL DEFAULT 0,
    authorized_roles_json TEXT,

    -- Data flow
    input_schema_json TEXT,
    output_schema_json TEXT,
    context_sources_json TEXT NOT NULL DEFAULT '[]',

    -- Execution contract
    postconditions_json TEXT NOT NULL,
    evidence_requirements_json TEXT NOT NULL DEFAULT '[]',
    available_tools_json TEXT NOT NULL DEFAULT '[]',
    timeout INTEGER
  );

  CREATE TABLE IF NOT EXISTS arcs (
    id TEXT PRIMARY KEY,
    net_id TEXT NOT NULL REFERENCES nets(id),
    place_id TEXT NOT NULL REFERENCES places(id),
    transition_id TEXT NOT NULL REFERENCES transitions(id),
    direction TEXT NOT NULL CHECK(direction IN ('place_to_transition', 'transition_to_place'))
  );

  CREATE TABLE IF NOT EXISTS policies (
    id TEXT PRIMARY KEY,
    institution_id TEXT NOT NULL REFERENCES institutions(id),
    scope TEXT NOT NULL,
    strength TEXT NOT NULL CHECK(strength IN ('constraint', 'procedure', 'preference', 'context')),
    text TEXT NOT NULL,
    metadata_json TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS instances (
    id TEXT PRIMARY KEY,
    net_id TEXT NOT NULL REFERENCES nets(id),
    status TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running', 'completed', 'stuck', 'suspended')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS tokens (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES instances(id),
    place_id TEXT NOT NULL REFERENCES places(id),
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS audit_entries (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES instances(id),
    timestamp TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    action TEXT NOT NULL,
    actor_json TEXT NOT NULL,
    transition_id TEXT,
    marking_before_json TEXT,
    marking_after_json TEXT,
    evidence_json TEXT,
    reasoning TEXT,
    prev_hash TEXT NOT NULL,
    entry_hash TEXT NOT NULL
  );

  CREATE INDEX IF NOT EXISTS idx_tokens_instance ON tokens(instance_id);
  CREATE INDEX IF NOT EXISTS idx_tokens_place ON tokens(place_id);
  CREATE INDEX IF NOT EXISTS idx_audit_instance ON audit_entries(instance_id);
  CREATE INDEX IF NOT EXISTS idx_audit_sequence ON audit_entries(instance_id, sequence);
  CREATE INDEX IF NOT EXISTS idx_policies_scope ON policies(scope);
  CREATE INDEX IF NOT EXISTS idx_arcs_transition ON arcs(transition_id);
  CREATE INDEX IF NOT EXISTS idx_arcs_place ON arcs(place_id);
`;

export class DB {
  readonly sqlite: Database.Database;

  constructor(path: string) {
    this.sqlite = new Database(path);
    this.sqlite.pragma("journal_mode = WAL");
    this.sqlite.pragma("foreign_keys = ON");
    this.sqlite.exec(SCHEMA);
  }

  listTables(): string[] {
    const rows = this.sqlite
      .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
      .all() as Array<{ name: string }>;
    return rows.map((r) => r.name);
  }

  close(): void {
    this.sqlite.close();
  }
}
