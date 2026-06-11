CREATE TABLE IF NOT EXISTS character_trait_deltas (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  character_id TEXT NOT NULL,
  source_event_id TEXT NOT NULL,
  trait_name TEXT NOT NULL,
  delta REAL NOT NULL,
  reason TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE,
  FOREIGN KEY (source_event_id) REFERENCES events(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_character_trait_deltas_project_character
ON character_trait_deltas(project_id, character_id, deleted_at, created_at);

CREATE INDEX IF NOT EXISTS idx_character_trait_deltas_event
ON character_trait_deltas(source_event_id, deleted_at);
