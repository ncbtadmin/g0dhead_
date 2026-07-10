//! The architectural walls (SLICE_10.md §3; rulings G2/G3/G4/G6).
//!
//! SC-D01 arch half, workspace-wide (G2) · SC-B04 arch half, workspace-wide
//! (G3) · the no-outward-transport wall (G3) · SC-H07 fallback-shape scan
//! (G2) · SC-C07 signature arch-pins (G6) · SC-H06 schema-driven secret
//! sweep (G4).
//!
//! These walls are witnesses against the tree itself: source files, the
//! manifests, and the lockfile are read at test time, so a violation cannot
//! compile its way past the gate — a hand list cannot go stale, because
//! there is no hand list (G2: crates are DISCOVERED by reading `crates/` at
//! runtime). Only the SC-H06 closing sweep touches the store; everything
//! else runs with no database at all.
//!
//! SC-I06's arch half (no API path admits unscanned material) lives with the
//! Section I threshold suite, not here.

mod common;

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Discovery — the anti-hand-list (ruling G2)
// ---------------------------------------------------------------------------

/// The workspace root (two levels above godhead-store's manifest).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

/// Every crate under `crates/` carrying a `src/`, discovered by reading the
/// directory at test time (G2). Discovery is proven non-vacuous against the
/// workspace manifest itself: every `[workspace] members` entry must have
/// been found — a crate added tomorrow is scanned tomorrow, with no test
/// edit, and an empty read_dir is a broken scan, never a clean one.
fn discovered_crate_srcs() -> Vec<(String, PathBuf)> {
    let crates_dir = workspace_root().join("crates");
    let mut found = Vec::new();
    for entry in fs::read_dir(&crates_dir).expect("read crates/") {
        let path = entry.expect("dir entry").path();
        let src = path.join("src");
        if src.is_dir() {
            let name = path
                .file_name()
                .expect("crate dir name")
                .to_string_lossy()
                .into_owned();
            found.push((name, src));
        }
    }
    found.sort();

    // Self-check against the root manifest: discovery covers every member.
    let root_manifest =
        fs::read_to_string(workspace_root().join("Cargo.toml")).expect("root Cargo.toml");
    let member_re = Regex::new(r#""crates/([A-Za-z0-9_-]+)""#).expect("regex");
    let mut members = 0;
    for cap in member_re.captures_iter(&root_manifest) {
        members += 1;
        let member = &cap[1];
        assert!(
            found.iter().any(|(name, _)| name.as_str() == member),
            "workspace member {member} was not discovered under crates/ — the scan is blind"
        );
    }
    assert!(
        members >= 11,
        "only {members} workspace members parsed; the scan's non-vacuity proof is broken"
    );
    found
}

/// Every `.rs` file under a source root, recursively.
fn rs_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read src dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
            }
        }
    }
    files
}

/// A line that is nothing but commentary.
fn is_comment(trimmed: &str) -> bool {
    trimmed.starts_with("//")
}

// ---------------------------------------------------------------------------
// SC-D01 — the threshold-literal scan, workspace-wide (ruling G2)
// ---------------------------------------------------------------------------

/// True when every mention of `token` on this line sits inside a string
/// literal. Segments split on `"` alternate code/string; even indices are
/// code. An escaped quote would skew the parity — none exists on any
/// current mention, and a future misparse fails CLOSED (the assert fires
/// and a human looks), never open.
fn mentions_outside_strings(line: &str, token: &str) -> bool {
    line.split('"').step_by(2).any(|seg| seg.contains(token))
}

/// SC-D01, arch half, widened per ruling G2 (subsumes and strictly widens
/// godhead-audit/tests/d_commitment.rs::sc_d01_config_citation's hand-listed
/// scan, which stays green where it lives — tests only accumulate).
///
/// Law VI.1: the coherence threshold is the sovereign's constant, read
/// fresh and cited by revision; agent code carrying its own number has
/// seceded. Every non-comment mention of the threshold's name in EVERY
/// discovered crate's src is (a) on a config-read line, or (b) the quoted
/// NAME alone — a key-registry arm or a refusal message — never adjacent
/// to an assigned value.
///
/// Honest horizon (G13): the scan is keyed to the literal name, so an
/// unnamed hardcoded number (`let t = 0.01;`) escapes it — that half
/// re-arms at SC-D01's runtime assertion (the citation column is NOT NULL
/// at the substrate; an uncited evaluation refuses regardless of what any
/// agent hardcodes), which runs in d_commitment.rs.
#[test]
fn sc_d01_workspace_scan() {
    // The quoted name immediately bound to a value is a fabrication even
    // inside a literal (e.g. json!({"coherence_threshold": 0.01})).
    let value_shape = Regex::new(r#"coherence_threshold"?\s*[:=]>?\s*-?[0-9.]"#).expect("regex");
    let mut scanned = 0usize;
    for (crate_name, src) in discovered_crate_srcs() {
        for file in rs_files(&src) {
            scanned += 1;
            let text = fs::read_to_string(&file).expect("read source file");
            for (idx, line) in text.lines().enumerate() {
                let trimmed = line.trim();
                if !trimmed.contains("coherence_threshold") || is_comment(trimmed) {
                    continue;
                }
                if trimmed.contains("get_config") {
                    continue; // the lawful shape: a config read
                }
                assert!(
                    !mentions_outside_strings(trimmed, "coherence_threshold"),
                    "{crate_name}/{}:{}: a threshold mention outside a config read \
                     or quoted name (Law VI.1): {trimmed}",
                    file.file_name().unwrap().to_string_lossy(),
                    idx + 1
                );
                assert!(
                    !value_shape.is_match(trimmed),
                    "{crate_name}/{}:{}: the threshold's name bound to a literal value \
                     (Law VI.1): {trimmed}",
                    file.file_name().unwrap().to_string_lossy(),
                    idx + 1
                );
            }
        }
    }
    assert!(
        scanned >= 40,
        "only {scanned} source files scanned — discovery is broken"
    );
}

// ---------------------------------------------------------------------------
// SC-B04 — the IPC scan, workspace-wide (ruling G3)
// ---------------------------------------------------------------------------

/// SC-B04, arch half, widened per ruling G3 (subsumes and strictly widens
/// godhead-store/tests/b_handoff.rs::arch_no_agent_channel's store-only
/// sweep; that test stays green where it lives and keeps godhead-store's
/// sanctioned-module assertion).
///
/// Law III.1: the store interface is the sole inter-agent surface. No crate
/// in the workspace — agent, office, or substrate — touches a socket, an OS
/// channel, an in-process channel, or a child process. The check is raw
/// `contains` over the full text: even a commented-out channel is a channel
/// someone reached for.
///
/// Honest horizon (G13): the needle set is the sanctioned finite list; an
/// IPC primitive not on it (shared memory, named pipes via a new crate)
/// escapes this grep — that half re-arms at the dependency walls below
/// (no_outward_transport_wall forbids the raw-socket crates outright) and
/// at review of any new dependency, which cannot enter Cargo.toml unseen.
#[test]
fn sc_b04_workspace_ipc_scan() {
    let forbidden = [
        "TcpListener",
        "TcpStream",
        "UdpSocket",
        "UnixListener",
        "UnixStream",
        "mpsc::channel",
        "broadcast::channel",
        "watch::channel",
        "std::process::Command",
    ];
    let mut scanned = 0usize;
    for (crate_name, src) in discovered_crate_srcs() {
        for file in rs_files(&src) {
            scanned += 1;
            let text = fs::read_to_string(&file).expect("read source file");
            for needle in forbidden {
                assert!(
                    !text.contains(needle),
                    "{crate_name}/{}: forbidden inter-process primitive '{needle}' \
                     (Law III.1 — agents correspond only through the eternal record)",
                    file.file_name().unwrap().to_string_lossy()
                );
            }
        }
    }
    assert!(
        scanned >= 40,
        "only {scanned} source files scanned — discovery is broken"
    );
}

// ---------------------------------------------------------------------------
// The no-outward-transport wall (ruling G3; SLICE_10 §0)
// ---------------------------------------------------------------------------

/// Dependency names declared in a manifest's `[..dependencies]` sections,
/// including `package = "…"` renames. Line-oriented TOML is sufficient for
/// this workspace's manifests, and a parse failure surfaces as a missing
/// anchor in the non-vacuity check below — fail closed.
fn manifest_dep_names(text: &str) -> Vec<String> {
    let key_re = Regex::new(r"^\s*([A-Za-z0-9_-]+)\s*=").expect("regex");
    let rename_re = Regex::new(r#"package\s*=\s*"([^"]+)""#).expect("regex");
    // Two section shapes carry dependencies. The section form
    // `[dependencies]` / `[dev-dependencies]` / `[target.'cfg(..)'.dependencies]`
    // lists dep names as keys. The per-dependency TABLE form
    // `[dependencies.reqwest]` / `[dev-dependencies.foo]` /
    // `[target.'cfg(..)'.dependencies.bar]` names the crate in the HEADER
    // itself — the segment after the last `dependencies.`. Missing that form
    // was the hole the Slice 11 opening round caught: `[dependencies.interprocess]`
    // trims to `dependencies.interprocess`, which does not end with
    // "dependencies", so the crate name was never captured and the wall went
    // blind to raw-socket/IPC crates declared that way.
    let mut in_deps = false; // a [..dependencies] table: keys are dep names
    let mut in_single = false; // a [..dependencies.<name>] table: a rename inside still counts
    let mut names = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            let section = trimmed.trim_matches(['[', ']']);
            in_deps = section.ends_with("dependencies");
            in_single = false;
            if !in_deps {
                if let Some((_, name)) = section.rsplit_once("dependencies.") {
                    if !name.is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                    {
                        names.push(name.to_string());
                        in_single = true;
                    }
                }
            }
            continue;
        }
        if in_deps {
            if let Some(cap) = key_re.captures(line) {
                names.push(cap[1].to_string());
            }
            if let Some(cap) = rename_re.captures(line) {
                names.push(cap[1].to_string());
            }
        } else if in_single {
            // Only a `package = "…"` rename matters inside a per-dep table;
            // version/features lines are not dependency names.
            if let Some(cap) = rename_re.captures(line) {
                names.push(cap[1].to_string());
            }
        }
    }
    names
}

/// The HTTP wall, mechanical (ruling G3; SLICE_10 §0/§3): no outward
/// transport has ever existed in this workspace, and this test makes that
/// review finding a standing arch fact. The first outward act is Slice 11's,
/// and it cannot ship while this wall stands: the door the Deacon guards is
/// bricked shut until he holds his post.
///
/// THIS WALL IS DELETABLE ONLY BY THE SLICE THAT MAKES DELETION SAFE, IN THE
/// SAME COMMIT THAT MAKES IT SAFE.
///
/// Two halves, each at its strongest true pin (verified against today's
/// lock before pinning — rustls and sqlx pull no HTTP client):
///
///  · MANIFESTS: no crate — root or member — names an HTTP client OR a
///    raw-socket crate as a direct dependency, under any section or rename.
///  · LOCKFILE: the HTTP-client names appear NOWHERE, transitively included
///    — total absence, not merely non-directness.
///
/// Justified exclusions from the LOCK half only: `socket2` and `mio` are in
/// the lock as transitives of tokio/sqlx, where they carry the store's own
/// Postgres connection — the substrate's inward lifeline (the project's
/// explicit live-DB override), not an outward transport any agent code can
/// reach (SC-B04 above proves no crate touches a socket type; the MANIFEST
/// half still forbids naming them directly).
#[test]
fn no_outward_transport_wall() {
    // The ordered list: HTTP clients and client-bearing frameworks.
    let http_clients = [
        "reqwest",
        "hyper",
        "ureq",
        "isahc",
        "curl",
        "attohttpc",
        "surf",
        "awc",
        "http-client",
    ];
    // Today's lock is clean of the wider client substrate too — pinned
    // while true; a slice that lawfully introduces transport rewrites this
    // wall in the commit that makes that safe.
    let http_substrate = [
        "h2",
        "http",
        "http-body",
        "tungstenite",
        "tokio-tungstenite",
        // The other maintained/legacy WebSocket transports — WebSocket is in
        // scope (tungstenite proves it), so the wall must not leave a lawful
        // `ws = "…"` a green path (Slice 11 opening round, minor finding).
        "ws",
        "websocket",
        "soketto",
        "native-tls",
        "openssl",
    ];
    let raw_socket = ["socket2", "mio", "nix", "interprocess"];

    // MANIFEST half: root + every discovered crate manifest.
    let mut manifests = vec![workspace_root().join("Cargo.toml")];
    for (_, src) in discovered_crate_srcs() {
        manifests.push(src.parent().expect("crate dir").join("Cargo.toml"));
    }
    let mut all_direct = Vec::new();
    for manifest in &manifests {
        let text = fs::read_to_string(manifest).expect("read manifest");
        for name in manifest_dep_names(&text) {
            for forbidden in http_clients
                .iter()
                .chain(&http_substrate)
                .chain(&raw_socket)
            {
                assert!(
                    name != *forbidden,
                    "{}: direct dependency on outward-transport crate '{name}'",
                    manifest.display()
                );
            }
            all_direct.push(name);
        }
    }
    assert!(
        all_direct.iter().any(|n| n == "sqlx") && all_direct.len() >= 20,
        "manifest parse found {} dependency names and no sqlx — the parser is blind",
        all_direct.len()
    );

    // LOCKFILE half: total absence, transitives included.
    let lock = fs::read_to_string(workspace_root().join("Cargo.lock")).expect("read Cargo.lock");
    let name_re = Regex::new(r#"(?m)^name = "([^"]+)"$"#).expect("regex");
    let lock_names: Vec<&str> = name_re
        .captures_iter(&lock)
        .map(|c| c.get(1).unwrap().as_str())
        .collect();
    assert!(
        lock_names.len() >= 100 && lock_names.contains(&"tokio"),
        "lock parse found {} packages and no tokio — the parser is blind",
        lock_names.len()
    );
    for name in &lock_names {
        for forbidden in http_clients.iter().chain(&http_substrate) {
            assert!(
                name != forbidden,
                "Cargo.lock carries outward-transport package '{name}' — the wall is breached"
            );
        }
        assert!(
            !name.starts_with("hyper-"),
            "Cargo.lock carries hyper-family package '{name}' — the wall is breached"
        );
    }
}

/// The manifest parser must see Cargo's per-dependency TABLE form, not only
/// the section form — the Slice 11 opening round found `[dependencies.<name>]`
/// slipped the wall (and the lock half does not check the raw-socket list, so
/// the manifest half is the sole guard for IPC crates). Pure-function
/// regression test, no DB.
#[test]
fn manifest_table_form_is_caught() {
    // Section form still works.
    assert!(
        manifest_dep_names("[dependencies]\nreqwest = \"0.12\"\n").contains(&"reqwest".to_string())
    );
    // The three table forms that used to go blind.
    assert!(
        manifest_dep_names("[dependencies.interprocess]\nversion = \"2\"\n")
            .contains(&"interprocess".to_string())
    );
    assert!(
        manifest_dep_names("[dev-dependencies.reqwest]\nversion = \"0.12\"\n")
            .contains(&"reqwest".to_string())
    );
    assert!(
        manifest_dep_names("[target.'cfg(unix)'.dependencies.nix]\nversion = \"0.29\"\n")
            .contains(&"nix".to_string())
    );
    // A rename inside a per-dep table names the REAL crate; version/features
    // lines under it are not dependency names.
    let names =
        manifest_dep_names("[dependencies.myhttp]\npackage = \"reqwest\"\nversion = \"1\"\n");
    assert!(
        names.contains(&"reqwest".to_string()),
        "rename not seen: {names:?}"
    );
    assert!(
        !names.contains(&"version".to_string()),
        "config key mis-captured: {names:?}"
    );
}

// ---------------------------------------------------------------------------
// SC-H07 — the fallback-shape scan, workspace-wide (ruling G2)
// ---------------------------------------------------------------------------

/// SC-H07, arch half (minted by ruling G2): no fallback-shaped extraction —
/// `unwrap_or` / `unwrap_or_else` / `unwrap_or_default` — applied to a
/// config value, anywhere in discovered workspace source. Law II.2 applied
/// to config (A.14): a constant the sovereign never set is a decision
/// nobody made, and a code path that fabricates one has made it.
///
/// HEURISTIC, DOCUMENTED: every `get_config(` occurrence opens a 12-line
/// window (the call line plus the 11 that follow); any non-comment line
/// inside a window carrying `.unwrap_or` is a hit. The window was sized
/// against the tree: every lawful extraction chain completes well inside
/// it, while the checked-fold saturations (`try_fold … unwrap_or(i64::MAX)`
/// — the B1 class, which saturates ARITHMETIC, not a config value) sit 18+
/// raw lines beyond their nearest config read, outside every window.
///
/// Known-clean state: ZERO hits. That includes postgres.rs's
/// bias_skew_threshold read — the class's survivor, fixed this slice to
/// refuse via ok_or_else, a fix this scan now holds in place — and
/// postgres.rs's known_source_ids roster, caught BY this sweep while it was
/// being written and fixed the same day.
///
/// Honest horizon (G13): extraction re-bound across a function boundary or
/// placed more than 11 lines from its read escapes this scan; that half
/// re-arms at SC-H07's runtime half (read-side refusal on absent or
/// mistyped constants, asserted per-key in the criterion suites), and the
/// window size is re-verified whenever this scan moves.
#[test]
fn sc_h07_no_fabricated_defaults() {
    const WINDOW: usize = 12;
    let mut windows = 0usize;
    let mut hits: Vec<String> = Vec::new();
    for (crate_name, src) in discovered_crate_srcs() {
        for file in rs_files(&src) {
            let text = fs::read_to_string(&file).expect("read source file");
            let lines: Vec<&str> = text.lines().collect();
            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if is_comment(trimmed) || !trimmed.contains("get_config(") {
                    continue;
                }
                windows += 1;
                for (offset, follower) in
                    lines[idx..lines.len().min(idx + WINDOW)].iter().enumerate()
                {
                    let follower = follower.trim();
                    if !is_comment(follower) && follower.contains(".unwrap_or") {
                        hits.push(format!(
                            "{crate_name}/{}:{}: fallback-shaped extraction within {offset} \
                             lines of a config read: {follower}",
                            file.file_name().unwrap().to_string_lossy(),
                            idx + offset + 1
                        ));
                    }
                }
            }
        }
    }
    assert!(
        windows >= 10,
        "only {windows} config-read sites found — the scan is blind"
    );
    assert!(
        hits.is_empty(),
        "a fabricated default is a decision the sovereign never made (SC-H07):\n{}",
        hits.join("\n")
    );
}

// ---------------------------------------------------------------------------
// SC-C07 — signature arch-pins (ruling G6)
// ---------------------------------------------------------------------------

/// The parameter list of `fn {name}(…)` in `text`, extracted by paren
/// balancing — robust to line breaks and rustfmt reflows.
fn param_list<'a>(text: &'a str, name: &str) -> &'a str {
    let sig_re = Regex::new(&format!(r"\bfn\s+{name}\s*\(")).expect("regex");
    let m = sig_re
        .find(text)
        .unwrap_or_else(|| panic!("sovereign surface fn {name} not found in interface.rs"));
    let open = m.end() - 1;
    let mut depth = 0usize;
    for (i, ch) in text[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open + 1..open + i];
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced parameter list for fn {name}");
}

/// SC-C07 signature arch-pins (ruling G6): "no sovereign surface accepts a
/// job identity" was, until now, a claim made by argument in review. This
/// test converts claims-by-argument into claims-by-test: the trait
/// declarations in interface.rs are parsed at test time, and every
/// sovereign-act method is pinned to (a) carry a human actor string
/// (`actor: &str` / `changed_by: &str`) and (b) carry NO job-shaped
/// parameter — no name containing `job`, no JobRecord/JobDraft type. IV.4:
/// these acts are the sovereign's; a surface that could take a job identity
/// is a surface an agent could reach for. The runtime GATE_BYPASS_ATTEMPT
/// behavior is each entry's own criterion test, per surface, elsewhere.
#[test]
fn sc_c07_signature_pins() {
    let interface = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interface.rs");
    let text = fs::read_to_string(interface).expect("read interface.rs");
    let sovereign_surfaces = [
        "lay_category_override",
        "resolve_petition",
        "resolve_proposal",
        "consent_decommission",
        "adopt_concordat",
        "set_config",
        "resolve_bias_warning",
        "author_mandate",
        "consent_admission",
    ];
    let actor_re = Regex::new(r"\b(actor|changed_by)\s*:\s*&\s*str\b").expect("regex");
    let job_param_re = Regex::new(r"(?i)\b[a-z0-9_]*job[a-z0-9_]*\s*:").expect("regex");
    for name in sovereign_surfaces {
        let params = param_list(&text, name);
        assert!(
            actor_re.is_match(params),
            "fn {name}: a sovereign act carries a human actor string (IV.4) — \
             params were: {params}"
        );
        assert!(
            !job_param_re.is_match(params),
            "fn {name}: a sovereign surface accepts a job-named parameter (SC-C07) — \
             params were: {params}"
        );
        assert!(
            !params.contains("JobRecord") && !params.contains("JobDraft"),
            "fn {name}: a sovereign surface accepts a job-typed parameter (SC-C07) — \
             params were: {params}"
        );
    }
}

// ---------------------------------------------------------------------------
// SC-H06 — the schema-driven secret sweep (ruling G4)
// ---------------------------------------------------------------------------

/// SC-H06, widened per ruling G4 (Law XV.1): the closing sweep goes
/// schema-driven — no hand list of tables to go stale. Every text /
/// varchar / char / json / jsonb (and text-array) column of every base
/// table in the public schema is enumerated from information_schema at
/// test time, and every value found is run through the PRODUCTION pattern
/// set, `godhead_store::secrets::scan` — one vocabulary for the write-side
/// guard and the sweep, so the sweep can never know less than the guard.
///
/// The live store carries every prior run's residue, so this closes over
/// far more than one pipeline run. On a hit the value is NOT printed — the
/// wall does not keep a diary (G6's register): location, pattern name, and
/// a sha256 prefix only.
///
/// Named `zz_` so it sorts last in this binary: a closing sweep. Non-DB
/// surfaces (intake raw copies under data_root) are not columns and are not
/// swept here — that half stays guarded by the write-path scan
/// (h_commons.rs::secret_scan_blocks_write) and is said here per G13.
#[tokio::test]
async fn zz_sc_h06_schema_driven_sweep() {
    use sha2::{Digest, Sha256};
    use sqlx::Row;

    let Some(store) = common::store().await else {
        return;
    };
    let columns: Vec<(String, String)> = sqlx::query(
        r#"SELECT c.table_name::text, c.column_name::text
           FROM information_schema.columns c
           JOIN information_schema.tables t
             ON t.table_schema = c.table_schema AND t.table_name = c.table_name
           WHERE c.table_schema = 'public'
             AND t.table_type = 'BASE TABLE'
             AND (c.data_type IN ('text', 'character varying', 'character', 'json', 'jsonb')
                  OR (c.data_type = 'ARRAY' AND c.udt_name IN ('_text', '_varchar')))
           ORDER BY c.table_name, c.column_name"#,
    )
    .fetch_all(store.raw_pool())
    .await
    .expect("enumerate text-bearing columns")
    .into_iter()
    .map(|row| (row.get::<String, _>(0), row.get::<String, _>(1)))
    .collect();

    // Non-vacuity: by slice 10 the schema carries the full substrate —
    // an under-count means the enumeration went blind, not that the store
    // went clean.
    let tables: std::collections::BTreeSet<&str> =
        columns.iter().map(|(t, _)| t.as_str()).collect();
    assert!(
        tables.len() >= 10 && columns.len() >= 30,
        "sweep enumerated {} columns across {} tables — the schema-driven census is blind",
        columns.len(),
        tables.len()
    );

    let mut values_scanned = 0usize;
    let mut hits: Vec<String> = Vec::new();
    for (table, column) in &columns {
        let sql = format!(r#"SELECT "{column}"::text FROM "{table}" WHERE "{column}" IS NOT NULL"#);
        let rows = sqlx::query(&sql)
            .fetch_all(store.raw_pool())
            .await
            .unwrap_or_else(|e| panic!("sweep {table}.{column}: {e}"));
        for row in rows {
            let value: String = row.get(0);
            values_scanned += 1;
            if let Some(pattern) = godhead_store::secrets::scan(&value) {
                let digest = Sha256::digest(value.as_bytes());
                hits.push(format!(
                    "{table}.{column}: pattern '{pattern}' (sha256 {:02x}{:02x}{:02x}{:02x}…)",
                    digest[0], digest[1], digest[2], digest[3]
                ));
            }
        }
    }
    assert!(
        values_scanned > 0,
        "the sweep scanned zero values — the census is blind"
    );
    assert!(
        hits.is_empty(),
        "a secret-shaped value is persisted in the eternal record (SC-H06, Law XV.1):\n{}",
        hits.join("\n")
    );
}
