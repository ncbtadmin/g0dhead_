-- Slice 9 hardening (adversarial review): the consistency walk's access
-- paths. Both tables are append-only and grow forever; every walk of one
-- scriptorium must not pay for every scriptorium ever.
CREATE INDEX refined_artifacts_by_env ON refined_artifacts (env_ref, produced_at);
CREATE INDEX returns_by_instruction ON returns (instruction_ref);
