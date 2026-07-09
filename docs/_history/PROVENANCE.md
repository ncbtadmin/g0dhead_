# docs/_history — Provenance Manifest

The original Phase A spec-development conversation is preserved in the working tree
at `docs/_history/original spec development chat/` and is **deliberately untracked**
(sovereign decision S5, 2026-07-09; the directory is named in `.gitignore`): it
carries personal and operational strings, and the repository has a public remote.
The content stays on the operator's machine; it never rides a clone or a push.

Citations into that corpus stay verifiable without the content: the byte count and
SHA-256 of each file are recorded here. A future reader holding the directory can
prove it is the corpus the project's reports cite with one command per file —
PowerShell: `Get-FileHash -Algorithm SHA256 <file>` · POSIX: `sha256sum <file>`.

| file | bytes | sha256 |
|---|---|---|
| `g0dhead_context_export.md` | 30089 | `7ca054f0faf08fe24638ca0aafa689c07cee8353a87f5ea8d91cba23dbf8cb7c` |
| `g0dhead_conversation_journal.txt` | 797 | `d57e9fdbc0217a76e2cae210991bd3a3e6506119076f36a39c4d10ef217ff464` |
| `g0dhead_conversation_raw.txt` | 672943 | `b56ac954eedb3053130519ea558c872e4b315297d058cfa490334ab13567fbed` |
| `g0dhead_conversation_readable.md` | 494295 | `43a06cfb3c315572e59b0ec7f98b82d02528081dbd02068b916e5c61c9801ffa` |

*(Hashes computed 2026-07-09 against the working copies then present. If any file is
ever intentionally revised, this manifest is revised in the same commit — a hash
mismatch with no matching commit is an incident, and incidents are recorded, never
smoothed over. The three tracked briefs in this directory — `05_central_dogma.md`,
`06_holy_standard.md`, `07_student_handbook.md` — are witnessed by git itself and
need no entry here.)*
