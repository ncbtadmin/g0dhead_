# Override Lineages and Release Semantics

**Status:** Analysis-only; non-canonical. D8 returned 2026-07-10: adopted as amended — riders: the IV.1/IV.5/SC-C01 constitutional amendment text is drafted with the desk's co-signature; the criteria name where the substrate-visible IV.1 wall lives once `user_overridden` leaves the protected payload.

- **Scope:** Bond/measurement separation, override compatibility, lineage concurrency, petitions, sovereign transition plans, release, effective-source selection, and P2B criteria.
- **Owning decisions:** D8; D4 only where measurement records and embedding spaces intersect.
- **Phase owner:** P2B.
- **Criteria hooks:** SC-C01–SC-C06, SC-D10, and the proposed `AC-` criteria below.
- **Amendment rows sourced:** D8 rows in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md), plus D4 measurement-split rows cross-linked there.

This is the sole normative proposal home for override and release mechanics.
The controlling roadmap and decision sheet link here and do not restate them.

---

## 1. Existing gap and record split

`LINK_SEVERED` and `LINK_FORCED` have schema and protection walls, but no
laying surface exists at `ffae6a8`. Release is less complete: IV.5 names the
sovereign's power to release, while A.7 and migration 0003 represent only
permanently laid `user_overridden: true` records.

The proposed relationship model separates identity from changing evidence:

- **Bond:** immutable endpoint identity and provenance. It carries no computed
  similarity, weight, mutable category, sever flag, or `user_overridden`
  marker.
- **`BondCategoryEvidence`:** append-only category source, policy/run identity,
  lineage epoch, and effective revision.
- **`BondQualification`:** `bond_ref · space_ref · link_policy_ref · similarity · qualified · lineage_epoch · effective_revision · run_ref`.
- **`WeightEvidence`:** `bond_ref · space_ref · weight_policy_ref · mode · weight · lineage_epoch · effective_revision · run_ref`.

The UI's “human-held” state is computed from standing lineage heads. If an
index is required, it is a separate lineage-derived projection; the protected
bond/node record never bears a mutable `user_overridden: true` marker. This
removes the Pass-6 contradiction between “immutable bond row” and “marker
updated in transaction.”

Machine bonds count in a space only where a qualification selected under that
space and applicable policy says so. An active `LINK_FORCED` head overrides
qualification in every space. Weight never rides the bond row.

## 2. Lineage and effective-source state

One persistent `OverrideLineage` exists for every applicable
`(subject_ref, override_kind)`, including lineages with no historical hand:

`subject_ref · override_kind · lineage_epoch · active_head_ref|null · released_fallback_ref|null · effective_revision · last_transition_ref`

- `lineage_epoch` advances on every sovereign lay, succession, release, or
  cross-kind supersession—while active and while empty. It defeats the
  empty-head ABA and invalidates machine work begun under an earlier hand.
- `active_head_ref` names the standing `OverrideRecord`, if any.
- `released_fallback_ref` names geometry-neutral `RELEASED_AS_STANDS` state
  when the hand lifted but lawful machine replacement has not yet won for a
  requested scope.
- `effective_revision` serializes global effective-state changes.

Machine-selected state is recorded separately per
`(subject_ref, override_kind, scope_ref)` where `scope_ref` is global or a
space/policy scope. An `EffectiveSourceSelection` carries
`lineage_epoch · selection_revision · selected_source_ref`. A machine publisher
captures both expected revisions and wins by CAS. Two same-epoch calculations
cannot both become authoritative.

For any requested scope, the literal selector is:

1. active override head;
2. CAS-selected lawful machine evidence carrying the current lineage epoch;
3. the current geometry-neutral released fallback; then
4. no effective value.

This order lets post-release evidence replace as-stands in the scope it
actually recalculated while preserving the fallback for other and future
spaces. Pre-release evidence carries an older epoch and is never restored.

## 3. Compatibility and complete transition state

By subject domain, `CATEGORY_REASSIGNED` holds nodes; the other kinds hold
bonds. On a bond:

- `LINK_FORCED ∥ WEIGHT_CORRECTED` is compatible.
- `LINK_SEVERED ∥ WEIGHT_CORRECTED` is incompatible. Laying severance
  supersedes either an active correction or an inactive released weight
  fallback in the same transaction.
- `LINK_FORCED ∥ LINK_SEVERED` is incompatible. Laying either supersedes the
  opposing active head **or released effective fallback** atomically.

Protection and effective state are different axes. A null head does not imply
that nothing is effective. Every affected lineage is therefore represented to
a command as:

`{ kind, lineage_epoch, active_head_ref|null, released_fallback_ref|null, effective_revision }`

The complete set includes the output lineage even when its expected head is
null. Commands lock that set in deterministic kind order and CAS every expected
state under one `transition_id`.

## 4. `TransitionPlan` governs every sovereign transition

Every direct or petition-granted sovereign override transition—singleton or
composite—binds one immutable, exact-hashed `TransitionPlan`. It contains:

- subject and requested act;
- the complete expected lineage-state set above;
- every active-head closure;
- every released-fallback supersession;
- resulting heads, fallback state, and effective consequences;
- content hash, actor/consent authority, and one transition ID.

The Notary or direct handler validates the entire plan and refuses any moved
head, epoch, fallback, or effective revision. It never closes or revives state
the sovereign did not review. A singleton release uses the same plan shape;
R14 therefore carries `H`, not only K/R/F.

This resolves the decisive counterexamples:

- released severance → force explicitly supersedes the null-head sever
  fallback before opening `LINK_FORCED`; and
- released weight → sever → re-force supersedes the weight fallback during
  severance, so the old corrected value cannot revive.

The two-ceremony alternative—explicit release followed by lay—is viable but
declined for routine correction. One plan preserves one sovereign act while
making every consequence visible.

## 5. Petition occurrences and terminal execution

The canonical `(subject, petition_class)` aggregate remains for recurrence and
suppression, but it no longer stores mutable history. Each ask creates an
immutable `PetitionOccurrence` binding the requested result, complete target
lineage-state set, and plan hash. Later facts are separate append-only records:

- `PetitionResolution` for decision and consent;
- `PetitionExecutionAttempt` for executed or terminally refused outcome; and
- successor provenance citing occurrence, resolution, plan, and attempt.

`change_kind` no longer conflates petition class and requested successor kind.
SILENCED binds the exact targeted lineage-state digest, not one vague subject
or one active head. A new hand/epoch creates a new question.

If execution finds `TARGET_RELEASED`, `TARGET_SUPERSEDED`, or another permanent
plan mismatch, it writes a terminal refusal attempt. The petition remains
visibly GRANTED, but zero-delay discovery excludes terminal attempts and does
not retry forever. The supervisor surfaces the failed grant until acknowledged
or superseded by a later lawful occurrence. This preserves SC-C06 without
turning an impossible grant into immortal labor.

## 6. Four-kind release semantics

Release is an attributed `OverrideReleaseRecord` and singleton
`TransitionPlan`. It closes exactly one current head, advances that lineage,
and installs a geometry-neutral fallback; other compatible kinds remain
untouched.

| Kind | Released fallback | Lawful machine replacement |
|---|---|---|
| `CATEGORY_REASSIGNED` | Corrected node category remains effective. | Fresh classification publishes a current-epoch global category selection. |
| `LINK_SEVERED` | Bond remains excluded in every space. | A current-epoch link evaluation may publish qualified existence for its space; a sovereign force supersedes the fallback globally. |
| `LINK_FORCED` | Bond existence and hand-assigned category remain as-stands. | Current-epoch qualification replaces existence per space; category evidence replaces category only through a lawful relationship-classification trigger. |
| `WEIGHT_CORRECTED` | Corrected value remains effective in every space lacking newer eligible evidence. | A current-epoch weight run CAS-publishes evidence for its space/policy scope. |

Release never falls back immediately to pre-release qualification,
classification, or weight evidence. `BondQualification`, category evidence,
and `WeightEvidence` all carry the relevant lineage epoch. Forced bonds count
toward density in every space while the active head stands; after release,
they continue as-stands in a space until that space receives lawful
post-release qualification.

When severance supersedes active or released weight state, it records the old
value historically but clears its effective fallback and advances the weight
lineage. A later force cannot revive it.

## 7. IV.1 and SC-C01 amendment

Append-only payloads and computed projections remove ordinary machine mutation
of human-held records. They do not, by themselves, legalize direct sovereign
release: IV.1 and SC-C01 currently name granted-petition consent as the only
mutation exception.

D8 therefore proposes one explicit amendment to **both IV.1 and SC-C01**:
lawful override state may change under either (a) a resolving consent naming a
granted petition or (b) an authenticated sovereign command bound to an
exact-hash `TransitionPlan`. A criterion-only exception would be shadow canon.
IV.5 gains the release representation and as-stands semantics above.

`OverrideRecord`s and release/transition records remain append-only. The
lineage control record and effective-source selections are the CAS substrate,
not the protected datum.

## 8. Trial evidence contract

This annex exports the effective-source witness consumed by
[TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md):

- `ACTIVE_OVERRIDE { head_ref, lineage_epoch }`;
- `RELEASED_AS_STANDS { release_ref, lineage_epoch, effective_revision }`;
- `MACHINE { evidence_ref, lineage_epoch, effective_revision, scope_ref, policy_ref, run_ref }`.

A trial binds the resolved source and every relevant lineage epoch even where
`active_head_ref` is null.

## 9. Acceptance criteria

P2B pins citable `AC-` criteria proving at least:

- no computed value or mutable held marker lives on the immutable bond payload;
- machine bonds count only under selected qualification for the bound
  space/policy, except active or released-as-stands force semantics above;
- all four release kinds preserve as-stands and exclude pre-release evidence;
- lawful post-release evidence displaces fallback only in its scope;
- two same-epoch publications yield one CAS-selected source;
- released sever → force and released weight → sever → re-force cannot produce
  contradictory or revived state;
- every singleton and composite sovereign transition hashes the complete
  active-and-inactive lineage-state set;
- recurrence never rewrites occurrence, consent, or execution evidence;
- permanent target mismatch becomes a surfaced terminal refusal, not endless
  rediscovery;
- direct and petition-granted plans write complete provenance; and
- trial evidence can cite active, released, or machine effective sources.

`links_for_node`, `list_matrices`, and the laying/release surfaces remain P2B
work. Final DDL belongs to ADR-2/ADR-4 after D8 returns; the state identities,
selectors, authority, and failure semantics above are decision inputs.
