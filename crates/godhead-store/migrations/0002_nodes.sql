-- Slice 2 — intake & endurance (docs/dev/SLICE_02.md).

-- INTAKE joins the job-type roster: the deterministic floor labors under
-- JobRecords for idempotency and recovery (doc 3 §2.7).
ALTER TABLE job_records DROP CONSTRAINT job_records_agent_type_check;
ALTER TABLE job_records ADD CONSTRAINT job_records_agent_type_check CHECK (agent_type IN
    ('SLAVE','AGGREGATOR','NOTARY','AUDITOR','RECONCILER','STUDENT','TEACHER','INTAKE'));

-- CLASSIFIED joins the v1 event taxonomy (floor classification, doc 2 §2.5).
ALTER TABLE log_snapshots DROP CONSTRAINT log_snapshots_event_check;
ALTER TABLE log_snapshots ADD CONSTRAINT log_snapshots_event_check CHECK (event IN
    ('INTAKE_RAW_COPIED','NORMALIZED','EMBEDDED','LINK_DRAWN','LINK_SEVERED',
     'WEIGHT_RECALC','POSTULANT_EMERGED','AUDIT_OPENED','REPORT_FILED',
     'PROPOSAL_FILED','COMMITTED','DECOMMISSIONED','OVERRIDE_LAID',
     'PETITION_OPENED','PETITION_RESOLVED','ADMITTED','REJECTED','REFUSAL',
     'VIOLATION','JOB_TRANSITION','CLASSIFIED'));

-- The atoms (doc 3 §2.1): raw content on disk by reference, derivative by
-- reference, metadata. Content is immutable once committed.
CREATE TABLE nodes (
    node_id           uuid PRIMARY KEY,
    filename          text NOT NULL,
    filetype          text NOT NULL,
    size_bytes        bigint NOT NULL CHECK (size_bytes >= 0),
    raw_path          text NOT NULL,
    raw_sha256        text NOT NULL CHECK (length(raw_sha256) = 64),
    derivative_path   text,
    derivative_sha256 text CHECK (derivative_sha256 IS NULL OR length(derivative_sha256) = 64),
    normalized        boolean NOT NULL DEFAULT false,
    intake_status     text NOT NULL DEFAULT 'RAW' CHECK (intake_status IN
                          ('RAW','NORMALIZED','DECODE_FAILED','UNSUPPORTED')),
    classification    jsonb NOT NULL DEFAULT '[]',
    notice            text,
    revision          int NOT NULL DEFAULT 1,
    schema_name       text NOT NULL,
    schema_version    text NOT NULL,
    produced_by       text NOT NULL,
    produced_at       timestamptz NOT NULL DEFAULT now()
);

-- Raw-copied-once is inviolable (doc 3 §4.2): the raw reference fields may
-- never be rewritten, by any writer, through any path.
CREATE FUNCTION godhead_forbid_raw_mutation() RETURNS trigger AS $$
BEGIN
    IF NEW.raw_path IS DISTINCT FROM OLD.raw_path
       OR NEW.raw_sha256 IS DISTINCT FROM OLD.raw_sha256
       OR NEW.size_bytes IS DISTINCT FROM OLD.size_bytes
       OR NEW.filename IS DISTINCT FROM OLD.filename
       OR NEW.filetype IS DISTINCT FROM OLD.filetype THEN
        RAISE EXCEPTION 'the atom is immutable: raw reference fields are never rewritten (doc 3 §4.2)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER no_raw_mutation BEFORE UPDATE ON nodes
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_raw_mutation();
