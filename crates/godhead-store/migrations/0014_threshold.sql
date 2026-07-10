-- Slice 10 — the Deacon's threshold (Dogma Book II §1; docs/dev/SLICE_10.md).
-- The quarantine namespace (Law V.4), ScanVerdicts, the Manifest, and the
-- walls: external-origin content lands ONLY here; INFECTED|SUSPECT|ERROR is
-- never admissible; one Manifest serves one mandate-trip; nothing in
-- quarantine is purged (SC-I05 — purging is the deferred Duty of the House).

-- A.12 — what lands at the threshold. The external bytes at rest, their
-- origin (job + the human mandate or brief behind it), and their standing.
CREATE TABLE quarantine_items (
    item_ref          uuid PRIMARY KEY,
    origin_job_ref    uuid NOT NULL REFERENCES job_records(job_id),
    mandate_ref       uuid REFERENCES mandates(mandate_id),
    brief_ref         uuid,
    filename          text NOT NULL,
    declared_type     text NOT NULL,
    content           bytea NOT NULL,
    scan_ref          uuid,
    consent_ref       uuid REFERENCES consent_records(consent_id),
    admitted_node_ref uuid REFERENCES nodes(node_id),
    held_since        timestamptz NOT NULL DEFAULT now(),
    revision          int NOT NULL DEFAULT 1,
    schema_name       text NOT NULL,
    schema_version    text NOT NULL,
    produced_by       text NOT NULL,
    produced_at       timestamptz NOT NULL DEFAULT now(),
    -- Every arrival began in a human hand (§1.4): a mandate or a brief.
    CHECK (mandate_ref IS NOT NULL OR brief_ref IS NOT NULL)
);

-- A.12 — one scan's verdict over one quarantined item. Office-reserved
-- (the actor-class wall arrives in 0015); append-only.
CREATE TABLE scan_verdicts (
    scan_id        uuid PRIMARY KEY,
    item_ref       uuid NOT NULL REFERENCES quarantine_items(item_ref),
    verdict        text NOT NULL CHECK (verdict IN ('CLEAN','INFECTED','SUSPECT','ERROR')),
    engine_alias   text NOT NULL,
    engine_version text NOT NULL,
    signature_rev  text,
    scanned_at     timestamptz NOT NULL DEFAULT now(),
    schema_name    text NOT NULL,
    schema_version text NOT NULL,
    produced_by    text NOT NULL,
    produced_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX verdicts_by_item ON scan_verdicts (item_ref, scanned_at DESC);

ALTER TABLE quarantine_items
    ADD CONSTRAINT quarantine_scan_fk FOREIGN KEY (scan_ref) REFERENCES scan_verdicts(scan_id);

-- Book II §1 step 4 — the Manifest. UNIQUE(trip_job_ref) IS the mechanical
-- rule of the deliberate gate: one Manifest per mandate-trip, never pooled
-- across trips (ruling G11).
CREATE TABLE manifests (
    manifest_id     uuid PRIMARY KEY,
    mandate_ref     uuid NOT NULL REFERENCES mandates(mandate_id),
    trip_job_ref    uuid NOT NULL UNIQUE REFERENCES job_records(job_id),
    items           jsonb NOT NULL,
    standing_notice text,
    presented_at    timestamptz NOT NULL DEFAULT now(),
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

-- An admission consent names the scan it saw: a newer, darker verdict
-- defeats a stale consent at the admission wall.
ALTER TABLE consent_records ADD COLUMN scan_ref uuid;

-- Preservation (SC-I05): nothing at the threshold is deleted — not items,
-- not verdicts, not manifests. Purging belongs to the deferred Duty of the
-- House, and that duty is not built.
CREATE TRIGGER no_delete_quarantine BEFORE DELETE ON quarantine_items
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_verdicts BEFORE DELETE ON scan_verdicts
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_manifests BEFORE DELETE ON manifests
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- A verdict, once rendered, stands (a re-scan is a NEW verdict); a
-- presented Manifest is testimony, frozen whole.
CREATE FUNCTION godhead_threshold_immutable() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'a % row is immutable at the threshold; a change is a new record (Book II §1)', TG_TABLE_NAME;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER verdicts_immutable BEFORE UPDATE ON scan_verdicts
    FOR EACH ROW EXECUTE FUNCTION godhead_threshold_immutable();
CREATE TRIGGER manifests_immutable BEFORE UPDATE ON manifests
    FOR EACH ROW EXECUTE FUNCTION godhead_threshold_immutable();

-- A quarantined item's SUBSTANCE is frozen: what arrived, from where, under
-- what human act, and when. Only its protocol standing (scan_ref,
-- consent_ref, admitted_node_ref, revision) may advance — and admission
-- converges: admitted_node_ref is set exactly once.
CREATE FUNCTION godhead_quarantine_core_immutable() RETURNS trigger AS $$
BEGIN
    IF NEW.content IS DISTINCT FROM OLD.content
       OR NEW.filename IS DISTINCT FROM OLD.filename
       OR NEW.declared_type IS DISTINCT FROM OLD.declared_type
       OR NEW.origin_job_ref IS DISTINCT FROM OLD.origin_job_ref
       OR NEW.mandate_ref IS DISTINCT FROM OLD.mandate_ref
       OR NEW.brief_ref IS DISTINCT FROM OLD.brief_ref
       OR NEW.held_since IS DISTINCT FROM OLD.held_since
       OR NEW.produced_by IS DISTINCT FROM OLD.produced_by
       OR NEW.produced_at IS DISTINCT FROM OLD.produced_at THEN
        RAISE EXCEPTION 'a quarantined item''s substance is immutable; only its protocol standing advances (Book II §1)';
    END IF;
    IF OLD.admitted_node_ref IS NOT NULL
       AND NEW.admitted_node_ref IS DISTINCT FROM OLD.admitted_node_ref THEN
        RAISE EXCEPTION 'admission is recorded exactly once; a retry converges, it never re-admits (Law I.3)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER quarantine_core_immutable BEFORE UPDATE ON quarantine_items
    FOR EACH ROW EXECUTE FUNCTION godhead_quarantine_core_immutable();

-- Law V.4 at the substrate (SC-I01): a mandate-trip job's content lands in
-- the quarantine namespace and nowhere else. A job whose brief IS a mandate
-- (kind CANON → FETCH_PER_CANON, WRIT → FETCH_PER_WRIT) is a fetching job;
-- the store rejects its writes to the internal-content surfaces below even
-- the API. produced_by/job columns name the writer per table.
CREATE FUNCTION godhead_quarantine_only() RETURNS trigger AS $$
DECLARE writer uuid;
BEGIN
    BEGIN
        writer := NEW.produced_by::uuid;
    EXCEPTION WHEN invalid_text_representation THEN
        RETURN NEW; -- not a job-authored row; other walls govern it
    END;
    IF EXISTS (SELECT 1 FROM job_records j JOIN mandates m ON m.mandate_id = j.brief_ref
               WHERE j.job_id = writer) THEN
        RAISE EXCEPTION 'external-origin content lands only in the quarantine namespace (Law V.4); % is not it', TG_TABLE_NAME;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER quarantine_only_nodes BEFORE INSERT ON nodes
    FOR EACH ROW EXECUTE FUNCTION godhead_quarantine_only();
CREATE TRIGGER quarantine_only_artifacts BEFORE INSERT ON artifacts
    FOR EACH ROW EXECUTE FUNCTION godhead_quarantine_only();
CREATE TRIGGER quarantine_only_env_items BEFORE INSERT ON environment_items
    FOR EACH ROW EXECUTE FUNCTION godhead_quarantine_only();
