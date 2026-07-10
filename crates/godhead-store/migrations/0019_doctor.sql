-- Slice 11b — the Doctor & the dissolution cascade (docs/dev/SLICE_11B.md).
-- Closes SLICE_07's deferred ORPHANED cascade on the record (§0.6).

-- §0.4 — the Doctor's deployment reference: doctor_env_ref -> student_env_ref.
-- The reference the orphan cascade walks; append-only, frozen (a new deployment
-- is a new act, never a mutation).
CREATE TABLE doctor_deployments (
    deployment_id   uuid PRIMARY KEY,
    doctor_env_ref  uuid NOT NULL REFERENCES environments(env_id),
    student_env_ref uuid NOT NULL REFERENCES environments(env_id),
    -- The second instrument: the CANONICAL_INSTRUCTION pairing (IX.5), the
    -- bridge to the corpus manifest. Both instruments recorded in one row.
    pairing_id      uuid NOT NULL REFERENCES pairings(pairing_id),
    deployed_at     timestamptz NOT NULL DEFAULT now(),
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX doctor_deployments_by_student ON doctor_deployments (student_env_ref);

CREATE FUNCTION godhead_deployment_immutable() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'a Doctor deployment is immutable; a new deployment is a new act (SLICE_11B §0.4)';
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER deployments_immutable BEFORE UPDATE ON doctor_deployments
    FOR EACH ROW EXECUTE FUNCTION godhead_deployment_immutable();
CREATE TRIGGER no_delete_deployments BEFORE DELETE ON doctor_deployments
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- §0.2 — retire_environment's human witness. DISSOLVED is a deliberate sovereign
-- act; retired_by names the human hand that struck the room.
ALTER TABLE environments ADD COLUMN retired_by text;

-- §0.5 / §0.2 — the environment status arc only ever descends, and the sole
-- lever to DISSOLVED is a HUMAN retirement:
--   · no silent revival: ORPHANED never returns to LIVE;
--   · DISSOLVED is struck: it never returns;
--   · a transition INTO DISSOLVED names a human retired_by — a job-uuid actor is
--     a gate bypass (IV.4), exactly as the agent-author wall rejects one on the
--     other human-reserved tables.
CREATE FUNCTION godhead_env_status_arc() RETURNS trigger AS $$
BEGIN
    IF OLD.status = 'ORPHANED' AND NEW.status = 'LIVE' THEN
        RAISE EXCEPTION 'no silent revival: ORPHANED is terminal for work; a fresh deployment, never a status flip (SC-J08; Standard §4.3)';
    END IF;
    IF OLD.status = 'DISSOLVED' AND NEW.status <> 'DISSOLVED' THEN
        RAISE EXCEPTION 'a DISSOLVED environment is struck; it does not return (A.8)';
    END IF;
    IF NEW.status = 'DISSOLVED' AND OLD.status <> 'DISSOLVED' THEN
        IF NEW.retired_by IS NULL
           OR NEW.retired_by ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
            RAISE EXCEPTION 'GATE_BYPASS_ATTEMPT: retiring an environment is human-reserved; retired_by is a human actor, never a job identity (Law IV.4)';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER env_status_arc BEFORE UPDATE ON environments
    FOR EACH ROW EXECUTE FUNCTION godhead_env_status_arc();

-- §0.3 — the Doctor-orphan cascade, at the substrate. When a Canon Student
-- environment LEAVES LIVE — for ORPHANED (its matrix decommissioned) or
-- DISSOLVED (retired) — every Doctor whose deployment names it goes ORPHANED.
-- Keying on the departure from LIVE catches BOTH levers with one wall, so no
-- dissolution path can forget the dependent. (A Doctor is a Teacher, never a
-- student_env_ref, so its own orphaning cascades no further — bounded.)
CREATE FUNCTION godhead_orphan_dependent_doctors() RETURNS trigger AS $$
BEGIN
    IF OLD.status = 'LIVE' AND NEW.status <> 'LIVE' THEN
        UPDATE environments SET status = 'ORPHANED', revision = revision + 1
        WHERE status = 'LIVE'
          AND env_id IN (SELECT doctor_env_ref FROM doctor_deployments
                         WHERE student_env_ref = NEW.env_id);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER orphan_dependent_doctors AFTER UPDATE ON environments
    FOR EACH ROW EXECUTE FUNCTION godhead_orphan_dependent_doctors();

-- A.5 — the two Slice-11b lifecycle events join the v1 taxonomy (the closed
-- event set is extended only by amendment; the standing pattern of 0002, 0005,
-- 0006, 0007, 0009, 0010, 0016). ENV_DISSOLVED at retire_environment,
-- DOCTOR_DEPLOYED at deploy_doctor.
ALTER TABLE log_snapshots DROP CONSTRAINT log_snapshots_event_check;
ALTER TABLE log_snapshots ADD CONSTRAINT log_snapshots_event_check CHECK (event IN
    ('INTAKE_RAW_COPIED','NORMALIZED','EMBEDDED','LINK_DRAWN','LINK_SEVERED',
     'WEIGHT_RECALC','POSTULANT_EMERGED','AUDIT_OPENED','REPORT_FILED',
     'PROPOSAL_FILED','COMMITTED','DECOMMISSIONED','OVERRIDE_LAID',
     'PETITION_OPENED','PETITION_RESOLVED','ADMITTED','REJECTED','REFUSAL',
     'VIOLATION','JOB_TRANSITION','CLASSIFIED','AMENDED','PROPOSAL_RESOLVED',
     'ENV_ESTABLISHED','ENV_ORPHANED','PAIRING_FORMED',
     'INSTRUCTION_FLAGGED','CONCORDAT_ADOPTED','BIAS_WARNING',
     'RETURN_FLAGGED','REFINED',
     'MANDATE_AUTHORED','CHAIN_APPENDED','QUARANTINED','SCAN_RECORDED',
     'MANIFEST_PRESENTED',
     'ENV_DISSOLVED','DOCTOR_DEPLOYED'));
