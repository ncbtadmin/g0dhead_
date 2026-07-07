# g0dhead_ — Document 1
# General Scope & Philosophy

> The umbrella document. All other documents inherit its vocabulary and principles. `[TBD]` and `[Fable: ...]` mark deliberate open questions.

---

## 1. What g0dhead_ Is

`g0dhead_` is a **local-first, ML-organized knowledge repository**. A user commits files into it; the system ingests, normalizes, classifies, and progressively organizes them into a navigable map of interlinked data — where the coherent, committed structures are **Cardinal matrices**, and the map is maintained by a hierarchy of **ephemeral agents** (Vectoring Slaves, Aggregators, Notaries, Auditors, Students, Teachers).

The intended feel of the store is **Obsidian-esque**: linked nodes, backlinks, local-first, navigable by a human. The intended feel of the *client* is **minimalist, brutalist/near-gothic** — a deliberate tool for someone who knows what they are looking at, not a consumer onboarding funnel.

**Title treatment:** the project is stylized `g0dhead_` (lowercase, zero-for-o, trailing underscore evoking a terminal cursor). The eventual UI title uses a pixel/terminal font with a slashed zero. In prose, the in-world concept is written "the GodHead."

**Audience:** a single expert operator now, with a small trusted multi-user future intended (friends and family on a home server). This does not raise the friendliness bar — the operator is happy to provide technical support — but it means the system must work *well*, and the persistence layer must not foreclose eventual multi-user distinction.

---

## 2. Foundational Principles

These govern every downstream document.

**2.1 — Ephemeral, invoked agents.**
No agent runs continuously. Each is a stateless worker: spins up on an event, reads state from the persistent store, works, writes state back, terminates. **The persistent store is the single source of truth.** Agents hold nothing between invocations. *(The Deacon and the Librarian are standing functionaries, not agents, and are exempt from this — see below and the relevant documents.)*

**2.2 — Standardization is survival, not style.**
Because agents are ephemeral and hold no memory, the **contracts between them must be perfect.** A reborn agent can only function if what it reads is in an exactly predictable shape every time — there is no institutional memory to absorb inconsistency. Every data structure, handoff format, flag, and schema is **rigidly standardized**. This is the load-bearing discipline of the entire system.

**2.3 — AI-optional baseline everywhere (the handleless hammer).**
Every stage has a deterministic non-AI floor, with AI as an enhancement layer. The floor is real but incomplete — a hammer with no handle: you can hit things with it, but it isn't really a hammer. Its purpose is to **degrade gracefully** and to **conserve AI context** by absorbing deterministic grunt work. Because agents are ephemeral and context-reliant, every token spent on work plain code could have done is wasted context. The floor exists to protect that context.

**2.4 — Weight = contextual influence, decoupled from physical magnitude.**
"Weight" means *how much a thing should influence the map* — never *how big it is.* Size and node count are naive proxies the system may measure but must not obey. A vast but meaningless file should carry near-zero weight; a small but significant one may carry high weight. Judging whether apparent magnitude translates into real influence is definitionally an ML task; the deterministic floor may measure size but cannot interpret it.

**2.5 — Advisory by design, not authoritative.**
The system **never forces ML** or acts on the user's behalf without invocation. It makes weight-state *legible* — static versus animated vector lines, escalating warnings — so the user can never be *unaware* that weights may need rebalancing, but the decision is always theirs. Weight accuracy is the user's responsibility; the system's job is to make inaccuracy impossible to miss.

**2.6 — Traceability via append-only logs.**
Every significant event writes a log snapshot. Logs never overwrite; each rotates the prior into a redundancy folder. This underpins provenance, security review, and recovery system-wide.

**2.7 — Deliberate human-invoked seam.**
Automatic processing carries a file from commit to "at rest in the GodHead with baseline structure," then stops. All deeper AI-driven organization is human-invoked. This pause is the boundary between deterministic intake and intelligent processing.

---

## 3. High-Level Topology

```
FRONTEND (public, GitHub) — brutalist/gothic desktop client, Windows + Linux
  • file selection + staging + deliberate "commit" trigger only; no processing
        │
        ▼  commit
BACKEND (server-resident; Railway + Postgres scaffold now → own server later)
  • the GodHead persistent store (single abstracted store interface)
  • the Supervisor (bookkeeping process; not a hypervisor, not separate compute)
  • ephemeral job execution
  • model-execution abstraction (local inference server + remote APIs)
  • standing functionaries: the Deacon (threshold/gate), the Librarian (deferred)
```

The frontend does only selection, staging, and commit. All compute and coordination live on the backend. Heavy model calls may be routed to higher-compute hardware while the server remains coordinator; the mechanism is detailed in the ML pipe.

---

## 4. Technology Intent

- **Language: Rust**, chosen for speed and for memory safety across many concurrent ephemeral workers — a natural fit for a persistent, agent-coordinated backend. Rust's ML ecosystem is comparatively thin; local-model execution is therefore reached through a separate inference server over a local endpoint, so local and remote models look identical to the core. Detailed in the ML pipe.
- **Persistence: PostgreSQL with `pgvector`**, one store holding both relational state and embedding vectors. Raw file bytes are stored on disk by reference. The store is abstracted behind one interface so the concrete substrate can swap (hosted → self-hosted → local) via configuration, not rewrite. Detailed in the persistence pipe.
- **Local capability** is a first-class requirement, not an afterthought.

---

## 5. Philosophy

`g0dhead_` exists to map information **the way it actually relates**, not the way a folder hierarchy pretends it does. Most tools store data; this system tries to let data **find its own shape**, with the only fully-coherent structures being the Cardinal matrices the agents converge on. Even if everything else stays a little loose, those should be real — a map that is actually a map.

The agent hierarchy — students, teachers, auditors, notaries, and the standing functionary who serves them — is an attempt to model how understanding actually gets built: gathering, organizing, checking, disagreeing, reconciling, committing. It is deliberately ambitious. It is meant to be built.

---

**AMENDMENTS — 2026-07-07 (Phase A authoring, ratified Dogma v0.5).** §1 and §5: the Cardinal removed from the agent lists — the Cardinal is the matrix itself (sovereign ruling); Notaries added; "functionaries" singularized (the Deacon stands alone; the Librarian remains deferred).
