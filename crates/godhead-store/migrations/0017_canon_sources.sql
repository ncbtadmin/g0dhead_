-- Slice 11 — v1 canon-fetch (sovereign ruling 2026-07-09; docs/dev/SLICE_11.md
-- §0.2; C.4 sources amendment). A CANON mandate's freeform `demands` clauses
-- stay the coverage surface, untouched; the mandate gains `sources` — typed
-- locators under the identical SC-J02 wall as writ targets — so
-- FETCH_PER_CANON executes against concrete, sovereign-named targets by the
-- same mandate-rooted machinery as a writ. A writ leaves `sources` empty (its
-- targets live in `demands`); a canon with empty `sources` has no v1 trip.
ALTER TABLE mandates ADD COLUMN sources jsonb NOT NULL DEFAULT '[]';
