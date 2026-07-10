-- Slice 10 — the J-floor (docs/dev/SLICE_10.md; ruling G9): the substrate of
-- Section J. The MandateRecord (C.4) with its human-authorship wall, and the
-- ProvenanceChain as a table (C.2; ruling G8) — append-in-flight, rooted in
-- a human hand, frozen at birth. Behavior (fetch execution, manifest maps)
-- is Slice 11's; the fetch layer itself stays absent behind the no-HTTP wall.

-- C.4 — the human-authored charter of every outward act (Handbook §1.4).
CREATE TABLE mandates (
    mandate_id      uuid PRIMARY KEY,
    kind            text NOT NULL CHECK (kind IN ('CANON','WRIT')),
    teacher_env_ref uuid REFERENCES environments(env_id),
    matrix_ref      uuid REFERENCES matrices(matrix_id),
    demands         jsonb NOT NULL,
    trip_budget     jsonb NOT NULL,
    authored_at     timestamptz NOT NULL DEFAULT now(),
    schema_name     text NOT NULL,
    schema_version  text NOT NULL,
    produced_by     text NOT NULL,
    produced_at     timestamptz NOT NULL DEFAULT now(),
    -- The tiers do not smear (§1.4): a canon is for a Teacher, a writ for a
    -- matrix, and each names exactly its own recipient.
    CHECK (kind <> 'CANON' OR (teacher_env_ref IS NOT NULL AND matrix_ref IS NULL)),
    CHECK (kind <> 'WRIT'  OR (matrix_ref IS NOT NULL AND teacher_env_ref IS NULL))
);

-- SC-J01 at the substrate: mandates are human-authored by construction —
-- the same wall consents stand behind (a UUID-shaped author IS an agent).
CREATE TRIGGER no_agent_mandates BEFORE INSERT OR UPDATE ON mandates
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_agent_author();

-- A mandate is a charter: frozen at birth. A correction is a new mandate.
CREATE FUNCTION godhead_mandate_immutable() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'a mandate is immutable; the sovereign authors a new one (C.4)';
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER mandates_immutable BEFORE UPDATE ON mandates
    FOR EACH ROW EXECUTE FUNCTION godhead_mandate_immutable();
CREATE TRIGGER no_delete_mandates BEFORE DELETE ON mandates
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

-- C.2 — the ProvenanceChain table (external-origin provenance; ruling G8).
-- One row per chain entry; the chain_ref is the subject whose arrival story
-- the chain narrates (for external material: the quarantine item).
CREATE TABLE provenance_chains (
    chain_ref        uuid NOT NULL,
    link_seq         int NOT NULL CHECK (link_seq >= 0),
    kind             text NOT NULL CHECK (kind IN
                         ('CANON','WRIT','BRIEF','FETCH','FOLLOW_UP','REFINEMENT','ADMISSION')),
    actor_job_ref    uuid NOT NULL REFERENCES job_records(job_id),
    mandate_ref      uuid REFERENCES mandates(mandate_id),
    prompt_or_reason text NOT NULL,
    produced         jsonb NOT NULL DEFAULT '[]',
    at               timestamptz NOT NULL DEFAULT now(),
    schema_name      text NOT NULL,
    schema_version   text NOT NULL,
    produced_by      text NOT NULL,
    produced_at      timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (chain_ref, link_seq)
);

-- The chain's grammar, at the substrate: the root (link_seq 0) begins in a
-- human hand (CANON|WRIT|BRIEF); a CANON or WRIT root cites its mandate;
-- append means append — entry N requires entry N-1 to already stand.
CREATE FUNCTION godhead_chain_append() RETURNS trigger AS $$
BEGIN
    IF NEW.link_seq = 0 THEN
        IF NEW.kind NOT IN ('CANON','WRIT','BRIEF') THEN
            RAISE EXCEPTION 'PROVENANCE_INCOMPLETE: chain root is %; every chain begins in a human hand (C.2)', NEW.kind;
        END IF;
        IF NEW.kind IN ('CANON','WRIT') AND NEW.mandate_ref IS NULL THEN
            RAISE EXCEPTION 'PROVENANCE_INCOMPLETE: a % root cites its mandate (C.4)', NEW.kind;
        END IF;
    ELSE
        IF NOT EXISTS (SELECT 1 FROM provenance_chains
                       WHERE chain_ref = NEW.chain_ref AND link_seq = NEW.link_seq - 1) THEN
            RAISE EXCEPTION 'PROVENANCE_INCOMPLETE: chain % has no entry %; append means append (§4.2)',
                NEW.chain_ref, NEW.link_seq - 1;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER chain_append_grammar BEFORE INSERT ON provenance_chains
    FOR EACH ROW EXECUTE FUNCTION godhead_chain_append();

-- Frozen at birth: an arrival story is never revised, only extended.
CREATE FUNCTION godhead_chain_immutable() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'a chain entry is immutable; the story extends, it never revises (C.2)';
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER chains_immutable BEFORE UPDATE ON provenance_chains
    FOR EACH ROW EXECUTE FUNCTION godhead_chain_immutable();
CREATE TRIGGER no_delete_chains BEFORE DELETE ON provenance_chains
    FOR EACH ROW EXECUTE FUNCTION godhead_forbid_delete();

CREATE INDEX chains_by_ref ON provenance_chains (chain_ref, link_seq);

-- The known-source registry SC-J02's resolution half checks source_id
-- locators against. Empty in v1 by design: the fetch layer is absent, so
-- v1's resolvable locator set is exactly the well-formed URIs — the day
-- this fills is the day sources ship.
INSERT INTO config_constants (key, tier, value, changed_by, schema_name, schema_version, produced_by) VALUES
    ('known_source_ids', 'OPERATIONAL', '[]', 'deployment:migration-0013', 'ConfigConstant', '1.0.0', 'deployment');
