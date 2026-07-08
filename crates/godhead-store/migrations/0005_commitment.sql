-- Slice 5 — the commitment chain (docs/dev/SLICE_05.md): Law VI mechanical.

-- Office-authored flags (the supervisor's composite barrier certification,
-- doc 3 §3.3) carry no job identity; produced_by names the office.
ALTER TABLE readiness_flags ALTER COLUMN job_id DROP NOT NULL;

-- AMENDED joins the v1 event taxonomy (recursive confirmation, Law VI.4).
ALTER TABLE log_snapshots DROP CONSTRAINT log_snapshots_event_check;
ALTER TABLE log_snapshots ADD CONSTRAINT log_snapshots_event_check CHECK (event IN
    ('INTAKE_RAW_COPIED','NORMALIZED','EMBEDDED','LINK_DRAWN','LINK_SEVERED',
     'WEIGHT_RECALC','POSTULANT_EMERGED','AUDIT_OPENED','REPORT_FILED',
     'PROPOSAL_FILED','COMMITTED','DECOMMISSIONED','OVERRIDE_LAID',
     'PETITION_OPENED','PETITION_RESOLVED','ADMITTED','REJECTED','REFUSAL',
     'VIOLATION','JOB_TRANSITION','CLASSIFIED','AMENDED'));

CREATE TABLE matrices (
    matrix_id              uuid PRIMARY KEY,
    status                 text NOT NULL DEFAULT 'POSTULANT' CHECK (status IN
                               ('POSTULANT','CARDINAL','DISSOLVED')),
    category               text NOT NULL,
    revision               int NOT NULL DEFAULT 1,
    audit_depth            int NOT NULL DEFAULT 0,
    node_refs              jsonb NOT NULL,
    link_refs              jsonb NOT NULL,
    emerged_by             uuid NOT NULL REFERENCES job_records(job_id),
    -- Law VI.1: a density evaluation lacking its citation fails validation.
    config_rev             int NOT NULL,
    committed_proposal_ref uuid,
    committed_consent_ref  uuid,
    committed_at           timestamptz,
    schema_name            text NOT NULL,
    schema_version         text NOT NULL,
    produced_by            text NOT NULL,
    produced_at            timestamptz NOT NULL DEFAULT now()
);

-- One live matrix per category: a standing POSTULANT or CARDINAL does not
-- re-emerge (SLICE_05 §3).
CREATE UNIQUE INDEX one_live_matrix_per_category ON matrices (category)
    WHERE status IN ('POSTULANT','CARDINAL');

CREATE TABLE audit_reports (
    report_id       uuid PRIMARY KEY,
    job_id          uuid NOT NULL REFERENCES job_records(job_id),
    matrix_ref      uuid NOT NULL REFERENCES matrices(matrix_id),
    matrix_revision int NOT NULL,
    auditor         text NOT NULL CHECK (auditor IN ('GABRIEL','LUCY')),
    kind            text NOT NULL CHECK (kind IN ('AFFIRMATION','INDICTMENT')),
    claims          jsonb NOT NULL,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now(),
    -- One report per auditor per matrix revision: the audit protocol
    -- spawns each side once per cycle.
    UNIQUE (matrix_ref, matrix_revision, auditor)
);

CREATE TABLE joint_proposals (
    proposal_id     uuid PRIMARY KEY,
    job_id          uuid NOT NULL REFERENCES job_records(job_id),
    matrix_ref      uuid NOT NULL REFERENCES matrices(matrix_id),
    matrix_revision int NOT NULL,
    report_refs     jsonb NOT NULL,
    verdict         text NOT NULL CHECK (verdict IN ('COMMIT','AMEND','REJECT')),
    changes         jsonb NOT NULL DEFAULT '[]',
    reasons         jsonb NOT NULL DEFAULT '[]',
    consent_ref     uuid,
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now(),
    -- A.11: changes required iff AMEND; reasons required iff REJECT.
    CHECK (verdict <> 'AMEND' OR jsonb_array_length(changes) > 0),
    CHECK (verdict = 'AMEND' OR jsonb_array_length(changes) = 0),
    CHECK (verdict <> 'REJECT' OR jsonb_array_length(reasons) > 0),
    UNIQUE (matrix_ref, matrix_revision)
);

-- The trial's records are eternal.
CREATE TRIGGER no_delete_matrices BEFORE DELETE ON matrices
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_reports BEFORE DELETE ON audit_reports
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_proposals BEFORE DELETE ON joint_proposals
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- Law VI.3 — fiat is impossible mechanically, not morally: the store
-- rejects any matrix-status mutation whose proposal -> consent -> act
-- chain does not resolve and cross-reference. The guarantee lives HERE,
-- below every writer of any kind.
CREATE FUNCTION godhead_require_commitment_chain() RETURNS trigger AS $$
BEGIN
    IF NEW.status = 'CARDINAL' AND (OLD.status IS DISTINCT FROM 'CARDINAL') THEN
        IF NEW.committed_proposal_ref IS NULL OR NEW.committed_consent_ref IS NULL THEN
            RAISE EXCEPTION 'fiat is impossible: CARDINAL requires a resolving proposal and consent (Law VI.3)';
        END IF;
        IF NOT EXISTS (
            SELECT 1 FROM joint_proposals p
            JOIN consent_records c ON c.consent_id = NEW.committed_consent_ref
            WHERE p.proposal_id = NEW.committed_proposal_ref
              AND p.matrix_ref = NEW.matrix_id
              AND p.verdict = 'COMMIT'
              AND c.subject_ref = p.proposal_id
              AND c.decision = 'GRANTED'
        ) THEN
            RAISE EXCEPTION 'fiat is impossible: the proposal -> consent chain does not resolve and cross-reference (Law VI.3)';
        END IF;
    END IF;
    -- A dissolved matrix does not rise; a Cardinal does not lapse back to
    -- Postulant. Forward arcs only.
    IF OLD.status = 'DISSOLVED' AND NEW.status <> 'DISSOLVED' THEN
        RAISE EXCEPTION 'a dissolved matrix does not rise (A.9)';
    END IF;
    IF OLD.status = 'CARDINAL' AND NEW.status = 'POSTULANT' THEN
        RAISE EXCEPTION 'a Cardinal does not lapse to Postulant (A.9)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER commitment_chain BEFORE UPDATE ON matrices
    FOR EACH ROW EXECUTE FUNCTION godhead_require_commitment_chain();

-- Birth is always Postulant: emergence is bookkeeping, never commitment.
CREATE FUNCTION godhead_matrices_born_postulant() RETURNS trigger AS $$
BEGIN
    IF NEW.status <> 'POSTULANT' THEN
        RAISE EXCEPTION 'a matrix is born POSTULANT; commitment is a tried act (Law VI.2-VI.3)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER born_postulant BEFORE INSERT ON matrices
    FOR EACH ROW EXECUTE FUNCTION godhead_matrices_born_postulant();
