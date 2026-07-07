# g0dhead_ — Document 4
# The ML Pipe

> Governs everything model-driven: how models are reached, the two classes of model, how embeddings build the graph, how weights are recalculated, and how the system recommends (never forces) intelligent processing. This is a crown-jewel document — the weight system (the GodHead proper) lives here. Inherits all principles from Document 1.

## 1. Scope

This document governs the ML substrate agents run on. It does **not** specify agent behavior (that is the dogma documents) — only the model layer beneath them.

## 2. The Uniform Model-Endpoint Interface

**2.1 — Everything is an endpoint.** The core never talks to a *model*; it talks to an **endpoint** — something it sends a request to and receives a result from. A locally-hosted model and a remote API are the same kind of thing to the core.

**2.2 — Separate inference server for local models.** Local models run in their own process (e.g. Ollama, vLLM) exposing a local endpoint. The core coordinates; purpose-built tools do inference. This keeps the core doing what it is good at and sidesteps Rust's thin native ML ecosystem. `[Fable: which local inference server.]`

**2.3 — Local and remote are interchangeable; routing is per-invocation.** The provider abstraction holds a **roster** of available endpoints. Each agent invocation selects one per job: cheap local models for high-volume work, stronger APIs for work needing reasoning muscle. Local is never chosen *instead of* API — the two coexist and are routed per task. Local-first is the endgame; remote APIs remain fully usable alongside.

**2.4 — Graceful degradation is routing, not a special case.** The handleless-hammer principle is realized here: if no reasoner endpoint is available, stages fall back to their deterministic or numerical floor. "No model" is just an empty roster, handled the same way everywhere.

## 3. Two Model Classes

The system uses **two fundamentally different kinds of model.** Conflating them is a category error.

**3.1 — Embedders (cheap, deterministic-floor geometry).** An embedder turns text into a **vector** — a position in meaning-space. It does not reason or generate; it *locates.* Semantically similar files land near each other even with no shared words, which makes "relatedness" **mathematically measurable** as vector distance.
- **Local by default, always.** Embedders are small and run well locally; embeddings run on the local embedder even when reasoning is routed to a remote API. This keeps the map's geometry free, private, and offline-capable. `[Fable: embedding model choice.]`
- Functions as a **second deterministic floor** — far smarter than filetype-bucketing, but still no reasoning tokens. A large share of the map's structure is built here, cheaply, reserving reasoners for actual judgment.

**3.2 — Reasoners (judgment, provider-routed).** The large language models. They *understand and generate* — judge, write, investigate, draft. Used by the AI classifier layer, the Auditors, Students, Teachers, and the weight system's assisted mode. Provider-routed per §2.3.

## 4. Embeddings → Links → Matrices (the core mechanic)

This is the heart of the GodHead.

**4.1 — Embedding.** Every normalized file is run through the embedder. One persisted vector per node.

**4.2 — Linking.** The system measures vector proximity and **draws links** between semantically close files. Links are **first-class database objects**, never written into file bodies (atoms and bonds are stored separately — see the persistence pipe). A link records: `source · target · similarity/weight · category · user_overridden · timestamps`.

**4.3 — Matrix emergence.** When enough links cluster under a shared **category**, and link-density crosses the **coherence threshold**, that cluster crystallizes into a **Postulant matrix** — *emergent*, not declared, recorded by the Aggregator as deterministic bookkeeping (Dogma VI.2). Commitment is a separate, tried act: only the audit path — Auditors, Joint Proposal, sovereign consent, a Notary's hands (Dogma VI.3) — professes a Postulant into a **Cardinal matrix.** The coherence threshold is a single system-wide constant that also governs where weights begin to matter — the same number, the same event: a cluster crossing it is simultaneously "dense enough for weights to matter" and "dense enough to become a Postulant." `[Fable: threshold value — empirical, unchanged.]`

**4.4 — User override (category + individual links).** The user may override both the **category** binding a matrix and **individual links** (sever a wrong link; force a correct one the ML missed). Overrides set `user_overridden:T`. **Agents must never silently or unilaterally revert a human override** — a hard rule carried into the dogma documents. Full manual graph authorship is out of scope for the first version.

## 5. The Weight System (crown jewel)

**5.1 — Weight = contextual influence, not magnitude** (Document 1, §2.4). Physical size and node count are naive proxies the system may measure but must not obey.

**5.2 — Event-driven recalculation.** Weights are static between events and become recalculation-**eligible** on **ingestion events** — new files or batches, and Student-returned provenance (which is itself an ingestion of new material). Recalculation **executes** only per a trigger the user has chosen or issued (§6.4); a user-configured standing trigger — including an interval — is standing consent, not system initiative. Nothing recalculates on the system's own initiative.

**5.3 — LLM-assisted, but dial-able.** The weight system is intentionally *a little* overkill: **reasoner-assisted** weighting is the intended default (a reasoner judges whether apparent magnitude deserves real influence). But this is the single biggest cost and latency variable in the system, so it is **dial-able**:
- **Assisted mode** (reasoner in the loop) — rich; the eventual default once self-hosted.
- **Numerical floor** (embeddings and statistics only, no reasoner) — graceful fallback for metered APIs or speed.
This applies the handleless-hammer principle to the crown jewel: richness when self-hosted, a real floor when not. `[Fable: the numerical floor method — the statistics used when no reasoner is in the loop.]`

**5.4 — Below the coherence threshold, weights are inert.** Loose nodes float with no weight influence and no matrix until density crosses the line.

## 6. Advisory Recommendation Model (never forced)

**6.1 — The system never forces ML** (Document 1, §2.5). It only ever *recommends.* Weight accuracy is the user's responsibility; the system's job is to make inaccuracy **impossible to miss.**

**6.2 — Graduated legibility.**
- **Static vector lines** — baseline, no recommendation active.
- **Animated / brightened vector lines** — a soft recommendation: this cluster would benefit from ML rebalancing. The user decides.
- **Escalating warning** — when a cluster grows well past the point where ML would meaningfully help and the user has still not invoked it, the recommendation intensifies (for example, an on-screen warning). Still not forcing — just refusing to let drift go silent. `[Fable: escalation threshold and warning UI.]`

**6.3 — Magnitude drives recommendation intensity, not automatic action.** Magnitude sets *how loudly the system suggests*, never *whether it acts.* Candidate magnitude signals the deterministic floor can cheaply flag: node-count delta, embedding displacement, distributional shift. The floor may flag; only a reasoner interprets (Document 1, §2.4). ML is never compelled.

**6.4 — Rebalance trigger is user-configurable.** The user owns the **cadence** of rebalancing: manual "rebalance now," on-file-add, on an interval, or combinations. `[Fable: parameter set.]` The system never rebalances without a trigger the user has chosen or issued.

**6.5 — ML-decides-when, frontend-renders-how.** The "alive / biomechanical" vector-line motion is **rendering** (frontend animation), not ML. The ML layer only decides *when* a line should animate (when a recommendation is active); the frontend owns *how* it looks. This boundary is strict — no reasoner is ever spent on a job the frontend should do.

---

**AMENDMENTS — 2026-07-07 (Phase A authoring, ratified Dogma v0.5).** §4.3: emergence is two-stage — threshold-crossing yields a **Postulant** (Aggregator bookkeeping); the audit path alone professes a Cardinal (Dogma VI). §5.2: recalculation *eligibility* (on ingestion) distinguished from *execution* (per user-chosen trigger); a standing trigger is standing consent — reconciling §5.2 with §6.4.
