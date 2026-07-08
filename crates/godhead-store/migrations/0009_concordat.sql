-- Slice 8 — the Teacher's lint & the Concordat (docs/dev/SLICE_08.md).

ALTER TABLE log_snapshots DROP CONSTRAINT log_snapshots_event_check;
ALTER TABLE log_snapshots ADD CONSTRAINT log_snapshots_event_check CHECK (event IN
    ('INTAKE_RAW_COPIED','NORMALIZED','EMBEDDED','LINK_DRAWN','LINK_SEVERED',
     'WEIGHT_RECALC','POSTULANT_EMERGED','AUDIT_OPENED','REPORT_FILED',
     'PROPOSAL_FILED','COMMITTED','DECOMMISSIONED','OVERRIDE_LAID',
     'PETITION_OPENED','PETITION_RESOLVED','ADMITTED','REJECTED','REFUSAL',
     'VIOLATION','JOB_TRANSITION','CLASSIFIED','AMENDED','PROPOSAL_RESOLVED',
     'ENV_ESTABLISHED','ENV_ORPHANED','PAIRING_FORMED',
     'INSTRUCTION_FLAGGED','CONCORDAT_ADOPTED','BIAS_WARNING'));

-- B.4 — the Concordat: every version ever cited is retained forever
-- (§3.3). Adoption is a sovereign act (A.14 test (b)).
CREATE TABLE concordat_artifacts (
    version           text PRIMARY KEY,
    capability_tables jsonb NOT NULL,
    pairing_semantics jsonb NOT NULL DEFAULT '{}',
    adopted_at        timestamptz NOT NULL DEFAULT now(),
    adopted_by        text NOT NULL,
    schema_name       text NOT NULL,
    schema_version    text NOT NULL,
    produced_by       text NOT NULL,
    produced_at       timestamptz NOT NULL DEFAULT now()
);

-- B.1 — Instructions. Flagged instructions are immutable (§1.4);
-- correction flows through supersedes_ref, never an edit.
CREATE TABLE instructions (
    instruction_id     uuid PRIMARY KEY,
    teacher_env_ref    uuid REFERENCES environments(env_id),
    teacher_tier       text NOT NULL CHECK (teacher_tier IN ('REGULAR','DEVOUT','CANON')),
    target_tier        text NOT NULL CHECK (target_tier IN ('REGULAR','DEVOUT','CANON')),
    concordat_version  text NOT NULL,
    objective          text NOT NULL,
    steps              jsonb NOT NULL,
    acceptance_criteria jsonb NOT NULL,
    sources_drawn      jsonb NOT NULL DEFAULT '[]',
    skew               boolean NOT NULL DEFAULT false,
    supersedes_ref     uuid REFERENCES instructions(instruction_id),
    flagged            boolean NOT NULL DEFAULT false,
    revision           int NOT NULL DEFAULT 1,
    schema_name        text NOT NULL,
    schema_version     text NOT NULL,
    produced_by        text NOT NULL,
    produced_at        timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX instructions_by_supersedes ON instructions (supersedes_ref);

-- Bias disclosure rows: every Regular Teacher output, its draws and skew
-- (§6.3). The trailing window is read from here.
CREATE TABLE regular_outputs (
    output_id       uuid PRIMARY KEY,
    instruction_ref uuid NOT NULL REFERENCES instructions(instruction_id),
    sources_drawn   jsonb NOT NULL,
    skew            boolean NOT NULL,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

-- One standing warning per pattern scope (§6.3).
CREATE TABLE bias_warnings (
    scope           text PRIMARY KEY,
    status          text NOT NULL DEFAULT 'STANDING' CHECK (status IN ('STANDING','ACKNOWLEDGED','SILENCED')),
    raised_at       timestamptz NOT NULL DEFAULT now(),
    resolved_at     timestamptz,
    revision        int NOT NULL DEFAULT 1,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

CREATE TRIGGER no_delete_concordats BEFORE DELETE ON concordat_artifacts
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_instructions BEFORE DELETE ON instructions
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- §1.4: a flagged Instruction's body is frozen; only the flagged bit and
-- revision may advance (which they do exactly once, at flagging).
CREATE FUNCTION godhead_instruction_immutable() RETURNS trigger AS $$
BEGIN
    IF OLD.flagged THEN
        IF NEW.teacher_env_ref IS DISTINCT FROM OLD.teacher_env_ref
           OR NEW.teacher_tier IS DISTINCT FROM OLD.teacher_tier
           OR NEW.target_tier IS DISTINCT FROM OLD.target_tier
           OR NEW.concordat_version IS DISTINCT FROM OLD.concordat_version
           OR NEW.objective IS DISTINCT FROM OLD.objective
           OR NEW.steps IS DISTINCT FROM OLD.steps
           OR NEW.acceptance_criteria IS DISTINCT FROM OLD.acceptance_criteria
           OR NEW.sources_drawn IS DISTINCT FROM OLD.sources_drawn
           OR NEW.skew IS DISTINCT FROM OLD.skew
           OR NEW.supersedes_ref IS DISTINCT FROM OLD.supersedes_ref
           OR NEW.flagged IS DISTINCT FROM OLD.flagged THEN
            RAISE EXCEPTION 'a flagged Instruction is immutable; correct by supersession, never an edit (Standard §1.4)';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER instruction_immutable BEFORE UPDATE ON instructions
    FOR EACH ROW EXECUTE FUNCTION godhead_instruction_immutable();

-- Day-one Concordat v1.0.0 — ships with the B.3 capability tables.
INSERT INTO concordat_artifacts (version, capability_tables, pairing_semantics, adopted_by, schema_name, schema_version, produced_by) VALUES
    ('1.0.0',
     '{"REGULAR": [], "DEVOUT": ["FETCH_PER_WRIT","REFINE","ORGANIZE","CONSOLIDATE","LINK_PROPOSE","VERIFY"], "CANON": ["FETCH_PER_CANON","COMPILE_CORPUS","VERIFY"]}',
     '{"DEVOUT_ASSIGNMENT": "Professor + Devout Student", "CANONICAL_INSTRUCTION": "Doctor + Canon Student"}',
     'sovereign', 'ConcordatArtifact', '1.0.0', 'sovereign');

-- Instruction budget ceiling per target tier (A.14 operational, lint e).
INSERT INTO config_constants (key, tier, value, changed_by, schema_name, schema_version, produced_by) VALUES
    ('instruction_budget_ceiling', 'OPERATIONAL',
     '{"REGULAR": 100000, "DEVOUT": 200000, "CANON": 200000}',
     'deployment:migration-0009', 'ConfigConstant', '1.0.0', 'deployment');
