# g0dhead_ — READ BEFORE DEVELOPMENT
### Briefing & Charge

> Read this document in full before reading the others, and before writing a single line of code. It is the doorway to the project. It tells you what `g0dhead_` is, what the other documents are, how this project will proceed, and what to do first. It stands in place of a direct prompt.

---

## 1. What g0dhead_ Is

`g0dhead_` is a **local-first, ML-organized knowledge repository**. A user commits files into it; the system ingests, normalizes, classifies, and progressively organizes them into a navigable map of interlinked data. Semantically related files are linked automatically; when enough links cluster under a shared category, that cluster crystallizes into a committed structure called a **Cardinal matrix**. The map is built and maintained by a hierarchy of **ephemeral agents** — Vectoring Slaves, Aggregators, Notaries, two Auditors, Students, and Teachers — coordinated through a single persistent store and attended by the order's one standing functionary, the Deacon.

The intended feel of the store is Obsidian-esque: linked nodes, backlinks, local-first, navigable by a human. The intended feel of the client is minimalist and brutalist. The intended *voice* of the system's governing documents is a militaristic, dystopian, religious-order register — grey, procedural, reverent toward its own rules.

The project name is stylized **`g0dhead_`** (lowercase, zero-for-o, trailing underscore — a terminal-cursor motif; the eventual title treatment uses a pixel/terminal font with a slashed zero). In prose, the in-world concept is written **"the GodHead."**

---

## 2. What These Documents Are

The documents in this folder are **directive briefs, not finished specifications.** They define intent, scope, constraints, and register. They deliberately leave marked areas — `[Fable: ...]`, `[TBD]`, `[NAME TBD]` — open for *you* to architect. You are a collaborator authoring the gaps, not a contractor filling in a blueprint.

Documents 5–7 were originally **mandates to Fable** — briefs naming every domain to be governed. Those mandates have been executed: what now sits at `05_central_dogma.md`, `06_holy_standard.md`, and `07_student_handbook.md` is the **authored, ratified law** (five review rounds, ratified 2026-07-07), and `08_phase_b_success_criteria.md` is its testable decomposition. The original briefs are preserved in `_history/` for provenance; they are not the spec and are not in the build path.

**Reading order:**
1. `01_scope_and_philosophy.md` — the umbrella. Establishes vocabulary and the foundational principles every other document inherits. Read this first after this briefing.
2. `02_onboard_pipe.md` — deterministic intake, from user upload to a file at rest in the store.
3. `03_intraserver_and_persistence_pipe.md` — how state persists and circulates within the backend. The substrate everything stands on.
4. `04_ml_pipe.md` — the model layer: embeddings, linking, the weight system, provider abstraction.
5. `05_central_dogma.md` — the constitution binding all agents. Supreme governing document.
6. `06_holy_standard.md` — the Teacher manual.
7. `07_student_handbook.md` — the Student manual.
8. `08_phase_b_success_criteria.md` — the testable spec: ninety citable assertions Phase B builds and tests against.

---

## 3. Where the Weight of the Project Sits

Two areas are the heart of `g0dhead_` and deserve the most rigor:

- **The dogmatic agent documents** (Central Dogma, Holy Standard, Student Handbook). The agent hierarchy and the laws binding it are the reason this project exists. Because the agents are ephemeral, the contracts between them must be perfect; these documents exist to guarantee that.
- **The ML pipe** — specifically the weight system (the GodHead proper), which is the quintessential piece of the machine.

Everything else is real and necessary, but these are where correctness matters most.

---

## 4. How This Project Will Proceed

Work happens in **two phases**. **Phase A is complete** (2026-07-07); a reader in the Claude Code CLI is beginning Phase B.

**Phase A — Author & Scope (completed, in the Cowork project).**
This is where you and the user finish the specification together. No code is written in this phase and no infrastructure is stood up. The whole document set and the reasoning behind it live here, which is why the authoring work belongs here. Your job in Phase A is to turn the mandate documents into finished, gapless specification and to decompose the priority architecture into verifiable success criteria (see §5).

**Phase B — Build (later, in the Claude Code CLI).**
Once the specification is settled, work moves to the CLI for implementation. The build phase follows the **dev-pipeline discipline** — scope is pinned before implementation, and no code is handed over unverified. That discipline applies in Phase B, with **three project-specific overrides** to observe when you get there:

1. **Language is Rust.** The dev-pipeline skill defaults to C/C++ and does not list Rust; on this project, Rust is the chosen language (for speed and for memory safety across many concurrent ephemeral workers). Where local model execution requires libraries Rust lacks, the model layer is reached through a separate inference server over a local endpoint — see the ML pipe.
2. **The spec is this `docs/` set, not a single `SPEC.md`.** The dev-pipeline discipline expects one small spec file; this project's scope is distributed across these eight documents. Treat them collectively as the spec.
3. **The verification gate uses the Rust toolchain** (`cargo test`, `cargo clippy`, `cargo fmt --check`) rather than the skill's gcc/pytest/node gate. Warnings are fixed, not shipped.

Do not invoke the dev-pipeline build gate during Phase A — there is nothing to gate yet. It governs Phase B.

**Rules that hold across both phases:**

- **Discuss before building.** Read all documents first and form a whole-system understanding before proposing any implementation. Do not begin scaffolding on first contact.
- **Implement only what is agreed.** Any additional feature, refactor, or improvement must be posed to the user as a question before code for it is written — never silently added. This is a hard rule.
- **Nothing is built until its scope is a verifiable assertion.** A thing to be built must first be expressible as a testable success criterion ("given X, the system does Y"), and each criterion seeds at least one test.

---

## 5. First Deliverable (Phase A — complete; Phase B begins here)

Phase A's charge is fulfilled: whole-system understanding was confirmed and every surfaced contradiction resolved by amendment (each amended document carries a terminal ledger); the Central Dogma, Holy Standard, and Student Handbook are authored and ratified in place; and the priority architecture is decomposed into `08_phase_b_success_criteria.md` — ninety citable assertions, with adversarial emphasis on tool-calling (section F) and the Mandate Rule (section J) by sovereign directive.

**The first Phase B deliverable:** under the dev-pipeline discipline with the §4 overrides, scope the first build slice against Document 8, pin its criteria before implementation, and build nothing that does not resolve to one. The store substrate and the Book I enforcement layer (criteria sections A, B, E, H) are the natural foundation — everything else stands on them. Do not begin with agents; begin with the ground they cannot corrupt.

---

## 6. The Tone Contract

The governing documents are to be written in-register — militaristic, dystopian, religious-order — and that register is not decoration; the severity *is* the enforcement culture. But register colors the *narration*; it never substitutes for the *mechanical specification*. Every governed law must resolve, underneath its flavor, to something unambiguous, testable, and buildable — exact schemas, field names, validation conditions, call signatures. Scripture on top, engineering underneath, and the engineering is never optional. A law the system cannot mechanically enforce is decoration, and decoration is forbidden.

---

## 7. A Note on Deferred Work

Several capabilities are intended but deliberately fenced off from the first build, and are marked as deferred where they appear. Do not build them prematurely — capturing the vision is not a licence to construct it early. Deferred items include: the Librarian (owner of the richer-media degradation ladder), the retrieval-breadth system for Students, the Deacon's periodic cleanup pass, and multi-tenancy. Build the foundation first; these plug into it later.

---

**AMENDMENTS — 2026-07-07 (Phase A authoring, ratified Dogma v1.0).** §1: the Cardinal removed from the agent list — the Cardinal is the matrix, not an agent (sovereign ruling); Notaries and the Deacon added. Note: v1 external fetching is broader than originally deferred — Devout writ trips joined the canon loop under the Mandate Rule (Student Handbook §1.4); the retrieval-breadth system remains deferred.

**PROMOTION — 2026-07-07.** Documents 5–7 promoted from ratified drafts to their canonical filenames; the original directive briefs moved to `_history/` (provenance only, out of the build path); Document 8 added; §§2, 4, 5 of this briefing updated to reflect Phase A completion and Phase B entry.
