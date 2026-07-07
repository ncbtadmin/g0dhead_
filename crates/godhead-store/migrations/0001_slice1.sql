-- Slice 1 — the store substrate (docs/dev/SLICE_01.md).
-- Envelope columns (A.1) are NOT NULL on every table: the store rejects
-- records missing any envelope field (SC-A06). All timestamps default to
-- now() and are never bound from the client (Law XII).

CREATE TABLE job_records (
    job_id          uuid PRIMARY KEY,
    agent_type      text NOT NULL CHECK (agent_type IN
                        ('SLAVE','AGGREGATOR','NOTARY','AUDITOR','RECONCILER','STUDENT','TEACHER')),
    auditor_name    text CHECK (auditor_name IN ('GABRIEL','LUCY')),
    tier            text CHECK (tier IN ('REGULAR','DEVOUT','CANON')),
    status          text NOT NULL DEFAULT 'PENDING' CHECK (status IN
                        ('PENDING','LEASED','RUNNING','WRITTEN','FLAGGED','TERMINATED','REFUSED')),
    attempt         int NOT NULL DEFAULT 1,
    input_refs      jsonb NOT NULL DEFAULT '[]',
    output_refs     jsonb NOT NULL DEFAULT '[]',
    env_ref         uuid,
    brief_ref       uuid,
    endpoint_alias  text,
    manual_version  text NOT NULL,
    -- Law XIV: no unbounded agent exists.
    max_wall_ms     bigint NOT NULL CHECK (max_wall_ms > 0),
    max_tool_calls  int NOT NULL CHECK (max_tool_calls > 0),
    max_tokens      bigint NOT NULL CHECK (max_tokens > 0),
    started_at      timestamptz,
    heartbeat_at    timestamptz,
    finished_at     timestamptz,
    revision        int NOT NULL DEFAULT 1,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

-- The generic write surface: outputs keyed (job_id, output_slot) so a
-- retried invocation converges instead of duplicating (Law I.3, SC-A03).
CREATE TABLE artifacts (
    job_id            uuid NOT NULL REFERENCES job_records(job_id),
    output_slot       text NOT NULL,
    payload           jsonb NOT NULL,
    -- Law VII.5: partials of a refused job are non-authoritative and
    -- invisible to readers.
    authoritative     boolean NOT NULL DEFAULT true,
    quarantine_marked boolean NOT NULL DEFAULT false,
    revision          int NOT NULL DEFAULT 1,
    schema_name       text NOT NULL,
    schema_version    text NOT NULL,
    produced_by       text NOT NULL,
    produced_at       timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (job_id, output_slot)
);

CREATE TABLE readiness_flags (
    flag_id         uuid PRIMARY KEY,
    job_id          uuid NOT NULL REFERENCES job_records(job_id),
    stage           text NOT NULL CHECK (length(stage) > 0),
    certifies       jsonb NOT NULL,
    validator       jsonb NOT NULL,
    status          text NOT NULL DEFAULT 'ACTIVE' CHECK (status IN
                        ('ACTIVE','CONSUMED','DISTRUSTED','SUPERSEDED')),
    revision        int NOT NULL DEFAULT 1,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now(),
    UNIQUE (job_id, stage)
);

CREATE TABLE refusal_records (
    refusal_id      uuid PRIMARY KEY,
    job_id          uuid NOT NULL REFERENCES job_records(job_id),
    law             text NOT NULL CHECK (law IN
                        ('I','II','III','IV','V','VI','VII','VIII','IX','X','XI','XII','XIII','XIV','XV')),
    reason          text NOT NULL CHECK (reason IN
                        ('SCHEMA_MISMATCH','VALIDATION_FAILED','FLAG_UNTRUSTED','TOOL_MALFORMED',
                         'TOOL_OUTPUT_INVALID','PROVENANCE_INCOMPLETE','OVERRIDE_CONFLICT',
                         'GATE_BYPASS_ATTEMPT','ENV_INVALID','LEASE_CONFLICT','BUDGET_EXCEEDED',
                         'LAW_CONFLICT')),
    subject_refs    jsonb NOT NULL DEFAULT '[]',
    detail          text NOT NULL,
    preserved_refs  jsonb NOT NULL DEFAULT '[]',
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE log_snapshots (
    log_id          uuid PRIMARY KEY,
    -- Store sequence establishes order (Law XII.2), never wall-clock.
    seq             bigint GENERATED ALWAYS AS IDENTITY,
    subject_ref     text NOT NULL,
    event           text NOT NULL CHECK (event IN
                        ('INTAKE_RAW_COPIED','NORMALIZED','EMBEDDED','LINK_DRAWN','LINK_SEVERED',
                         'WEIGHT_RECALC','POSTULANT_EMERGED','AUDIT_OPENED','REPORT_FILED',
                         'PROPOSAL_FILED','COMMITTED','DECOMMISSIONED','OVERRIDE_LAID',
                         'PETITION_OPENED','PETITION_RESOLVED','ADMITTED','REJECTED','REFUSAL',
                         'VIOLATION','JOB_TRANSITION')),
    payload         jsonb NOT NULL DEFAULT '{}',
    prior_ref       uuid REFERENCES log_snapshots(log_id),
    severity        text NOT NULL CHECK (severity IN ('info','warning','violation','suppressed')),
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE lease_records (
    lease_id        uuid PRIMARY KEY,
    subject_ref     uuid NOT NULL,
    job_id          uuid NOT NULL REFERENCES job_records(job_id),
    ttl_ms          bigint NOT NULL CHECK (ttl_ms > 0),
    active          boolean NOT NULL DEFAULT true,
    acquired_at     timestamptz NOT NULL DEFAULT now(),
    heartbeat_at    timestamptz NOT NULL DEFAULT now(),
    expires_at      timestamptz NOT NULL,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

-- Law XI.1: one hand on one record. The partial unique index is the
-- race-proof arbiter: two concurrent acquisitions cannot both insert.
CREATE UNIQUE INDEX one_active_lease_per_subject ON lease_records (subject_ref) WHERE active;

CREATE TABLE config_constants (
    key             text PRIMARY KEY,
    tier            text NOT NULL CHECK (tier IN ('SOVEREIGN','OPERATIONAL')),
    value           jsonb NOT NULL,
    revision        int NOT NULL DEFAULT 1,
    changed_at      timestamptz NOT NULL DEFAULT now(),
    changed_by      text NOT NULL,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

-- Law III.4 / V.1: flags, logs, and refusals are never deleted — enforced
-- at the substrate so no writer, of any kind, has a delete path (SC-B03).
CREATE FUNCTION godhead_forbid_delete() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'deletion forbidden on %: records are superseded, never destroyed (Laws III.4, V.1)',
        TG_TABLE_NAME;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER no_delete_flags BEFORE DELETE ON readiness_flags
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_logs BEFORE DELETE ON log_snapshots
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_refusals BEFORE DELETE ON refusal_records
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- Day-one operational constants (A.14). Sovereign constants
-- (coherence_threshold, quarantine_retention_days) are [TBD empirically]
-- and deliberately not seeded: a placeholder sovereign value would be a
-- decision the sovereign never made.
INSERT INTO config_constants (key, tier, value, changed_by, schema_name, schema_version, produced_by) VALUES
    ('tool_repair_attempts',   'OPERATIONAL', '2',     'deployment:migration-0001', 'ConfigConstant', '1.0.0', 'deployment'),
    ('lease_ttl_ms',           'OPERATIONAL', '30000', 'deployment:migration-0001', 'ConfigConstant', '1.0.0', 'deployment'),
    ('bias_skew_threshold',    'OPERATIONAL', '0.50',  'deployment:migration-0001', 'ConfigConstant', '1.0.0', 'deployment'),
    ('bias_pattern_window',    'OPERATIONAL', '20',    'deployment:migration-0001', 'ConfigConstant', '1.0.0', 'deployment'),
    ('bias_pattern_threshold', 'OPERATIONAL', '0.60',  'deployment:migration-0001', 'ConfigConstant', '1.0.0', 'deployment');
