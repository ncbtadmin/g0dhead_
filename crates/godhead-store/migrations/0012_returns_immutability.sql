-- Slice 9 hardening, round 2 (adversarial review): total immutability.
--
-- The 0010 trigger compared six named columns, which left the provenance
-- envelope (produced_by, produced_at, schema_*) rewritable on a flagged
-- Return — who certified, and when, could be silently falsified. A
-- flagged Return is now frozen whole; a correction is a fresh Return.
CREATE OR REPLACE FUNCTION godhead_return_immutable() RETURNS trigger AS $$
BEGIN
    IF OLD.flagged THEN
        RAISE EXCEPTION 'a flagged Return is immutable; a correction is a fresh Return (Handbook §3.1)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- A derivation record has no lawful UPDATE path at all: 0010 forbade
-- DELETE with the rationale that a wrong derivation is FOUND by the
-- re-derivability/closure walk — leaving UPDATE open defeated exactly
-- that (rewrite source_refs/content_sha until the walk passes). Frozen
-- at birth.
CREATE FUNCTION godhead_refined_immutable() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'a derivation record is immutable; debris is found by the walk, never rewritten (Handbook §1.2)';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER refined_immutable BEFORE UPDATE ON refined_artifacts
    FOR EACH ROW EXECUTE FUNCTION godhead_refined_immutable();
