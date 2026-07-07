-- Slice 3 — human sovereignty (docs/dev/SLICE_03.md): Law IV mechanical.

CREATE TABLE override_records (
    override_id     uuid PRIMARY KEY,
    subject_ref     uuid NOT NULL,
    kind            text NOT NULL CHECK (kind IN
                        ('LINK_SEVERED','LINK_FORCED','CATEGORY_REASSIGNED','WEIGHT_CORRECTED')),
    basis           text NOT NULL CHECK (basis IN ('SOVEREIGN_HAND','GRANTED_PETITION')),
    prior_ref       uuid REFERENCES override_records(override_id),
    consent_ref     uuid,
    protected_state jsonb NOT NULL,
    user_overridden boolean NOT NULL DEFAULT true CHECK (user_overridden),
    laid_at         timestamptz NOT NULL DEFAULT now(),
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now(),
    -- A.7: consent_ref required iff the basis is a granted petition.
    CHECK (basis <> 'GRANTED_PETITION' OR consent_ref IS NOT NULL),
    CHECK (basis <> 'SOVEREIGN_HAND' OR consent_ref IS NULL)
);
CREATE INDEX overrides_by_subject ON override_records (subject_ref, produced_at DESC);

-- One petition lineage per (subject, kind): recurrence escalates the
-- record, it never duplicates (IV.2).
CREATE TABLE petition_records (
    petition_id       uuid PRIMARY KEY,
    subject_ref       uuid NOT NULL,
    change_kind       text NOT NULL CHECK (change_kind IN
                          ('LINK_SEVERED','LINK_FORCED','CATEGORY_REASSIGNED','WEIGHT_CORRECTED')),
    reason            text NOT NULL,
    evidence_refs     jsonb NOT NULL DEFAULT '[]',
    proposed_change   jsonb NOT NULL,
    status            text NOT NULL DEFAULT 'OPEN' CHECK (status IN
                          ('OPEN','DECLINED','ESCALATED','GRANTED','SILENCED')),
    occurrence_count  int NOT NULL DEFAULT 1,
    consent_ref       uuid,
    execution_job_ref uuid REFERENCES job_records(job_id),
    resolved_at       timestamptz,
    schema_name       text NOT NULL,
    schema_version    text NOT NULL,
    produced_by       text NOT NULL,
    produced_at       timestamptz NOT NULL DEFAULT now(),
    UNIQUE (subject_ref, change_kind)
);

CREATE TABLE consent_records (
    consent_id      uuid PRIMARY KEY,
    subject_ref     uuid NOT NULL,
    decision        text NOT NULL CHECK (decision IN
                        ('ADMITTED','REJECTED','GRANTED','DECLINED','SILENCED')),
    scope           text NOT NULL DEFAULT 'ITEM' CHECK (scope IN ('ITEM','BATCH')),
    decided_by      text NOT NULL,
    decided_at      timestamptz NOT NULL DEFAULT now(),
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now()
);

-- IV.3: declines are signal; petitions and consents are never purged.
CREATE TRIGGER no_delete_petitions BEFORE DELETE ON petition_records
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_consents BEFORE DELETE ON consent_records
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();
CREATE TRIGGER no_delete_overrides BEFORE DELETE ON override_records
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- Law IV.4 at the substrate: overrides, consents, and config are
-- human-reserved. Every agent write path stamps its job UUID as
-- produced_by; sovereign acts stamp a human actor string. A UUID-shaped
-- author on a reserved table IS a gate bypass — rejected below even the
-- store's API. (The successor override laid under a grant is stamped with
-- the consent's decider: the authority is the consent, never the Notary.)
CREATE FUNCTION godhead_forbid_agent_author() RETURNS trigger AS $$
BEGIN
    IF NEW.produced_by ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
        RAISE EXCEPTION 'GATE_BYPASS_ATTEMPT: % is human-reserved; no agent-callable path exists (Law IV.4)',
            TG_TABLE_NAME;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER no_agent_overrides BEFORE INSERT OR UPDATE ON override_records
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_agent_author();
CREATE TRIGGER no_agent_consents BEFORE INSERT OR UPDATE ON consent_records
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_agent_author();
CREATE TRIGGER no_agent_config BEFORE INSERT OR UPDATE ON config_constants
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_agent_author();

INSERT INTO config_constants (key, tier, value, changed_by, schema_name, schema_version, produced_by) VALUES
    ('petition_stall_ms', 'OPERATIONAL', '60000', 'deployment:migration-0003', 'ConfigConstant', '1.0.0', 'deployment');
