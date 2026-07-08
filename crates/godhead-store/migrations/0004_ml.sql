-- Slice 4 — weights & the ML floor (docs/dev/SLICE_04.md).

CREATE EXTENSION IF NOT EXISTS vector;

-- One persisted vector per node (doc 3 §2.2): never recomputed when it
-- can be read.
CREATE TABLE embeddings (
    node_id         uuid PRIMARY KEY REFERENCES nodes(node_id),
    embedding       vector(256) NOT NULL,
    embedder_alias  text NOT NULL,
    dims            int NOT NULL CHECK (dims = 256),
    revision        int NOT NULL DEFAULT 1,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

-- The bonds (doc 3 §2.3): first-class records, never written into file
-- bodies. Canonical ordering source < target — one row per pair.
CREATE TABLE links (
    link_id         uuid PRIMARY KEY,
    source_ref      uuid NOT NULL REFERENCES nodes(node_id),
    target_ref      uuid NOT NULL REFERENCES nodes(node_id),
    similarity      real NOT NULL CHECK (similarity >= -1 AND similarity <= 1),
    weight          real NOT NULL DEFAULT 0,
    category        text NOT NULL,
    user_overridden boolean NOT NULL DEFAULT false,
    revision        int NOT NULL DEFAULT 1,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now(),
    UNIQUE (source_ref, target_ref),
    CHECK (source_ref < target_ref)
);
CREATE INDEX links_by_category ON links (category);

-- Recalculation eligibility (doc 4 §5.2): ingestion marks, triggers execute.
CREATE TABLE rebalance_state (
    category        text PRIMARY KEY,
    eligible        boolean NOT NULL DEFAULT false,
    marked_at       timestamptz,
    last_recalc_at  timestamptz,
    config_rev      int,
    revision        int NOT NULL DEFAULT 1,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

-- Operational dials (A.14). coherence_threshold stays sovereign and
-- unseeded: a placeholder sovereign value would be a decision the
-- sovereign never made.
INSERT INTO config_constants (key, tier, value, changed_by, schema_name, schema_version, produced_by) VALUES
    ('link_similarity_threshold', 'OPERATIONAL', '0.30',                'deployment:migration-0004', 'ConfigConstant', '1.0.0', 'deployment'),
    ('weight_mode',               'OPERATIONAL', '"floor"',             'deployment:migration-0004', 'ConfigConstant', '1.0.0', 'deployment'),
    ('rebalance_trigger',         'OPERATIONAL', '{"kind": "manual"}',  'deployment:migration-0004', 'ConfigConstant', '1.0.0', 'deployment');
