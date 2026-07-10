-- Slice 11 — the V.4 brief-rooted seam, closed both ways (docs/dev/SLICE_11.md
-- §4; the opening round's recorded seam). Closure (a) — every external fetch is
-- mandate-rooted by construction — lives in the fetch labor (SC-J03 binds a
-- resolving mandate). Closure (b) is here: godhead_quarantine_only bars from the
-- internal namespace not only a mandate-rooted fetch job, but ANY job that has
-- deposited external material into quarantine, whatever its charter — so a
-- brief-rooted depositor cannot launder its own external bytes into
-- nodes/artifacts/environment_items either.
--
-- The admission node mint is unaffected: it runs under a SEPARATE intake stage
-- job that has deposited nothing, so `EXISTS(quarantine_items WHERE
-- origin_job_ref = writer)` is false for it (the depositor was the fetch job).
-- Timing caveat, disclosed (§4): a job that writes an internal row BEFORE it
-- deposits is not caught by (b) alone — that case is covered by (a), since a
-- lawful fetcher is mandate-rooted and deposits before any admission.
CREATE OR REPLACE FUNCTION godhead_quarantine_only() RETURNS trigger AS $$
DECLARE writer uuid;
BEGIN
    BEGIN
        writer := NEW.produced_by::uuid;
    EXCEPTION WHEN invalid_text_representation THEN
        RETURN NEW; -- not a job-authored row; other walls govern it
    END;
    IF EXISTS (SELECT 1 FROM job_records j JOIN mandates m ON m.mandate_id = j.brief_ref
               WHERE j.job_id = writer)
       OR EXISTS (SELECT 1 FROM quarantine_items q WHERE q.origin_job_ref = writer) THEN
        RAISE EXCEPTION 'external-origin content lands only in the quarantine namespace (Law V.4); % is not it', TG_TABLE_NAME;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
