-- Slice 5 hardening — findings of the adversarial review (SLICE_05 §6).

-- PROPOSAL_RESOLVED joins the v1 taxonomy: the sovereign's answer to a
-- Joint Proposal is its own event, not the Petition Protocol's.
ALTER TABLE log_snapshots DROP CONSTRAINT log_snapshots_event_check;
ALTER TABLE log_snapshots ADD CONSTRAINT log_snapshots_event_check CHECK (event IN
    ('INTAKE_RAW_COPIED','NORMALIZED','EMBEDDED','LINK_DRAWN','LINK_SEVERED',
     'WEIGHT_RECALC','POSTULANT_EMERGED','AUDIT_OPENED','REPORT_FILED',
     'PROPOSAL_FILED','COMMITTED','DECOMMISSIONED','OVERRIDE_LAID',
     'PETITION_OPENED','PETITION_RESOLVED','ADMITTED','REJECTED','REFUSAL',
     'VIOLATION','JOB_TRANSITION','CLASSIFIED','AMENDED','PROPOSAL_RESOLVED'));

-- Office-authored flags are unique per stage: UNIQUE (job_id, stage) does
-- not bind NULL job_ids, so racing barrier certifications could mint
-- duplicates. One certification per barrier.
CREATE UNIQUE INDEX one_office_flag_per_stage ON readiness_flags (stage)
    WHERE job_id IS NULL;

-- The sovereign speaks once, at the substrate: a proposal's consent_ref,
-- once set, is never rewritten.
CREATE FUNCTION godhead_consent_set_once() RETURNS trigger AS $$
BEGIN
    IF OLD.consent_ref IS NOT NULL
       AND NEW.consent_ref IS DISTINCT FROM OLD.consent_ref THEN
        RAISE EXCEPTION 'the sovereign speaks once: proposal % is already answered', OLD.proposal_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER consent_set_once BEFORE UPDATE ON joint_proposals
    FOR EACH ROW EXECUTE FUNCTION godhead_consent_set_once();

-- A professed Cardinal is not re-tried in place: its commitment provenance
-- and membership are immutable; the only forward arc is decommission
-- (Law VI.5). Replaces the 0005 trigger function.
CREATE OR REPLACE FUNCTION godhead_require_commitment_chain() RETURNS trigger AS $$
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
    IF OLD.status = 'CARDINAL' THEN
        IF NEW.status = 'POSTULANT' THEN
            RAISE EXCEPTION 'a Cardinal does not lapse to Postulant (A.9)';
        END IF;
        IF NEW.committed_proposal_ref IS DISTINCT FROM OLD.committed_proposal_ref
           OR NEW.committed_consent_ref IS DISTINCT FROM OLD.committed_consent_ref
           OR NEW.committed_at IS DISTINCT FROM OLD.committed_at
           OR NEW.node_refs IS DISTINCT FROM OLD.node_refs
           OR NEW.link_refs IS DISTINCT FROM OLD.link_refs
           OR NEW.audit_depth IS DISTINCT FROM OLD.audit_depth THEN
            RAISE EXCEPTION 'a professed Cardinal is not re-tried or rewritten in place; decommission is the only door (Law VI.5)';
        END IF;
    END IF;
    IF OLD.status = 'DISSOLVED' AND NEW.status <> 'DISSOLVED' THEN
        RAISE EXCEPTION 'a dissolved matrix does not rise (A.9)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
