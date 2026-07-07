# g0dhead_ — Document 2
# The Onboard Data Pipe
### From User Upload to the Threshold of the GodHead

> Governs the deterministic intake path: from a user committing files to those files resting in the store with baseline structure, awaiting human-invoked intelligent processing. Inherits the Central Dogma and terminates into the store defined in the persistence pipe.

## 0. Charge

Specify the intake path a file travels from the moment the user commits it to the moment it rests in the GodHead with baseline, non-AI structure. This entire path must run **deterministically, without AI** — AI is invited only after the file is at rest and only at the user's initiative. Author it gaplessly, in register where it serves.

## 1. Principles Specific to Intake

**1.1 — Intake is deliberate, not reactive.** Files are selected and staged in the client; nothing moves until the user issues an explicit **commit**. There is no filesystem watcher auto-ingesting drops. The first trigger is always a human act.

**1.2 — The whole intake path is deterministic (the floor, standing alone).** Every step from commit to at-rest runs without a **reasoner**. The local embedder — geometry, not judgment (ML pipe §3.1) — is floor machinery, and may run at intake to draw baseline links; if it is unavailable, the file rests linkless and embedding backfills later. Intake never waits on a model and never asks one to think. This is the handleless hammer made literal: it works, it populates the store, it asks nothing of judgment. Intelligence is a later, human-invoked enhancement — never a requirement of intake.

**1.3 — Raw is copied exactly once.** The single duplication of raw content into persistence happens here, and only here.

## 2. The Intake Path

**2.1 — Commit.** The client transmits the staged files to the backend on the user's explicit action. The client's role ends at delivery; it performs no processing.

**2.2 — Raw copy + first log.** Each committed file's raw bytes are copied into persistence exactly once (to disk, by reference — see the persistence pipe). This event writes the first append-only log snapshot for that file. Log fields include at least: filename, filetype, upload date/time, size, and normalized-state (initially false). Logs rotate, never overwrite.

**2.3 — Normalization.** The raw file is reduced to a standardized textual derivative, which becomes the substrate for embedding and agent work. The raw original is always preserved separately and untouched; the derivative never replaces it.

- **Normalization target: clean UTF-8 text.** Every file is decoded to UTF-8 with standardized whitespace and line-endings. Structure is **retained, not stripped** — where a file carries structure (keys, tags, fields), that structure is flattened into the text derivative rather than discarded, because the structure is signal for a knowledge-mapping system, not noise.
- **Text-native and near-text types** (the initial supported set) normalize by direct decode plus light cleanup, fully deterministically.
- **Types that are not directly decode-able** are handled by **degradation**: extracting a textual representation. Trivial degradations (structured-text formats) are deterministic; richer degradations that require interpretation (OCR of scanned material, transcription of audio/video, captioning of images) are the domain of the **Librarian** and are a later expansion — see §3.
- Decode failures or suspect encodings MUST be **logged and flagged**, never silently accepted. A faulty normalization is surfaced, not buried. `[Fable: the per-type decode/cleanup specifics.]`

**2.4 — Supported types (initial).** The first build supports everything that degrades to UTF-8 text **deterministically**: text-native types (e.g. `.txt`, `.json`, `.py`, `.html`, `.md`), near-text structured formats (e.g. CSV/TSV, XML, YAML, TOML, RTF), and **text-layer PDFs** (extractable without OCR). Raw of *any* type is always accepted and stored for future support; when a committed type is not yet processable, the system stores it and surfaces an incompatibility notice rather than rejecting it. Types requiring interpretation to degrade (scanned PDFs, images, audio/video) are deferred to the Librarian. `[Fable: finalize the initial supported set.]`

**2.5 — Deterministic classification (the floor).** The normalized file is assigned baseline categories with starting weights by **hardcoded rules** — structural labeling drawn from filetype and trivial content signals (e.g. a known keyword literally present in text). Structured-data types map to a "database" bucket; source-code types to a "programming" bucket; markup to a "markup" bucket; and so on. These labels are **deliberately low-trust** — placeholders that provide a sane first impression and let the store be immediately legible, explicitly marked as overridable so the later AI layer knows not to over-weight them.

**2.6 — Landing in the GodHead (baseline structure).** The normalized, floor-classified file takes its place in the store as a node. Baseline links may be drawn from cheap, deterministic embedding-similarity geometry, producing an initial map with weak or inert influence (weights carry no force below the coherence threshold). The file is now **at rest**.

## 3. The Librarian (deferred)

The richer-media degradation ladder — everything that requires interpretation rather than decoding to become readable text (OCR, transcription, captioning) — belongs to a standing functionary, the **Librarian**, who renders raw fetched material into studyable text. The Librarian is **named here but not built in the first version**: for the initial supported types there is nothing for him to do, as they decode to UTF-8 trivially. He is the eventual owner of the degradation work, and richer-type support plugs into him when it is built. Paired with the Deacon (who scans returning material at the threshold), the two form the intake gauntlet for anything a Student brings home: scanned, then rendered. `[Fable: the Librarian is deferred — do not build in v1.]`

## 4. The Seam — Where Intake Ends

**4.1 — Automatic processing stops here.** Intake carries a file only as far as *at rest in the GodHead with baseline structure.* All deeper organization — AI classification, weighted aggregation (in the course of which Postulant emergence is recorded as deterministic bookkeeping — Dogma VI.2), audit, commitment — is **human-invoked, or runs only under a standing trigger the user has configured**; it is never triggered by intake itself. This pause is the deliberate boundary between deterministic intake and intelligent processing.

**4.2 — The system recommends across the seam; it does not cross it alone.** Once files are at rest, the system may signal — through legible visual state — that intelligent processing would help. It never forces that processing and never crosses the seam on its own initiative. The recommendation mechanics belong to the ML pipe.

## 5. Standing Charge

Specify the intake path gaplessly and deterministically, commit-to-rest, in register where it serves. Guarantee raw-copied-once, first-log-on-copy, flag-don't-bury on decode failure, and the hard stop at the seam. Leave open only what is marked open.

---

**AMENDMENTS — 2026-07-07 (Phase A authoring, ratified Dogma v0.5).** §1.2: "without a model" → "without a reasoner" — the local embedder is floor machinery and may draw baseline links at intake, backfilling if unavailable (embedder-as-floor ruling). §4.1: Postulant emergence named as deterministic bookkeeping within aggregation; user-configured standing triggers acknowledged alongside direct invocation (standing-consent ruling).
