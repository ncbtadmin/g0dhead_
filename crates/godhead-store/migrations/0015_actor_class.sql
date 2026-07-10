-- Slice 10 — session-scoped actor-class authentication (ruling G10; SC-I07a)
-- and the content-hash certification columns (ruling G7; S3).
--
-- The store's own API paths set a transaction-local credential
-- (`SET LOCAL godhead.actor_class`) inside the transaction that writes a
-- reserved table; these triggers verify the session's class matches what
-- the table demands. Below the API the variable is absent, so 'deacon',
-- 'sovereign', and 'forged' become the same object in the only sense that
-- matters: all rejected. The credential lives only in the code path that is
-- the lawful surface — no second truth.
--
-- NOTE for future migrations: seeding a class-guarded table from SQL
-- requires `SET LOCAL godhead.actor_class = 'sovereign';` first (each
-- migration runs in its own transaction).

SET LOCAL godhead.actor_class = 'sovereign';

-- The admission-legibility constants (SC-I07b; Book II §1 doctrine, ruling
-- G11) — operational tier: thresholds of NOTICE, never of blocking.
INSERT INTO config_constants (key, tier, value, changed_by, schema_name, schema_version, produced_by) VALUES
    ('admission_batch_threshold', 'OPERATIONAL', '50',      'deployment:migration-0015', 'ConfigConstant', '1.0.0', 'deployment'),
    ('admission_rate_window_ms',  'OPERATIONAL', '3600000', 'deployment:migration-0015', 'ConfigConstant', '1.0.0', 'deployment'),
    ('admission_rate_threshold',  'OPERATIONAL', '5',       'deployment:migration-0015', 'ConfigConstant', '1.0.0', 'deployment');

CREATE FUNCTION godhead_require_actor_class() RETURNS trigger AS $$
DECLARE cls text;
BEGIN
    cls := current_setting('godhead.actor_class', true);
    IF cls IS NULL OR cls = '' THEN
        RAISE EXCEPTION 'GATE_BYPASS_ATTEMPT: % is %-reserved and this path did not authenticate as any class; below the API, ''sovereign'', ''deacon'', and ''forged'' are the same string: rejected (ruling G10)',
            TG_TABLE_NAME, TG_ARGV[0];
    END IF;
    IF cls <> TG_ARGV[0] THEN
        RAISE EXCEPTION 'GATE_BYPASS_ATTEMPT: % demands actor class ''%''; the session authenticated as ''%'' (ruling G10)',
            TG_TABLE_NAME, TG_ARGV[0], cls;
    END IF;
    -- The row stamps what the class claims: an office writes as itself.
    IF TG_ARGV[0] LIKE 'office:%' AND NEW.produced_by <> TG_ARGV[0] THEN
        RAISE EXCEPTION 'GATE_BYPASS_ATTEMPT: % authenticated as ''%'' but stamps produced_by ''%'' (ruling G10)',
            TG_TABLE_NAME, TG_ARGV[0], NEW.produced_by;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Sovereign-reserved: the IV.4 tables the agent-author trigger already
-- shapes, now class-authenticated on top — and the mandates table with them.
CREATE TRIGGER class_overrides BEFORE INSERT OR UPDATE ON override_records
    FOR EACH ROW EXECUTE FUNCTION godhead_require_actor_class('sovereign');
CREATE TRIGGER class_consents BEFORE INSERT OR UPDATE ON consent_records
    FOR EACH ROW EXECUTE FUNCTION godhead_require_actor_class('sovereign');
CREATE TRIGGER class_config BEFORE INSERT OR UPDATE ON config_constants
    FOR EACH ROW EXECUTE FUNCTION godhead_require_actor_class('sovereign');
CREATE TRIGGER class_mandates BEFORE INSERT OR UPDATE ON mandates
    FOR EACH ROW EXECUTE FUNCTION godhead_require_actor_class('sovereign');

-- Office-reserved: the Deacon's threshold records.
CREATE TRIGGER class_verdicts BEFORE INSERT ON scan_verdicts
    FOR EACH ROW EXECUTE FUNCTION godhead_require_actor_class('office:deacon');
CREATE TRIGGER class_manifests BEFORE INSERT ON manifests
    FOR EACH ROW EXECUTE FUNCTION godhead_require_actor_class('office:deacon');

-- Ruling G7 — byte-integrity certification: the SHA-256 of the canonical
-- body, persisted at FLAG, re-proven at every read of the flagged record.
ALTER TABLE instructions ADD COLUMN content_sha text
    CHECK (content_sha IS NULL OR length(content_sha) = 64);
ALTER TABLE returns ADD COLUMN content_sha text
    CHECK (content_sha IS NULL OR length(content_sha) = 64);
