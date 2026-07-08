# Phase B — Slice 4: Weights & the ML Floor
### Pinned scope — signed off 2026-07-07

> Section M of Document 8: the model layer beneath the agents (doc 4) —
> endpoints, embeddings, links, and the dial-able weight system. Floor-first:
> every criterion is testable without an external service.

## 1. Pinned criteria (Document 8, section M — 6 criteria)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-M01 | Ingestion marks recalc-eligibility only; execution happens per configured trigger (manual / on-add / interval) | `sc_m01_triggers` |
| SC-M02 | Below the coherence threshold, weights are inert in every consumer query | `sc_m02_weights_inert_below_threshold` |
| SC-M03 | Same ingestion completes under assisted and floor modes; floor makes zero reasoner calls | `sc_m03_mode_dial` |
| SC-M04 | Empty endpoint roster: every stage completes or degrades to floor, zero crashes | `sc_m04_empty_roster` |
| SC-M05 | One persisted embedding per node; repeat requests read, never recompute | `sc_m05_embed_once` |
| SC-M06 | Embedder-down intake rests normalized+linkless, flagged for backfill; backfill never touches the atom | `sc_m06_backfill` |

## 2. What this slice builds

- **Migration 0004**: `CREATE EXTENSION vector`; `embeddings` (one row per
  node, `vector(256)`), `links` (first-class edges, doc 3 §2.3: canonical
  source<target ordering, similarity, weight, category, user_overridden),
  `rebalance_state` (per-category eligibility). Operational constants:
  `link_similarity_threshold` (0.30), `weight_mode` ("floor"),
  `rebalance_trigger` ({"kind":"manual"}). `coherence_threshold` remains
  sovereign and **unseeded** — tests set it as the sovereign.
- **Schemas**: EmbeddingRecord, LinkRecord, RebalanceState.
- **Store methods**: `put_embedding` (converges on conflict, logs EMBEDDED,
  marks eligibility), `get_embedding`, `embedding_backlog` (the SC-M06
  "flagged for backfill" surface), `similar_nodes` (native pgvector cosine),
  `draw_link` (canonical order, honors `user_overridden`, logs LINK_DRAWN,
  marks eligibility), `links_by_category`, `set_link_weight` (CAS; an
  overridden link refuses OVERRIDE_CONFLICT — fixed stars), `live_weights`
  (the consumer query: cites `config_rev` per Law VI.1; density below
  `coherence_threshold` ⇒ inert), `rebalance_state`, `clear_rebalance_eligibility`.
- **`crates/godhead-ml`**:
  - `roster` — the uniform endpoint interface (doc 4 §2): `Embedder` and
    `Reasoner` traits, alias-keyed roster, per-invocation routing; an empty
    roster routes to the floor everywhere, never errors.
  - `lexical` — the built-in floor embedder: hashed bag-of-words, 256 dims,
    L2-normalized, deterministic. The Ollama/HTTP endpoint is a later
    config swap behind the same trait (deferred: nothing live to verify).
  - `slave` — the Vectoring Slave: backlog → embed → persist → flag; one
    labor, volume its virtue. No embedder in the roster ⇒ backlog remains,
    quietly.
  - `aggregate` — consolidation: links from vector proximity at
    `link_similarity_threshold`; weight recalculation per mode — **floor**:
    `w = sim / √(deg(a)·deg(b))` (degree-normalized similarity; resolves
    doc 4 §5.3's open floor-method marker); **assisted**: floor × reasoner
    multiplier, degrading to floor when no reasoner is rostered.
  - `rebalance` — the trigger machinery (doc 4 §5.2/§6.4): ingestion marks
    eligibility (store-side); `rebalance_now` (human actor, no job
    identity) executes on demand; `rebalance_tick` evaluates the configured
    standing trigger — manual ⇒ never, on-add ⇒ when eligible, interval ⇒
    when eligible and elapsed. Nothing recalculates on system initiative.
- **SC-C07 ledger**: "invoking rebalance outside a user-configured trigger"
  is claimed here — execution paths take a human actor or a configured
  trigger; no job-identity path exists.

## 3. Non-goals

- No Postulant emergence, no matrices (section D owns VI.2 bookkeeping).
- No Ollama/vLLM/API endpoints — the traits are the seam; integration
  waits for a live server to verify against.
- No advisory legibility UI (animated lines, escalating warnings) — the
  ML layer computes state; rendering is the frontend's, later.
- No link/weight override *laying* surfaces beyond what slice 3 built —
  but recalc already works around `user_overridden` links.

## 4. Edge cases

- Repeat slave run over an embedded scope: zero embedder calls.
- Overridden link untouched by recalc (structural skip + store rejection).
- `live_weights` with no sovereign threshold set: refuses (Law VI.1 —
  a density evaluation must cite config), never guesses.
- Interval trigger: tick before elapse does nothing; after, executes once
  and clears eligibility.
- Self-links impossible (substrate CHECK); duplicate links converge on
  the canonical ordering.

## 5. Delivery — gate passed 2026-07-07

All 6 criteria green plus one regression test (`tests/m_ml.rs`, 7 tests);
slices 1–3 unregressed — 45 tests workspace-wide; fmt/clippy/test clean.
pgvector 0.8.4 confirmed live on Railway; migration 0004 applied.

**Adversarial review (multi-agent, 19 agents, 3 lenses × verify):** 16
findings raised, 6 refuted, 10 confirmed — all 10 fixed before delivery:

1. *Stranded jobs* — slave and aggregator labors now end every mid-pass
   failure in a Law VII refusal (mirroring the Notary); no job is left
   RUNNING. Pinned by `backfill_contains_per_node_failure`.
2. *Head-of-line blocking* — `backfill_tick` contains failures per node
   (`BackfillSummary { embedded, failures }`), continuing past a broken
   derivative instead of aborting the pass.
3. *Lost-update race + double execution on eligibility* — replaced the
   post-pass unconditional clear with an atomic pre-pass **claim**
   (`claim_rebalance_eligibility`: `UPDATE … WHERE category AND eligible`);
   marks laid by concurrent ingestions survive; N racing ticks yield one
   execution; a failed pass restores the mark it consumed. Root-cause
   correction: `draw_link` no longer marks eligibility — doc 4 §5.2 marks
   on *ingestion* events, and consolidation is not ingestion.
4. *Fabricated citation* — an unset sovereign threshold now records a NULL
   `config_rev`, never revision 0 (Law VI.1: cite what was read).
5. *Atomicity & races in the store* — `put_embedding` is one transaction
   (vector + log + eligibility mark); `draw_link` uses ON CONFLICT (two
   concurrent first-draws arbitrate cleanly) and a single-statement
   overridden-guard; `set_link_weight` staleness in consolidate is
   retried per Law XI.3 (lose → re-read), standing down if a human hand
   lands mid-pass; `links_touched` counts each undirected pair once,
   drawn from its lesser endpoint only.

Design notes: floor weight formula `w = sim / √(deg(a)·deg(b))` resolves
doc 4 §5.3's open marker; `store_now()` added to the Store trait (Law XII —
elapsed time is judged against the store's clock only); the lexical floor
embedder is FNV-1a hashed unigrams+bigrams, 256-dim, L2-normalized, behind
the `Embedder` trait an Ollama endpoint later replaces via config.
