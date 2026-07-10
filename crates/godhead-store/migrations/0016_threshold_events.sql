-- Slice 10 — the five threshold & J-floor events join the v1 taxonomy
-- (A.5: the closed event set is extended only by amendment). 0013–0015
-- shipped the tables and their write paths but not this amendment, so
-- every slice-10 log write (MANDATE_AUTHORED at author_mandate,
-- CHAIN_APPENDED at append_chain_entry, QUARANTINED at quarantine_deposit,
-- SCAN_RECORDED at record_scan_verdict and the scan-pass failure surface,
-- MANIFEST_PRESENTED at assemble_manifest) died at the substrate. Found by
-- the Section I suite; amended here in the standing pattern (0002, 0005,
-- 0006, 0007, 0009, 0010) — a new migration, because an applied migration
-- is never rewritten (the migrator checksums are testimony too).
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
     'MANIFEST_PRESENTED'));
