-- Slice 7 hardening — findings of the adversarial review (SLICE_07 §6).

-- X.1: the conferral is recorded immutably for the life of the
-- environment. Only status and revision may change; the identity fields
-- (kind, matrix, tier, title, name, establisher) are frozen at
-- establishment — immutability by a positive guard, not the mere absence
-- of a mutator.
CREATE FUNCTION godhead_environment_immutable() RETURNS trigger AS $$
BEGIN
    IF NEW.kind IS DISTINCT FROM OLD.kind
       OR NEW.matrix_ref IS DISTINCT FROM OLD.matrix_ref
       OR NEW.tier IS DISTINCT FROM OLD.tier
       OR NEW.title IS DISTINCT FROM OLD.title
       OR NEW.name IS DISTINCT FROM OLD.name
       OR NEW.established_by IS DISTINCT FROM OLD.established_by THEN
        RAISE EXCEPTION 'the conferral is immutable: established under a title and name for the life of the environment (Law X.1)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER environment_immutable BEFORE UPDATE ON environments
    FOR EACH ROW EXECUTE FUNCTION godhead_environment_immutable();
