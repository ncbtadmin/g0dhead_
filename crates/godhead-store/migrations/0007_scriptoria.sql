-- Slice 7 — scriptoria & titles (docs/dev/SLICE_07.md): Laws IX–X floor.

-- New v1 events for the environment floor.
ALTER TABLE log_snapshots DROP CONSTRAINT log_snapshots_event_check;
ALTER TABLE log_snapshots ADD CONSTRAINT log_snapshots_event_check CHECK (event IN
    ('INTAKE_RAW_COPIED','NORMALIZED','EMBEDDED','LINK_DRAWN','LINK_SEVERED',
     'WEIGHT_RECALC','POSTULANT_EMERGED','AUDIT_OPENED','REPORT_FILED',
     'PROPOSAL_FILED','COMMITTED','DECOMMISSIONED','OVERRIDE_LAID',
     'PETITION_OPENED','PETITION_RESOLVED','ADMITTED','REJECTED','REFUSAL',
     'VIOLATION','JOB_TRANSITION','CLASSIFIED','AMENDED','PROPOSAL_RESOLVED',
     'ENV_ESTABLISHED','ENV_ORPHANED','PAIRING_FORMED'));

CREATE TABLE environments (
    env_id          uuid PRIMARY KEY,
    kind            text NOT NULL CHECK (kind IN ('TEACHER','STUDENT')),
    matrix_ref      uuid NOT NULL REFERENCES matrices(matrix_id),
    -- Regulars establish no environment (X.1): DEVOUT or CANON only.
    tier            text NOT NULL CHECK (tier IN ('DEVOUT','CANON')),
    title           text NOT NULL,
    name            text NOT NULL,
    established_by  uuid NOT NULL REFERENCES job_records(job_id),
    established_at  timestamptz NOT NULL DEFAULT now(),
    status          text NOT NULL DEFAULT 'LIVE' CHECK (status IN ('LIVE','ORPHANED','DISSOLVED')),
    revision        int NOT NULL DEFAULT 1,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE environment_items (
    env_id          uuid NOT NULL REFERENCES environments(env_id),
    item_ref        uuid NOT NULL,
    provenance      jsonb NOT NULL,
    -- The Pairing Exception grants flagged handoff artifacts only (IX.5).
    flagged         boolean NOT NULL DEFAULT false,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (env_id, item_ref)
);

CREATE TABLE pairings (
    pairing_id      uuid PRIMARY KEY,
    kind            text NOT NULL CHECK (kind IN ('DEVOUT_ASSIGNMENT','CANONICAL_INSTRUCTION')),
    teacher_env_ref uuid NOT NULL REFERENCES environments(env_id),
    student_env_ref uuid NOT NULL REFERENCES environments(env_id),
    matrix_ref      uuid NOT NULL REFERENCES matrices(matrix_id),
    formed_at       timestamptz NOT NULL DEFAULT now(),
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now(),
    UNIQUE (teacher_env_ref, student_env_ref)
);

-- Authorship-provenance persists for the life of the environment (X.1);
-- the record is history, never destroyed.
CREATE TRIGGER no_delete_environments BEFORE DELETE ON environments
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_pairings BEFORE DELETE ON pairings
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- The conferral constants (A.14, operational): the name roster and the
-- honorific set. Conferrals are immutable snapshots — roster edits cannot
-- rewrite an established title/name (X.4).
INSERT INTO config_constants (key, tier, value, changed_by, schema_name, schema_version, produced_by) VALUES
    ('name_roster', 'OPERATIONAL',
     '["Miroslav","Vesna","Dragan","Ludmila","Bogdan","Zorka","Casimir","Radomir","Slavena","Tihomir"]',
     'deployment:migration-0007', 'ConfigConstant', '1.0.0', 'deployment'),
    ('honorific_set', 'OPERATIONAL',
     '{"teacher": {"DEVOUT": "Professor", "CANON": "Doctor"}, "student": ["Br.","Sr.","Cde.","Ctz."]}',
     'deployment:migration-0007', 'ConfigConstant', '1.0.0', 'deployment');
