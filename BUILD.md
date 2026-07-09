# g0dhead_ — Build & Verification

## Toolchain

- **Rust** (stable, via [rustup](https://rustup.rs)) — `cargo`, `clippy`, `rustfmt` all required.
  On Windows the default MSVC target needs the Visual Studio Build Tools; rustup's installer offers to fetch them.
- **PostgreSQL** — Railway-hosted for now, reached via `DATABASE_URL`. Provision the database pgvector-capable (the ML slice will need `CREATE EXTENSION vector`).

## Configuration

Copy `.env.example` to `.env` (untracked) and set `DATABASE_URL`. Tests that need the
database skip loudly when it is unset — a run without `DATABASE_URL` is not a full gate pass.

## The verification gate (Phase B, doc 00 §4)

Run from the repo root; all three must pass, warnings fixed, not shipped:

```
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

## Layout

```
Cargo.toml                  workspace root
crates/godhead-schemas/     Appendix A record types + validation
crates/godhead-store/       the Store trait + Postgres substrate + migrations
docs/                       the spec (docs 00–08) — canonical, read 00 first
docs/dev/SLICE_NN.md        pinned slice specs + delivery ledgers, one per slice — the highest N is current
```
