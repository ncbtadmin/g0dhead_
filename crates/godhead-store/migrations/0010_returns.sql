-- Slice 9 — Student returns & stewardship (docs/dev/SLICE_09.md).

ALTER TABLE log_snapshots DROP CONSTRAINT log_snapshots_event_check;
ALTER TABLE log_snapshots ADD CONSTRAINT log_snapshots_event_check CHECK (event IN
    ('INTAKE_RAW_COPIED','NORMALIZED','EMBEDDED','LINK_DRAWN','LINK_SEVERED',
     'WEIGHT_RECALC','POSTULANT_EMERGED','AUDIT_OPENED','REPORT_FILED',
     'PROPOSAL_FILED','COMMITTED','DECOMMISSIONED','OVERRIDE_LAID',
     'PETITION_OPENED','PETITION_RESOLVED','ADMITTED','REJECTED','REFUSAL',
     'VIOLATION','JOB_TRANSITION','CLASSIFIED','AMENDED','PROPOSAL_RESOLVED',
     'ENV_ESTABLISHED','ENV_ORPHANED','PAIRING_FORMED',
     'INSTRUCTION_FLAGGED','CONCORDAT_ADOPTED','BIAS_WARNING',
     'RETURN_FLAGGED','REFINED'));

-- B.2 — the ReturnManifest: everything a Student is for ends in one.
-- Flagged returns are immutable; a Return that flags is certified.
CREATE TABLE returns (
    return_id         uuid PRIMARY KEY,
    instruction_ref   uuid NOT NULL REFERENCES instructions(instruction_id),
    student_env_ref   uuid NOT NULL REFERENCES environments(env_id),
    concordat_version text NOT NULL,
    items             jsonb NOT NULL,
    completion        jsonb NOT NULL,
    flagged           boolean NOT NULL DEFAULT false,
    revision          int NOT NULL DEFAULT 1,
    schema_name       text NOT NULL,
    schema_version    text NOT NULL,
    produced_by       text NOT NULL,
    produced_at       timestamptz NOT NULL DEFAULT now()
);

-- A refined artifact and its derivation (Handbook §1.2b).
CREATE TABLE refined_artifacts (
    artifact_id     uuid PRIMARY KEY,
    env_ref         uuid NOT NULL REFERENCES environments(env_id),
    source_refs     jsonb NOT NULL,
    method          text NOT NULL,
    content_sha     text NOT NULL CHECK (length(content_sha) = 64),
    revision        int NOT NULL DEFAULT 1,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

CREATE TRIGGER no_delete_returns BEFORE DELETE ON returns
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- A derivation record is never deleted; a wrong one is found by the
-- closure/re-derivability walk (§1.2), not erased.
CREATE TRIGGER no_delete_refined BEFORE DELETE ON refined_artifacts
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- A flagged Return is immutable (its refinement feeds the Professor only
-- through the certified Return); correction is a fresh Return.
CREATE FUNCTION godhead_return_immutable() RETURNS trigger AS $$
BEGIN
    IF OLD.flagged THEN
        IF NEW.instruction_ref IS DISTINCT FROM OLD.instruction_ref
           OR NEW.student_env_ref IS DISTINCT FROM OLD.student_env_ref
           OR NEW.concordat_version IS DISTINCT FROM OLD.concordat_version
           OR NEW.items IS DISTINCT FROM OLD.items
           OR NEW.completion IS DISTINCT FROM OLD.completion
           OR NEW.flagged IS DISTINCT FROM OLD.flagged THEN
            RAISE EXCEPTION 'a flagged Return is immutable; a correction is a fresh Return (Handbook §3.1)';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER return_immutable BEFORE UPDATE ON returns
    FOR EACH ROW EXECUTE FUNCTION godhead_return_immutable();
