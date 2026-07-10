# ADR-3 — Authority Substrate: Formats, Module Layout, and D7 Record Shapes

**Status:** ACCEPTED — desk-reviewed record; non-canonical.

- **Decision:** D7, returned 2026-07-10 as “ADOPT — authority classes,
  registries, envelopes, receipts, executors as amended; the R17a/b
  pairing-authority pricing travels with its phase”
  (ROADMAP_RECONCILIATION.md §15.1).
- **Normative mechanism source:** AUTHORITY_REGISTRIES.md. This ADR chooses
  the concrete formats, dependency layout, application records, and D7 record
  shapes. If it conflicts with that annex, the annex governs until both are
  amended together.
- **Desk record:** DESK_ADR3_ROUND2_CONSOLIDATED, 2026-07-10. It supersedes
  DESK_REVIEW_ADR3 wherever the two touch the same subject. Section 11
  preserves the final dispositions and the desk’s corrections.
- **Rows consumed:** the seven D7 rows in AMENDMENT_MATRIX.md. Section 10 is
  the row-by-row consumption record.
- **Residence:** client, session, token-family, bootstrap, local-recovery, and
  authentication-detail records remain application schema under the
  dev-register. Command, notice, warning, and judgment shapes follow the
  amendment rows named in §10. This ADR does not itself amend canon or create
  a migration.
- **Phase ownership:** P2A delivers §§2–5 and §7, the R19/R20 checkpoint surface,
  and the registered-and-refusing inventory. P2B delivers the notice and bias
  surfaces in §6 and the served executors. R23 lands at P5. A05 is P2A RCPT
  work because its database/filesystem transition is resumable.

---

## 1. Context and decision boundary

P2A must make authentication, authority, and refusal structural before P2B
adds the broad application façade. The substrate therefore has to do all of
the following without creating a domain dependency cycle:

1. authenticate bootstrap, enrollment, sessions, revocation, rotation, and
   host-local recovery;
2. mint unforgeable, operation-bound authority contexts;
3. keep every reserved operation behind a typed public wrapper;
4. enumerate every reserved, authentication, and executor operation in one
   versioned inventory;
5. bind idempotency, expected state, freshness, and reviewed content to a
   canonical request digest;
6. give every authenticated logical reserved command one durable identity;
7. refuse unshipped, stale, replayed, or mismatched work without fabricating
   a JobRecord; and
8. resume R19 and A05 safely across concurrency and process failure.

The record below is a design pin, not executable DDL. Owning migrations must
implement every named CHECK, unique index, foreign key, trigger, writer gate,
and crash test. A comment is not a substitute for an enforcement mechanism.

## 2. Credential and digest contract

### 2.1 Canonical token grammar

Every v1 bearer or local credential contains exactly 32 bytes from an
operating-system CSPRNG. The prefix and any future checksum are outside that
payload and consume none of its 256 random bits. The payload is canonical
unpadded base64url:

- alphabet: ASCII A–Z, a–z, 0–9, hyphen, underscore;
- encoded length: exactly 43 characters for 32 decoded bytes;
- padding: the equals character is forbidden on presentation;
- decoding: reject invalid alphabet, invalid length, non-canonical encodings,
  and decoded lengths other than 32 before any database lookup; and
- rendering: re-encoding the decoded bytes must reproduce the presented
  43-character payload exactly.

| Credential class | Exact v1 prefix | Full length | Domain context |
|---|---:|---:|---|
| bootstrap | <code>gdh_boot_v1_</code> | 55 | <code>gdh:boot:v1</code> |
| access session | <code>gdh_sess_v1_</code> | 55 | <code>gdh:sess:v1</code> |
| refresh token | <code>gdh_refresh_v1_</code> | 58 | <code>gdh:refresh:v1</code> |
| client credential | <code>gdh_client_v1_</code> | 57 | <code>gdh:client:v1</code> |
| recovery credential | <code>gdh_recovery_v1_</code> | 59 | <code>gdh:recovery:v1</code> |

The prefixes are versioned detection and routing aids. They are not authority,
proof of validity, or a substitute for persisted-state checks.

The exact v1 scanner expression is:

~~~text
gdh_(?:boot|sess|refresh|client|recovery)_v1_[A-Za-z0-9_-]{43}
~~~

Scanner integrations must apply token boundaries so a longer base64url run is
not accepted as a credential. CI fixtures use the exact grammar but are
structurally valid fakes whose decoded bytes are fixed public test vectors and
whose digests can never be seeded in a live registry.

### 2.2 Hashing and comparison

For a decoded 32-byte secret S of credential class C, the stored verifier is:

~~~text
SHA-256(ASCII("gdh:" || C || ":v1") || 0x00 || S)
~~~

The separator byte makes the concatenation unambiguous. Each verifier column
is PostgreSQL <code>bytea</code> with <code>octet_length(...) = 32</code>;
it is never unconstrained text. The relevant class table enforces verifier
uniqueness, so an RNG collision or duplicate insertion fails closed.

SHA-256 is selected for machine-generated 256-bit credentials. Argon2id would
add a per-guess cost, but that cost is not proportionate under the 256-bit
entropy invariant, especially on the every-request access-token path. This is
not a claim that a KDF “buys nothing,” and it does not apply if a
human-chosen secret is introduced later.

Application comparison operates only on fixed 32-byte operands and uses the
dependency selected at the P2A gate in §2.5. An indexed PostgreSQL equality
probe is a membership lookup; this ADR does not describe it as constant-time.
No timing guarantee is claimed for the database, network, allocator, or whole
process.

### 2.3 Verification order and revocation

The verifier follows one order for every credential:

1. validate prefix, version, total length, alphabet, padding absence, decoded
   length, and canonical re-encoding;
2. derive the class-separated 32-byte verifier;
3. locate the persisted record;
4. compare fixed-length operands in application code where a candidate is
   available; and
5. check the complete live state for that class before minting a context.

Access acceptance checks the access-session status and revision, token-family
status and revision, and client status and revision on every request.
Refresh acceptance checks the refresh row, its family, and its client.
Credential possession alone never bypasses persisted revocation.

### 2.4 Control-specific rules

- **A01, bootstrap.** First enrollment is a conjunction: it is callable only
  through the recovery-elevation-capable loopback or host-CLI path **and** it
  consumes the unused deployment secret. There is no LAN bootstrap. The
  bootstrap generation, first-client enrollment, application event, and canon
  events commit in one TX after the recovery-elevation preflight has completed.
- **A02, issue and refresh.** Access sessions and refresh credentials have
  separate rows and lifecycles. Refresh consumption is one conditional state
  update; concurrent presentations cannot both mint successors. Presentation
  of a consumed refresh verifier revokes the entire family. Rows required to
  recognize replay are retained.
- **A03, rotate or re-enroll.** A SovereignContext or RecoveryContext is
  bound to A03, its request digest, and the expected client revision. The old
  client credential is revoked in the transaction that creates the new
  generation.
- **A04, revoke.** The command CAS-validates the expected client or access
  session revision. A terminal revocation is idempotent.
- **A05, recover.** Recovery is host-local and single-use. It is RCPT mode,
  not a PostgreSQL-only transaction. Its staged cross-substrate protocol is
  §7.

The annex's "rotated after every use" is literal. For A01 and the
RecoveryContext branch of A03/A04, presenting the protected credential is not
itself a RecoveryContext: it begins the §7.3 preflight, which spends N and
activates N+1 before minting one process-local, operation-bound context. The
command then consumes that context whether its TX completes or refuses. A05's
accepted RCPT owns the same N → N+1 rotation as part of its resumable work. A
TX-elevation verifier failure, lost claim race, or pre-PREPARED staging failure
mints no context and is not a use. For A05, verifier/precondition failure mints
no context; a rolled-back acceptance destroys its ephemeral context but commits
no successful use. Once acceptance commits, later staging failure belongs to
that same use and must resume or fail closed, never abandon. Bootstrap
issuance/reissue uses no RecoveryContext.
SovereignContext executions of A03/A04 do not touch the host recovery
credential.

### 2.5 P2A dependency and leakage gate

No concrete new Rust dependency is chosen by this document. P2A has a
blocking dependency-selection gate that records:

- every direct dependency, exact locked version, enabled feature, and reason;
- the operating-system entropy source on each supported platform;
- MSRV and workspace compatibility;
- current advisory review and license/publish posture;
- Windows and POSIX coverage;
- failure behavior, including entropy-source failure; and
- known-answer and negative test vectors for encoding, hashing, and
  fixed-size equality.

The gate passes only if tests prove:

- entropy failure refuses without deterministic or weak fallback;
- secret wrappers have redacted Debug output and no default serialization;
- alternative encodings, padding, wrong lengths, and wrong prefixes refuse
  before lookup;
- comparison APIs receive fixed-size operands;
- authority crates declare the dependencies directly rather than relying on a
  transitive copy;
- the lockfile evidence is recorded; and
- Windows and POSIX platform evidence is present.

P2A also pins redaction tests for token-like values in Debug output, ordinary
logs, panic messages, HTTP request/response traces, model context, and
provenance payloads. The scanner expression above and a per-prefix fixture
must fail CI if any of those surfaces emits a specimen credential.

## 3. Dependency-safe module layout

### 3.1 Crate DAG

The accepted layout is two crates with the server as the sole composition
root:

~~~text
godhead-server
  ├──> godhead-authority
  │      ├──> godhead-authority-core
  │      └──> domain/store implementation crates
  └──> domain/store implementation crates
             └──> godhead-authority-core

godhead-authority-core ──> no GodHead domain, Store, transport, or server crate
~~~

No edge points from godhead-authority-core upward. Domain and Store crates can
therefore name a context type in a reserved signature while
godhead-authority can call those signatures without forming
authority → domain → authority.

### 3.2 The leaf crate: godhead-authority-core

The leaf owns:

- <code>OperationId::Reserved</code>, <code>OperationId::Auth</code>, and
  <code>OperationId::Executor</code>;
- the K/R/F/H envelope types and canonical request-digest type;
- <code>SovereignContext</code>, <code>OperatorSession</code>,
  <code>StandingAuthority</code>, <code>RecoveryContext</code>, and
  <code>DependencyLossContext</code>;
- private context fields plus the process-local issuer/verifier seal; and
- the versioned serialization forms needed by receipts and historical reads.

The crate has no domain dependency and forbids unsafe code. Contexts expose no
public constructor and implement neither Default nor Deserialize nor an
unrestricted Clone. If a context must move, it is moved or converted into a
more restrictive operation-specific capability; it is not copied into a
second usable authority.

One such restriction is the initial A05 acceptance capability. After the
upper crate verifies ACTIVE N, host-local source, exact operation/K/digest,
expected state, and freshness, core may mint a move-only
<code>A05AcceptanceContext</code>: an operation-specific form of
RecoveryContext, not a sixth authority class. Its only verifier-bearing
consumer is the A05 acceptance wrapper. It cannot apply recovery effects,
resume work, or call any other recovery/control path.

Rust has no friend-crate visibility, so the split does not pretend that a
<code>pub(crate)</code> constructor in the leaf is callable from the upper
crate. Instead, core creates one process-local issuer/verifier pair at the
server composition root. The non-clone issuer is moved into
godhead-authority; verifier handles are moved into the live domain/Store
services. An issuer mints a context with a private issuer seal and an opaque,
non-authorizing issuer_epoch label; a reserved service rejects it unless the
seal matches its own verifier.
Another crate can create an isolated pair through the same public bootstrap
API, but contexts from that pair cannot pass the already-composed service's
verifier. Neither the private issuer seal nor a context is serializable or
valid across process restart. The epoch label may be persisted only to prove
that a READY proof belongs to a dead/different issuer; knowing it cannot mint
or verify anything. It is a fresh OS-random UUID at composition start and is
never reused.

Accepted RCPT work does not depend on preserving that ephemeral object. After
a restart, the authority service may remint a **continuation context of the
original class** only from all of these durable facts: an ACCEPTED receipt,
the identical operation id/request digest/actor and original revision
binding, the next PENDING step, and a newly acquired fenced receipt lease.
The reminted context additionally binds
<code>(receipt_ref, step_no, lease_generation)</code> and can call only that
step’s continuation wrapper. This resumes the already authorized logical
command; it cannot start a new command, change its payload, repeat a completed
step, or widen its authority.

For A05, a receipt-derived RecoveryContext may resume only the staged-file,
activation, and startup-reconciliation steps authorized by that receipt. Once
generation N is SPENT it cannot repeat enrollment/revocation effects or
authorize any other recovery. Such a continuation is the same accepted
logical use, not another presentation/use of the recovery credential, so each
step does not demand another generation. For R19, the equivalent
receipt-derived SovereignContext resumes only the pending promotion step under
its recorded plan/digest. Standing-authority RCPT rows also re-prove their
persisted trigger occurrence before reminting.

Every context binds this tuple:

~~~text
(operation_id,
 request_digest,
 client_or_session_revision,
 elevation_or_freshness_occurrence)
~~~

StandingAuthority uses the exact trigger/consent revision and persisted
freshness occurrence. A TX RecoveryContext additionally uses its READY proof,
activated rotation, issuer epoch, verified recovery generation, and source
channel; an A05 continuation instead uses its receipt/step/lease fence.
DependencyLossContext uses the exact dependency-loss
occurrence minted by a decommission transaction or substrate path. A context
for R15 cannot call R24, and a context for one request digest cannot authorize
another.

### 3.3 The upper crate: godhead-authority

The upper crate owns:

- persisted-state authentication and the only context mints;
- the complete versioned code registry and its database mirror seed;
- A01–A05 control APIs;
- public typed wrappers, one per reserved operation;
- a private erased dispatcher used only after typed authorization; and
- executor claim and resumption gates.

There is no public <code>execute(operation, context, payload)</code>. Such a
function would erase the type wall. Public entry is operation-specific, for
example a wrapper for R15 that accepts only the R15-bound
SovereignContext. Macro generation may reduce repetition, but generated
functions remain typed and independently visible to compile-fail tests.

godhead-server creates exactly one core issuer/verifier pair for a running
composition, gives the issuer only to the private authority service, and gives
verifier handles only to the domain/Store service constructors. The authority
service never returns or lends its issuer. The arch test fixes this wiring;
an integration test proves a correctly shaped context minted by a different
pair is refused by the live service.

The production database pool and its verifier-bearing Store service remain
inside that composition and are not exported by the application façade. A
separately constructed issuer plus a separately constructed Store object is
not accepted as authority over that composed service; the integration test
uses one production-shaped test composition and attacks that same instance,
rather than proving only that two isolated test objects differ.

A01 and A02 have distinct authentication APIs because they create or verify
the authority state that ordinary commands consume. They never share the
reserved-command dispatcher signature. A03, A04, and A05 are authentication
controls with receipt identities and operation-bound contexts.

Raw CANON-Teacher environment creation, raw
CANONICAL_INSTRUCTION pairing, and dependency-loss orphaning remain
non-exported or accept the exact core capability. DependencyLossContext is
minted only by the decommission transaction/substrate path and is consumed
only by <code>orphan_environment</code>; no other public or internal operation
may accept it.

### 3.4 Operation-bound elevation

#### Sovereign elevation

An authenticated OperatorSession is not a SovereignContext. Elevation requires
fresh proof beyond session possession: either re-presentation of the active
client credential or a dedicated approval proof selected by P2A. The proof:

1. is verified against current client and session revisions;
2. binds the stable operation id and canonical request digest;
3. records the checked freshness window and a unique elevation occurrence;
4. is single-use; and
5. mints one operation-bound SovereignContext.

The context is consumed by its typed wrapper. A request cannot choose an actor
class or substitute a different operation after elevation.

#### Recovery elevation for TX authentication controls

The protected recovery credential is verifier material, not a serializable
RecoveryContext. For A01 and the RecoveryContext branch of A03/A04, its only
power is to begin an exact-operation recovery-elevation preflight. The
preflight binds operation, K, canonical digest/version, source channel,
expected state, and freshness; completes N → N+1 across PostgreSQL and the
filesystem; and only then inserts a READY elevation proof and mints one
process-sealed context. The context also binds that proof and issuer epoch.

READY is not accepted command work and the database row is not a bearer. It
cannot be redeemed or reminted from K/digest after process restart. The typed
TX wrapper conditionally consumes READY in the same transaction as the
control's effects, terminal identity, provenance, application event, and canon
events. A stale/precondition refusal consumes it in that refusal transaction.
If the issuer is lost or freshness expires first, READY becomes STRANDED; a
retry must present the then-active credential and rotate again. This preserves
TX semantics without calling a filesystem epilogue part of a completed TX.

A05 is intentionally different. The private verifier result may either drive
its sealed, no-effect precondition-refusal writer or, after all acceptance
preconditions pass, mint one A05AcceptanceContext. That context's sole wrapper
consumes it in the transaction that inserts the ACCEPTED receipt, live A05
rotation, fence, and acceptance events. A rolled-back acceptance transaction
leaves no durable use and the move-only context is gone; a committed one makes
the rotation obligation inescapable. Thereafter the receipt is durable
authority for only its staged rotation/recovery continuation. A05 therefore
remains RCPT and never receives a TX elevation proof. Recovery elevation is
context construction, analogous to sovereign elevation, not another
application operation or an independently callable route.

### 3.5 Stable operation inventory

Persistent identifiers are versioned and never re-used:

~~~text
reserved:R01:v1
reserved:R02a:v1
reserved:R02b:v1
reserved:R03:v1
reserved:R04:v1
reserved:R05:v1
reserved:R06:v1
reserved:R07:v1
reserved:R08:v1
reserved:R09:v1
reserved:R10:v1
reserved:R11:v1
reserved:R12:v1
reserved:R13:v1
reserved:R14:v1
reserved:R15:v1
reserved:R16:v1
reserved:R17a:v1
reserved:R17b:v1
reserved:R18:v1
reserved:R19:v1
reserved:R20:v1
reserved:R21:v1
reserved:R22:v1
reserved:R23:v1
reserved:R24:v1
auth:A01:v1
auth:A02:v1
auth:A03:v1
auth:A04:v1
auth:A05:v1
executor:proposal-execution:v1
executor:decommission-execution:v1
executor:admission-processing:v1
executor:petition-execution:v1
executor:audit-barrier-certification:v1
executor:reconciliation:v1
~~~

R02a/R02b and R17a/R17b are distinct members, so the Reserved namespace
contains the 26 literal annex rows. The six Executor members record authority, claim, fencing, restart
discovery, and phase exactly like the other inventory entries. The four served
executors use their persisted pending work and fenced claim. Barrier
certification and reconciliation ticks bind the exact TrialCycle/reconciliation
state and refuse stale work.

Recovery genesis and pre-enrollment bootstrap issuance/reissue are
deployment-substrate paths, not application operations: they use no
RecoveryContext, have no runtime or network handler, CommandReceipt,
application-auth event, or canon command event, and therefore add no
OperationId member. Their offline installer writers are separately pinned in
§7 and become unreachable once their deployment preconditions close.

R02b’s logical K is its persisted trigger occurrence:
<code>(trigger_revision, window_index)</code>. The P2B shape may add a
first-class occurrence table, but one config revision alone may never identify
an unbounded series of ticks.

### 3.6 Structural evidence

P2A supplies all of the following:

- one pinned inventory test covering 26 Reserved, five Auth, and six Executor
  ids, five context mints, and every sealed primitive;
- a database integration test comparing the code registry with the immutable
  operation-to-class mirror in §5.1;
- compile-fail tests proving context construction, raw reserved calls, and
  wrong-context calls do not compile from a downstream crate;
- compile-fail/integration tests proving A05AcceptanceContext reaches only its
  atomic acceptance wrapper, its private verifier result reaches only refusal
  or that mint, and neither can be serialized, cloned, or used for effects;
- an issuer-mismatch test proving a context from a separately constructed
  core issuer/verifier pair cannot authorize the live composition;
- route-table tests proving A01 and A05 are absent from the LAN listener;
- database tests proving A01/A02/Executor receipt inserts fail the exact
  receipt-eligibility foreign key while all 29 eligible ids match the annex;
- registered-and-refusing end-to-end tests for every unshipped row; and
- a crate-graph test proving godhead-authority-core remains a dependency leaf
  and no domain → godhead-authority edge appears.

## 4. Authentication application records

These tables are application schema under the dev-register, not Appendix A.
All verifier columns are fixed 32-byte <code>bytea</code>. The owning
migration supplies exact status/timestamp arc triggers, revision-CAS writers
where a revision column is shown, locked current-status CAS writers for the
generation/credential rows without one, and no-delete triggers named in §4.2.

### 4.1 Proposed shape

~~~sql
CREATE TABLE app_clients (
    client_id          uuid PRIMARY KEY,
    display_name       text NOT NULL,
    status             text NOT NULL CHECK (status IN ('ACTIVE','REVOKED')),
    revision           bigint NOT NULL DEFAULT 1 CHECK (revision > 0),
    enrolled_at        timestamptz NOT NULL DEFAULT now(),
    revoked_at         timestamptz,
    CHECK (
        (status = 'ACTIVE'  AND revoked_at IS NULL) OR
        (status = 'REVOKED' AND revoked_at IS NOT NULL)
    )
);

CREATE UNIQUE INDEX app_one_active_client_v1
    ON app_clients ((true)) WHERE status = 'ACTIVE';

CREATE TABLE app_client_credentials (
    credential_ref     uuid PRIMARY KEY,
    client_ref         uuid NOT NULL REFERENCES app_clients(client_id),
    generation_no      bigint NOT NULL CHECK (generation_no > 0),
    credential_digest  bytea NOT NULL UNIQUE
        CHECK (octet_length(credential_digest) = 32),
    status             text NOT NULL CHECK (status IN ('ACTIVE','REVOKED')),
    created_at         timestamptz NOT NULL DEFAULT now(),
    revoked_at         timestamptz,
    UNIQUE (client_ref, generation_no),
    CHECK (
        (status = 'ACTIVE'  AND revoked_at IS NULL) OR
        (status = 'REVOKED' AND revoked_at IS NOT NULL)
    )
);

CREATE UNIQUE INDEX app_one_active_credential_per_client
    ON app_client_credentials (client_ref) WHERE status = 'ACTIVE';

CREATE TABLE app_token_families (
    family_id          uuid PRIMARY KEY,
    client_ref         uuid NOT NULL REFERENCES app_clients(client_id),
    status             text NOT NULL CHECK (status IN ('ACTIVE','REVOKED')),
    revision           bigint NOT NULL DEFAULT 1 CHECK (revision > 0),
    revoked_reason     text NULL CHECK (
        revoked_reason IN ('REPLAY','EXPLICIT','CLIENT_REVOKED')
    ),
    created_at         timestamptz NOT NULL DEFAULT now(),
    revoked_at         timestamptz,
    CHECK (
        (status = 'ACTIVE'  AND revoked_reason IS NULL AND revoked_at IS NULL) OR
        (status = 'REVOKED' AND revoked_reason IS NOT NULL AND revoked_at IS NOT NULL)
    )
);

CREATE TABLE app_access_sessions (
    session_id          uuid PRIMARY KEY,
    family_ref          uuid NOT NULL REFERENCES app_token_families(family_id),
    access_digest       bytea NOT NULL UNIQUE
        CHECK (octet_length(access_digest) = 32),
    status              text NOT NULL CHECK (
        status IN ('ACTIVE','REVOKED','EXPIRED')
    ),
    revision            bigint NOT NULL DEFAULT 1 CHECK (revision > 0),
    expires_at          timestamptz NOT NULL,
    ttl_config_revision bigint NOT NULL CHECK (ttl_config_revision > 0),
    created_at          timestamptz NOT NULL DEFAULT now(),
    revoked_at          timestamptz,
    CHECK (
        (status = 'ACTIVE' AND revoked_at IS NULL) OR
        (status = 'REVOKED' AND revoked_at IS NOT NULL) OR
        (status = 'EXPIRED')
    )
);

CREATE INDEX app_access_live_lookup
    ON app_access_sessions (access_digest, expires_at)
    WHERE status = 'ACTIVE';
CREATE INDEX app_access_family_lookup
    ON app_access_sessions (family_ref, status);

CREATE TABLE app_refresh_tokens (
    refresh_ref         uuid PRIMARY KEY,
    family_ref          uuid NOT NULL REFERENCES app_token_families(family_id),
    generation_no       bigint NOT NULL CHECK (generation_no > 0),
    refresh_digest      bytea NOT NULL UNIQUE
        CHECK (octet_length(refresh_digest) = 32),
    status              text NOT NULL CHECK (
        status IN ('ACTIVE','CONSUMED','REVOKED','EXPIRED')
    ),
    revision            bigint NOT NULL DEFAULT 1 CHECK (revision > 0),
    expires_at          timestamptz NOT NULL,
    ttl_config_revision bigint NOT NULL CHECK (ttl_config_revision > 0),
    successor_ref       uuid NULL REFERENCES app_refresh_tokens(refresh_ref)
        DEFERRABLE INITIALLY DEFERRED,
    created_at          timestamptz NOT NULL DEFAULT now(),
    consumed_at         timestamptz,
    revoked_at          timestamptz,
    UNIQUE (family_ref, generation_no),
    CHECK (
        (status = 'ACTIVE' AND consumed_at IS NULL AND revoked_at IS NULL
                           AND successor_ref IS NULL) OR
        (status = 'CONSUMED' AND consumed_at IS NOT NULL AND revoked_at IS NULL
                             AND successor_ref IS NOT NULL) OR
        (status = 'REVOKED' AND revoked_at IS NOT NULL) OR
        (status = 'EXPIRED')
    )
);

CREATE UNIQUE INDEX app_one_active_refresh_per_family
    ON app_refresh_tokens (family_ref) WHERE status = 'ACTIVE';
CREATE UNIQUE INDEX app_refresh_one_predecessor
    ON app_refresh_tokens (successor_ref) WHERE successor_ref IS NOT NULL;
CREATE INDEX app_refresh_live_lookup
    ON app_refresh_tokens (refresh_digest, expires_at)
    WHERE status = 'ACTIVE';

CREATE TABLE app_bootstrap_generations (
    generation_no       bigint PRIMARY KEY CHECK (generation_no > 0),
    secret_digest       bytea NOT NULL UNIQUE
        CHECK (octet_length(secret_digest) = 32),
    status              text NOT NULL CHECK (
        status IN ('UNUSED','CONSUMED','SUPERSEDED')
    ),
    consumed_by_client  uuid NULL REFERENCES app_clients(client_id),
    superseded_by_generation bigint NULL UNIQUE,
    created_at          timestamptz NOT NULL DEFAULT now(),
    consumed_at         timestamptz,
    superseded_at       timestamptz,
    CHECK (
        (status = 'UNUSED' AND consumed_by_client IS NULL
                           AND consumed_at IS NULL
                           AND superseded_by_generation IS NULL
                           AND superseded_at IS NULL) OR
        (status = 'CONSUMED' AND consumed_by_client IS NOT NULL
                             AND consumed_at IS NOT NULL
                             AND superseded_by_generation IS NULL
                             AND superseded_at IS NULL) OR
        (status = 'SUPERSEDED' AND consumed_by_client IS NULL
                               AND consumed_at IS NULL
                               AND superseded_by_generation = generation_no + 1
                               AND superseded_at IS NOT NULL)
    ),
    FOREIGN KEY (superseded_by_generation)
        REFERENCES app_bootstrap_generations(generation_no)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE UNIQUE INDEX app_one_unused_bootstrap
    ON app_bootstrap_generations ((true)) WHERE status = 'UNUSED';

CREATE TABLE app_recovery_generations (
    generation_no       bigint PRIMARY KEY CHECK (generation_no > 0),
    credential_digest   bytea NOT NULL UNIQUE
        CHECK (octet_length(credential_digest) = 32),
    status              text NOT NULL CHECK (
        status IN ('PREPARED','ACTIVE','SPENT','SUPERSEDED')
    ),
    prepared_by_rotation uuid NULL,
    prepared_by_genesis uuid NULL,
    spent_by_rotation  uuid NULL,
    prepared_at         timestamptz,
    activated_at        timestamptz,
    spent_at            timestamptz,
    created_at          timestamptz NOT NULL DEFAULT now(),
    CHECK (
        (prepared_by_rotation IS NULL) <>
        (prepared_by_genesis IS NULL)
    ),
    CHECK (
        (status = 'PREPARED' AND prepared_at IS NOT NULL
                             AND activated_at IS NULL AND spent_at IS NULL) OR
        (status = 'ACTIVE' AND activated_at IS NOT NULL AND spent_at IS NULL) OR
        (status = 'SPENT' AND spent_by_rotation IS NOT NULL
                          AND spent_at IS NOT NULL) OR
        (status = 'SUPERSEDED')
    )
);

CREATE UNIQUE INDEX app_one_active_recovery
    ON app_recovery_generations ((true)) WHERE status = 'ACTIVE';
CREATE UNIQUE INDEX app_one_prepared_recovery
    ON app_recovery_generations ((true)) WHERE status = 'PREPARED';

CREATE TABLE app_recovery_genesis (
    genesis_ref         uuid PRIMARY KEY,
    generation_no       bigint NOT NULL CHECK (generation_no = 1),
    stage_identity      text NOT NULL UNIQUE,
    credential_digest   bytea NULL CHECK (
        credential_digest IS NULL OR octet_length(credential_digest) = 32
    ),
    status              text NOT NULL CHECK (
        status IN ('CLAIMED','PREPARED','ACTIVE','ABANDONED')
    ),
    created_at          timestamptz NOT NULL DEFAULT now(),
    terminal_at         timestamptz NULL,
    CHECK (
        (status = 'CLAIMED' AND terminal_at IS NULL) OR
        (status = 'PREPARED' AND credential_digest IS NOT NULL
                             AND terminal_at IS NULL) OR
        (status = 'ACTIVE' AND credential_digest IS NOT NULL
                           AND terminal_at IS NOT NULL) OR
        (status = 'ABANDONED' AND terminal_at IS NOT NULL)
    )
);

CREATE UNIQUE INDEX app_one_live_recovery_genesis
    ON app_recovery_genesis ((true))
    WHERE status IN ('CLAIMED','PREPARED','ACTIVE');

ALTER TABLE app_recovery_generations
    ADD CONSTRAINT app_recovery_prepared_genesis_fk
    FOREIGN KEY (prepared_by_genesis)
    REFERENCES app_recovery_genesis(genesis_ref);

CREATE TABLE app_recovery_request_bindings (
    binding_ref        uuid PRIMARY KEY,
    operation_id       text NOT NULL CHECK (
        operation_id IN ('auth:A01:v1','auth:A03:v1','auth:A04:v1')
    ),
    idempotency_key    text NOT NULL,
    request_digest     bytea NOT NULL
        CHECK (octet_length(request_digest) = 32),
    canonicalization_version text NOT NULL,
    source_channel     text NOT NULL CHECK (
        source_channel IN ('LOOPBACK','HOST_CLI')
    ),
    expected_state_revision bigint NULL CHECK (
        expected_state_revision IS NULL OR expected_state_revision > 0
    ),
    bootstrap_generation bigint NULL
        REFERENCES app_bootstrap_generations(generation_no),
    freshness_occurrence uuid NOT NULL UNIQUE,
    fresh_until        timestamptz NOT NULL,
    created_at         timestamptz NOT NULL DEFAULT now(),
    CHECK (fresh_until > created_at),
    CHECK (
        (operation_id = 'auth:A01:v1'
         AND bootstrap_generation IS NOT NULL
         AND expected_state_revision IS NULL) OR
        (operation_id IN ('auth:A03:v1','auth:A04:v1')
         AND bootstrap_generation IS NULL
         AND expected_state_revision IS NOT NULL)
    ),
    UNIQUE (operation_id, idempotency_key),
    UNIQUE (
        binding_ref, operation_id, idempotency_key, request_digest,
        canonicalization_version, source_channel
    )
);

CREATE TABLE app_recovery_elevation_refusals (
    refusal_ref        uuid PRIMARY KEY,
    binding_ref        uuid NOT NULL
        REFERENCES app_recovery_request_bindings(binding_ref),
    presented_generation bigint NOT NULL
        REFERENCES app_recovery_generations(generation_no),
    attempted_digest   bytea NOT NULL
        CHECK (octet_length(attempted_digest) = 32),
    attempted_canonicalization_version text NOT NULL,
    source_channel     text NOT NULL CHECK (
        source_channel IN ('LOOPBACK','HOST_CLI')
    ),
    reason_code        text NOT NULL CHECK (
        reason_code = 'ELEVATION_IDEMPOTENCY_MISMATCH'
    ),
    occurred_at        timestamptz NOT NULL DEFAULT now(),
    UNIQUE (
        binding_ref, presented_generation, attempted_digest,
        attempted_canonicalization_version
    )
);

CREATE TABLE app_recovery_rotations (
    rotation_ref       uuid PRIMARY KEY,
    rotation_kind      text NOT NULL CHECK (
        rotation_kind IN ('TX_ELEVATION','A05')
    ),
    operation_id       text NOT NULL CHECK (
        operation_id IN (
            'auth:A01:v1','auth:A03:v1','auth:A04:v1','auth:A05:v1'
        )
    ),
    request_binding_ref uuid NULL,
    idempotency_key    text NOT NULL,
    request_digest     bytea NOT NULL
        CHECK (octet_length(request_digest) = 32),
    canonicalization_version text NOT NULL,
    source_channel     text NOT NULL CHECK (
        source_channel IN ('LOOPBACK','HOST_CLI')
    ),
    issuer_epoch       uuid NULL,
    attempt_no         bigint NOT NULL CHECK (attempt_no > 0),
    prior_generation   bigint NOT NULL
        REFERENCES app_recovery_generations(generation_no),
    next_generation    bigint NOT NULL,
    stage_identity     text NOT NULL UNIQUE,
    receipt_ref        uuid NULL UNIQUE,
    status             text NOT NULL CHECK (
        status IN ('CLAIMED','PREPARED','ACTIVATED','ABANDONED')
    ),
    created_at         timestamptz NOT NULL DEFAULT now(),
    prepared_at        timestamptz NULL,
    activated_at       timestamptz NULL,
    abandoned_at       timestamptz NULL,
    CHECK (next_generation = prior_generation + 1),
    CHECK (
        (rotation_kind = 'TX_ELEVATION'
         AND operation_id IN ('auth:A01:v1','auth:A03:v1','auth:A04:v1')
         AND request_binding_ref IS NOT NULL
         AND issuer_epoch IS NOT NULL
         AND receipt_ref IS NULL) OR
        (rotation_kind = 'A05'
         AND operation_id = 'auth:A05:v1'
         AND request_binding_ref IS NULL
         AND issuer_epoch IS NULL
         AND receipt_ref IS NOT NULL
         AND attempt_no = 1)
    ),
    CHECK (
        (status = 'CLAIMED' AND prepared_at IS NULL
                            AND activated_at IS NULL
                            AND abandoned_at IS NULL) OR
        (status = 'PREPARED' AND prepared_at IS NOT NULL
                             AND activated_at IS NULL
                             AND abandoned_at IS NULL) OR
        (status = 'ACTIVATED' AND prepared_at IS NOT NULL
                              AND activated_at IS NOT NULL
                              AND abandoned_at IS NULL) OR
        (status = 'ABANDONED'
                              AND rotation_kind = 'TX_ELEVATION'
                              AND prepared_at IS NULL
                              AND activated_at IS NULL
                              AND abandoned_at IS NOT NULL)
    ),
    UNIQUE (request_binding_ref, attempt_no),
    FOREIGN KEY (
        request_binding_ref, operation_id, idempotency_key, request_digest,
        canonicalization_version, source_channel
    ) REFERENCES app_recovery_request_bindings (
        binding_ref, operation_id, idempotency_key, request_digest,
        canonicalization_version, source_channel
    )
);

CREATE UNIQUE INDEX app_recovery_one_live_rotation
    ON app_recovery_rotations (prior_generation)
    WHERE status IN ('CLAIMED','PREPARED');

CREATE UNIQUE INDEX app_recovery_one_live_binding_rotation
    ON app_recovery_rotations (request_binding_ref)
    WHERE rotation_kind = 'TX_ELEVATION'
      AND status IN ('CLAIMED','PREPARED');

CREATE TABLE app_recovery_elevation_proofs (
    proof_ref           uuid PRIMARY KEY,
    rotation_ref        uuid NOT NULL UNIQUE
        REFERENCES app_recovery_rotations(rotation_ref),
    request_binding_ref uuid NOT NULL,
    operation_id        text NOT NULL CHECK (
        operation_id IN ('auth:A01:v1','auth:A03:v1','auth:A04:v1')
    ),
    idempotency_key     text NOT NULL,
    request_digest      bytea NOT NULL
        CHECK (octet_length(request_digest) = 32),
    canonicalization_version text NOT NULL,
    source_channel      text NOT NULL CHECK (
        source_channel IN ('LOOPBACK','HOST_CLI')
    ),
    expected_state_revision bigint NULL CHECK (
        expected_state_revision IS NULL OR expected_state_revision > 0
    ),
    bootstrap_generation bigint NULL
        REFERENCES app_bootstrap_generations(generation_no),
    freshness_occurrence uuid NOT NULL,
    fresh_until         timestamptz NOT NULL,
    issuer_epoch        uuid NOT NULL,
    status              text NOT NULL CHECK (
        status IN ('READY','CONSUMED','STRANDED')
    ),
    auth_event_ref      uuid NULL UNIQUE,
    receipt_ref         uuid NULL UNIQUE,
    ready_at            timestamptz NULL,
    consumed_at         timestamptz NULL,
    stranded_at         timestamptz NULL,
    CHECK (
        (operation_id = 'auth:A01:v1'
         AND bootstrap_generation IS NOT NULL
         AND expected_state_revision IS NULL) OR
        (operation_id IN ('auth:A03:v1','auth:A04:v1')
         AND bootstrap_generation IS NULL
         AND expected_state_revision IS NOT NULL)
    ),
    CHECK (
        (status = 'READY' AND ready_at IS NOT NULL
                          AND ready_at < fresh_until
                          AND auth_event_ref IS NULL
                          AND receipt_ref IS NULL
                          AND consumed_at IS NULL
                          AND stranded_at IS NULL) OR
        (status = 'STRANDED' AND auth_event_ref IS NULL
                             AND receipt_ref IS NULL
                             AND consumed_at IS NULL
                             AND stranded_at IS NOT NULL) OR
        (status = 'CONSUMED' AND ready_at IS NOT NULL
                             AND ready_at < fresh_until
                             AND consumed_at IS NOT NULL
                             AND consumed_at <= fresh_until
                             AND stranded_at IS NULL
                             AND (
              (operation_id = 'auth:A01:v1'
               AND auth_event_ref IS NOT NULL AND receipt_ref IS NULL) OR
              (operation_id IN ('auth:A03:v1','auth:A04:v1')
               AND auth_event_ref IS NULL AND receipt_ref IS NOT NULL)
         ))
    ),
    FOREIGN KEY (
        request_binding_ref, operation_id, idempotency_key, request_digest,
        canonicalization_version, source_channel
    ) REFERENCES app_recovery_request_bindings (
        binding_ref, operation_id, idempotency_key, request_digest,
        canonicalization_version, source_channel
    )
);

CREATE UNIQUE INDEX app_recovery_one_ready_binding_proof
    ON app_recovery_elevation_proofs (request_binding_ref)
    WHERE status = 'READY';

CREATE TABLE app_recovery_transitions (
    transition_ref    uuid PRIMARY KEY,
    rotation_ref      uuid NOT NULL UNIQUE
        REFERENCES app_recovery_rotations(rotation_ref),
    from_generation   bigint NOT NULL UNIQUE
        REFERENCES app_recovery_generations(generation_no),
    to_generation     bigint NOT NULL UNIQUE
        REFERENCES app_recovery_generations(generation_no),
    recorded_at       timestamptz NOT NULL DEFAULT now(),
    CHECK (to_generation = from_generation + 1)
);

CREATE TABLE app_auth_events (
    event_id             uuid PRIMARY KEY,
    event_sequence       int NOT NULL CHECK (event_sequence > 0),
    prior_event_ref      uuid NULL REFERENCES app_auth_events(event_id)
        DEFERRABLE INITIALLY DEFERRED,
    operation_id         text NOT NULL,
    canonicalization_version text NOT NULL,
    control_id           text NOT NULL CHECK (
        control_id IN ('A01','A02','A03','A04','A05')
    ),
    outcome              text NOT NULL,
    reason_code          text NULL,
    subject_kind         text NOT NULL CHECK (
        subject_kind IN (
            'CLIENT','TOKEN_FAMILY','ACCESS_SESSION','REFRESH_TOKEN',
            'BOOTSTRAP_GENERATION','RECOVERY_GENERATION'
        )
    ),
    subject_uuid         uuid NULL,
    subject_generation   bigint NULL CHECK (
        subject_generation IS NULL OR subject_generation > 0
    ),
    state_revision       bigint NULL CHECK (
        state_revision IS NULL OR state_revision > 0
    ),
    actor_kind           text NOT NULL CHECK (
        actor_kind IN (
            'CLIENT','SESSION','RECOVERY','STANDING',
            'RECOGNIZED_CREDENTIAL'
        )
    ),
    actor_ref            text NOT NULL,
    source_channel       text NOT NULL CHECK (
        source_channel IN ('LOOPBACK','HOST_CLI','LAN','INTERNAL')
    ),
    idempotency_key      text NOT NULL,
    request_digest       bytea NOT NULL CHECK (octet_length(request_digest) = 32),
    receipt_ref          uuid NULL,
    occurred_at          timestamptz NOT NULL DEFAULT now(),
    CHECK (
        operation_id = 'auth:' || control_id || ':v1'
    ),
    CHECK (
        (subject_kind IN ('BOOTSTRAP_GENERATION','RECOVERY_GENERATION')
         AND subject_uuid IS NULL AND subject_generation IS NOT NULL) OR
        (subject_kind NOT IN ('BOOTSTRAP_GENERATION','RECOVERY_GENERATION')
         AND subject_uuid IS NOT NULL AND subject_generation IS NULL)
    ),
    CHECK (
        (control_id = 'A01' AND outcome IN ('ENROLLED','REFUSED')) OR
        (control_id = 'A02' AND outcome IN (
            'ISSUED','REFRESHED','FAMILY_REVOKED_REPLAY','REFUSED'
        )) OR
        (control_id = 'A03' AND outcome IN ('ROTATED','REENROLLED','REFUSED')) OR
        (control_id = 'A04' AND outcome IN (
            'SESSION_REVOKED','CLIENT_REVOKED','IDEMPOTENT_TERMINAL','REFUSED'
        )) OR
        (control_id = 'A05' AND outcome IN (
            'RECOVERY_ACCEPTED','RECOVERY_COMPLETED','RECOVERY_REFUSED',
            'RECOVERY_RECONCILED'
        ))
    ),
    CHECK (
        (outcome LIKE '%REFUSED' AND reason_code IS NOT NULL) OR
        (outcome NOT LIKE '%REFUSED')
    ),
    CHECK (
        (outcome = 'FAMILY_REVOKED_REPLAY'
         AND actor_kind = 'RECOGNIZED_CREDENTIAL') OR
        (outcome <> 'FAMILY_REVOKED_REPLAY'
         AND actor_kind <> 'RECOGNIZED_CREDENTIAL')
    ),
    CHECK (
        (control_id = 'A01'
         AND source_channel IN ('LOOPBACK','HOST_CLI')) OR
        (control_id = 'A05' AND outcome <> 'RECOVERY_RECONCILED'
         AND source_channel IN ('LOOPBACK','HOST_CLI')) OR
        (control_id = 'A05' AND outcome = 'RECOVERY_RECONCILED'
         AND source_channel = 'INTERNAL') OR
        control_id NOT IN ('A01','A05')
    ),
    CHECK (
        (control_id IN ('A03','A04','A05') AND receipt_ref IS NOT NULL) OR
        (control_id IN ('A01','A02') AND receipt_ref IS NULL)
    ),
    CHECK (
        (event_sequence = 1 AND prior_event_ref IS NULL) OR
        (event_sequence > 1 AND prior_event_ref IS NOT NULL)
    ),
    UNIQUE (control_id, idempotency_key, event_sequence)
);

CREATE UNIQUE INDEX app_auth_one_root_per_key
    ON app_auth_events (control_id, idempotency_key)
    WHERE event_sequence = 1;

CREATE UNIQUE INDEX app_auth_single_event_a01_a04
    ON app_auth_events (control_id, idempotency_key)
    WHERE control_id IN ('A01','A02','A03','A04');
~~~

The command-receipt foreign keys on app_auth_events.receipt_ref,
app_recovery_rotations.receipt_ref, and
app_recovery_elevation_proofs.receipt_ref, plus the proof's auth-event foreign
key, are added after their target tables exist. The operation-id foreign keys
are added after the registry mirror exists.

The immutable recovery request binding is elevation/authentication state, not
a CommandReceipt and not command acceptance. Before an A03/A04 TX receipt
exists, a same-(operation,K)/different-digest presentation writes the typed
app_recovery_elevation_refusal and mints no context; it is not the Q7
CommandRefusal that requires a standing receipt. Once the command TX has
created its terminal receipt, ordinary Q7 receipt/refusal rules govern every
retry. Treating READY as accepted work would silently turn TX into RCPT, so
this boundary is structural rather than terminological.

The rotation/proof triggers verify exact operation, K, digest,
canonicalization, source, expected state, freshness, generation, issuer epoch,
and authority class. A01 proof consumption requires its exact terminal
app-auth event; A03/A04 require their exact terminal RECOVERY-class receipt;
A05 activation requires its exact receipt and completion event. The command
anchor and READY → CONSUMED transition occur in the same TX.

### 4.2 Required writers and triggers

The owning migration must provide:

- no-delete triggers on clients, credential generations, families, access
  sessions, refresh rows, bootstrap/recovery generations, recovery genesis,
  request bindings, elevation refusals, rotations, elevation proofs,
  transitions, and auth events;
- append-only triggers on recovery request bindings, elevation refusals, and
  transitions; their insert-bound identity, digest, version, source, expected
  state, freshness, generation, and created-at fields never change;
- an append-only trigger on app_auth_events: no UPDATE and no DELETE;
- an auth-event lineage trigger: A01–A04 have exactly the one root event;
  A05 begins with RECOVERY_ACCEPTED or a terminal RECOVERY_REFUSED, each later
  row names the immediately prior sequence under the same versioned operation/
  canonicalization/control/key/digest/receipt, zero or more
  RECOVERY_RECONCILED rows may follow acceptance, one RECOVERY_COMPLETED or
  RECOVERY_REFUSED terminates the lineage, and no row follows a terminal event;
- table-specific arc-and-coherence triggers: revision-bearing app_clients and
  app_token_families allow ACTIVE → REVOKED only; app_access_sessions allows
  ACTIVE → REVOKED|EXPIRED only; app_refresh_tokens allows
  ACTIVE → CONSUMED|REVOKED|EXPIRED only; each successful arc advances
  revision by exactly +1. Non-revision app_client_credentials uses a locked
  ACTIVE → REVOKED status CAS; app_bootstrap_generations uses locked
  UNUSED → CONSUMED|SUPERSEDED, with the exact consuming client or adjacent
  superseding generation and timestamp fixed in that transition;
  app_recovery_generations uses locked
  PREPARED → ACTIVE and ACTIVE → SPENT|SUPERSEDED. Every listed target other
  than ACTIVE is terminal, state-specific witness/timestamp fields are set
  once, and no trigger permits resurrection by clearing those fields;
- app_recovery_genesis allows CLAIMED → PREPARED|ABANDONED and
  PREPARED → ACTIVE only; ACTIVE and ABANDONED are terminal. Its deployment
  writer is unreachable after any recovery generation exists, and CLAIMED
  insertion locks/rechecks that both the generation table and live-genesis
  index are empty;
- a TX_ELEVATION rotation arc of CLAIMED → PREPARED|ABANDONED and
  PREPARED → ACTIVATED only; an accepted A05 rotation permits only
  CLAIMED → PREPARED → ACTIVATED. PREPARED and every terminal state never
  retreat. ABANDONED is legal only for TX_ELEVATION before N is SPENT and
  retains immutable evidence while releasing the live-rotation index;
- rotation identity fields and created_at are immutable. Only the legal
  status/timestamp fields change, and an A05 receipt_ref is fixed at insert;
- TX_ELEVATION insertion locks/rechecks ACTIVE N, matches the immutable
  request binding, assigns the next attempt number under that binding, and
  verifies the host-local credential without minting a RecoveryContext. A05
  insertion instead matches its ACCEPTED receipt. Either insertion refuses if
  another live rotation owns N or a transition already spends it; TX insertion
  also refuses while the same binding has a live rotation or READY proof;
- the private A05 verifier result is non-serializable and reaches only the
  no-effect terminal-refusal writer or the A05AcceptanceContext mint. The
  move-only context reaches only the atomic acceptance writer; that writer
  consumes it and must insert the ACCEPTED receipt and CLAIMED rotation
  together or roll back both;
- an ACCEPTED A05 rotation is also the persisted auth-state fence: A01–A04,
  A02 issuance/refresh, and every client/session/token-family writer lock and
  recheck that no such fence covers their target. Acceptance locks the exact
  target revisions and installs the fence atomically, so ordinary concurrent
  mutation cannot make its later effects transaction stale;
- PREPARED requires N SPENT, N+1 PREPARED, and the immutable adjacent
  transition, all naming the same rotation. ACTIVATED requires N+1 ACTIVE and
  the writer's verified live-file digest. The same activation transaction
  inserts READY for a still-live/fresh TX issuer, inserts STRANDED otherwise,
  or completes the A05 receipt/events;
- an elevation proof is inserted as READY only for the still-live/fresh issuer,
  otherwise directly as STRANDED; the only legal update is
  READY → CONSUMED|STRANDED. Binding,
  rotation, issuer epoch, expected state, freshness, and ready_at are fixed.
  The row alone cannot remint a context. CONSUMED is one conditional update in
  the command TX and requires the exact terminal event/receipt; STRANDED has
  no command anchor and is required on issuer loss, restart, or freshness
  expiry;
- a same-(operation,K)/different-digest elevation attempt appends an immutable
  elevation refusal against the standing binding before context mint. Its
  insert trigger requires a verified host-local presentation, resolves the
  exact standing operation/K binding, proves presented_generation is the
  matching ACTIVE verifier, proves the attempted digest or version differs,
  and records the attempted source; an identical request cannot mint this
  refusal. A later attempt for the same K/digest may follow an ABANDONED
  rotation or STRANDED proof, but gets a strictly increasing attempt_no and
  must use the then-ACTIVE credential;
- an immutable one-row-per-successful-RecoveryContext-use transition: its
  rotation, unique from/to generations, and +1 adjacency must agree; a deferred
  trigger requires from_generation SPENT and to_generation PREPARED by that
  same rotation before the prepare transaction commits;
- one transaction for family replay revocation covering the family and every
  live access/refresh child;
- refresh consumption as a conditional
  <code>ACTIVE → CONSUMED</code> update that returns exactly one winner;
- a deferred refresh-lineage trigger requiring a CONSUMED row’s successor to
  belong to the same family at exactly generation_no +1, with the conditional
  ACTIVE → CONSUMED winner as the sole path permitted to insert that
  successor; app_refresh_one_predecessor prevents two parents from naming it;
- one-active-client, one-active-credential, one-active-refresh,
  one-unused-bootstrap, one-active-recovery, and one-prepared-recovery
  enforcement through the partial indexes above; and
- lookup plans proving the live token paths use the named indexes.

app_auth_events records only attributable post-authentication activity, with
one narrow A02 security case: a presented digest that matches a known
CONSUMED refresh row is a recognized-credential replay. It mints no context,
but atomically revokes the family and writes FAMILY_REVOKED_REPLAY attributed
to actor_kind RECOGNIZED_CREDENTIAL and that refresh/family identity. A random,
unknown, malformed, or otherwise unrecognized unauthenticated request refuses
before actor elevation and before persistence, so the table never invents a
human actor for a pre-authentication attempt.

Any future P7 retention/destruction rule must retain every consumed refresh
row needed to recognize replay for a still-live family. Destruction remains
subject to the Duty of the House and A.14(c); it is not inferred from an
EXPIRED or CONSUMED status.

### 4.3 Authority-map and lost-response semantics

A01, A03, A04, and A05 change who may exercise authority. Successful controls
emit canon COMMAND_ACCEPTED and COMMAND_COMPLETED with their stable
operation_id. A02 only mints ephemeral bearer state and changes no authority
holder; under the recorded bounded reading of Law V.1, its detail and event
identity stay application-side. All five controls still write their
post-authentication application event.

A01 and A02 use app_auth_events as their durable identity anchors. A03, A04,
and A05 use command_receipts and correlate their application event to it.
This resolves the phrase “uniform command identity” without forcing
pre-context authentication through the reserved dispatcher.

For A02, and for A01 once its TX has run, same key and same request digest
discovers the single root event. Before an A01 root exists, the recovery
request binding governs same-K retry/mismatch exactly as §4.1 states. A
different digest never inserts a contradictory second root. For A05,
event_sequence and prior_event_ref form the enforced
accepted/reconciled/terminal lineage while the CommandReceipt remains the
logical command anchor.

Secret delivery is intentionally not replayable:

- retrying A01 returns its terminal app-auth event and no first-client
  plaintext credential; if the enrollment response was lost after bootstrap
  consumption, the newly rotated ACTIVE N+1 host recovery credential may begin
  A05;
- retrying an A02 idempotency key returns the terminal app-auth event and no
  plaintext token; after a lost successful response a **still-valid enrolled
  client proof** may submit a new A02 issue key and obtain a fresh pair. A new
  key never revives or reuses the consumed refresh credential;
- retrying an A03 key returns the terminal command receipt and no plaintext
  credential; after a lost successful response the sole supported recovery
  is A05; and
- retrying A05 returns its receipt state and never replays a replacement
  client credential. If a crash/lost response destroys that plaintext after
  the replacement was enrolled, startup finishes the accepted transition,
  then the operator uses the now-ACTIVE N+1 host recovery credential to run a
  new A05 under a new key and obtain a new replacement credential;
- no event or receipt stores encrypted or recoverable bearer plaintext.

Tests cover both lost-response branches. “Retry-convergent” describes durable
state and identity, not re-delivery of a secret the server deliberately did
not retain.

## 5. Command identity, receipt, refusal, step, and outbox substrate

### 5.1 Operation-to-class mirror

One heterogeneous receipt table cannot use a one-class-per-table G10 trigger.
The substrate therefore carries an immutable database mirror of the code
registry:

~~~sql
CREATE TABLE authority_operation_registry (
    operation_id             text PRIMARY KEY,
    namespace                text NOT NULL CHECK (
        namespace IN ('RESERVED','AUTH','EXECUTOR')
    ),
    authority_requirement    text NOT NULL,
    completion_mode          text NOT NULL CHECK (
        completion_mode IN ('TX','RCPT','DISPATCH','EXECUTOR')
    ),
    phase                    text NOT NULL,
    requires_k               boolean NOT NULL,
    requires_r               boolean NOT NULL,
    requires_f               boolean NOT NULL,
    requires_h               boolean NOT NULL,
    canonicalization_version text NOT NULL,
    registry_version         int NOT NULL CHECK (registry_version > 0),
    acceptance_contract      text NOT NULL,
    completion_contract      text NOT NULL,
    UNIQUE (namespace, operation_id),
    UNIQUE (operation_id, canonicalization_version),
    UNIQUE (operation_id, completion_mode, canonicalization_version)
);

CREATE TABLE authority_operation_classes (
    operation_id       text NOT NULL
        REFERENCES authority_operation_registry(operation_id),
    authority_class    text NOT NULL CHECK (
        authority_class IN (
            'SOVEREIGN','OPERATOR_SESSION','STANDING',
            'RECOVERY','DEPENDENCY_LOSS'
        )
    ),
    PRIMARY KEY (operation_id, authority_class)
);

CREATE TABLE authority_receipt_operations (
    operation_id       text PRIMARY KEY
        REFERENCES authority_operation_registry(operation_id),
    CHECK (
        operation_id LIKE 'reserved:%' OR
        operation_id IN ('auth:A03:v1','auth:A04:v1','auth:A05:v1')
    )
);

CREATE TABLE authority_executor_routes (
    command_operation_id  text NOT NULL
        REFERENCES authority_operation_registry(operation_id),
    work_kind             text NOT NULL,
    executor_id           text NOT NULL
        REFERENCES authority_operation_registry(operation_id),
    PRIMARY KEY (command_operation_id, work_kind, executor_id)
);
~~~

The registry is seeded with all 37 operation ids at P2A. The receipt-eligible
relation is seeded with exactly the 26 Reserved ids plus A03/A04/A05: 29 rows,
never A01, A02, or an Executor id. The registry, allowed-class relation,
receipt-eligible relation, and executor routes are immutable and no-delete at
runtime; only the migration owner may seed a newly pinned registry version.
The code/DB arch test compares every metadata field, every allowed-class set,
the exact receipt-eligible set, and every executor route.

The receipt writer first joins operation_id to authority_receipt_operations,
then joins
<code>(NEW.operation_id, NEW.authority_class)</code> to the normalized
allowed-class relation, checks NEW.completion_mode, checks mode-sensitive
envelope nullability, and compares the transaction-local actor class with the
selected allowed class. A03 and A04 each have SOVEREIGN and RECOVERY rows.
Direct SQL cannot bless an invented operation or class merely by spelling it
in a receipt.

A01 has a RECOVERY allowed-class row; its deployment secret remains a required
second factor, not a second actor class. A02 has no context-class row because
its authority_requirement is the pre-context
<code>ENROLLED_CLIENT_OR_REFRESH_PROOF</code>. It uses its distinct Auth API
and never enters the receipt trigger. Both A01 and A02 retain the annex’s TX
mode: each commits its application identity, state effects, and applicable
canon events in one authentication transaction, without manufacturing a
CommandReceipt. Executor-route writes additionally prove
that executor_id names an EXECUTOR-namespace row and that the
command-operation/work-kind/executor triple is present in
authority_executor_routes.

### 5.2 CommandReceipt and CommandRefusal

One CommandReceipt is minted or discovered for each authenticated logical
reserved command: all R rows and A03/A04/A05. A01 and A02 are excluded as
stated in §4.3.

~~~sql
CREATE TABLE command_receipts (
    receipt_ref              uuid PRIMARY KEY,
    operation_id             text NOT NULL
        REFERENCES authority_receipt_operations(operation_id),
    idempotency_key          text NOT NULL UNIQUE,
    request_digest           bytea NOT NULL
        CHECK (octet_length(request_digest) = 32),
    canonicalization_version text NOT NULL,
    authority_class          text NOT NULL,
    completion_mode          text NOT NULL CHECK (
        completion_mode IN ('TX','RCPT','DISPATCH')
    ),
    actor_ref                text NOT NULL,
    context_binding          jsonb NOT NULL,
    expected_state           jsonb NULL,
    freshness_fact           jsonb NULL,
    content_hash             bytea NULL
        CHECK (content_hash IS NULL OR octet_length(content_hash) = 32),
    work_manifest_digest     bytea NULL CHECK (
        work_manifest_digest IS NULL OR
        octet_length(work_manifest_digest) = 32
    ),
    work_manifest_sealed_at  timestamptz NULL,
    status                   text NOT NULL CHECK (
        status IN ('ACCEPTED','COMPLETED','REFUSED')
    ),
    terminal_outcome         jsonb NULL,
    created_at               timestamptz NOT NULL DEFAULT now(),
    terminal_at              timestamptz NULL,
    CHECK (
        (status = 'ACCEPTED'
         AND completion_mode = 'RCPT'
         AND terminal_outcome IS NULL AND terminal_at IS NULL) OR
        (status IN ('COMPLETED','REFUSED')
         AND terminal_outcome IS NOT NULL AND terminal_at IS NOT NULL)
    ),
    UNIQUE (receipt_ref, operation_id, idempotency_key),
    FOREIGN KEY (operation_id, completion_mode, canonicalization_version)
        REFERENCES authority_operation_registry(
            operation_id, completion_mode, canonicalization_version
        ),
    FOREIGN KEY (operation_id, authority_class)
        REFERENCES authority_operation_classes(operation_id, authority_class)
);

CREATE TABLE command_refusals (
    refusal_ref       uuid PRIMARY KEY,
    refusal_identity  bytea NOT NULL UNIQUE
        CHECK (octet_length(refusal_identity) = 32),
    operation_id      text NOT NULL
        REFERENCES authority_receipt_operations(operation_id),
    request_ref       uuid NULL REFERENCES command_receipts(receipt_ref),
    idempotency_key   text NULL,
    attempted_digest  bytea NULL
        CHECK (attempted_digest IS NULL OR octet_length(attempted_digest) = 32),
    reason_code       text NOT NULL,
    detail            jsonb NULL,
    actor_ref         text NOT NULL,
    refused_at        timestamptz NOT NULL DEFAULT now(),
    CHECK (
        idempotency_key IS NOT NULL OR attempted_digest IS NOT NULL
    ),
    CHECK (
        request_ref IS NULL OR
        (idempotency_key IS NOT NULL AND attempted_digest IS NOT NULL)
    ),
    UNIQUE (request_ref, attempted_digest),
    FOREIGN KEY (request_ref, operation_id, idempotency_key)
        REFERENCES command_receipts(
            receipt_ref, operation_id, idempotency_key
        )
);

ALTER TABLE app_auth_events
    ADD CONSTRAINT app_auth_events_receipt_fk
    FOREIGN KEY (receipt_ref) REFERENCES command_receipts(receipt_ref);

ALTER TABLE app_auth_events
    ADD CONSTRAINT app_auth_events_operation_version_fk
    FOREIGN KEY (operation_id, canonicalization_version)
    REFERENCES authority_operation_registry(
        operation_id, canonicalization_version
    );

ALTER TABLE app_recovery_generations
    ADD CONSTRAINT app_recovery_prepared_rotation_fk
    FOREIGN KEY (prepared_by_rotation)
    REFERENCES app_recovery_rotations(rotation_ref);

ALTER TABLE app_recovery_generations
    ADD CONSTRAINT app_recovery_spent_rotation_fk
    FOREIGN KEY (spent_by_rotation)
    REFERENCES app_recovery_rotations(rotation_ref);

ALTER TABLE app_recovery_rotations
    ADD CONSTRAINT app_recovery_rotation_receipt_fk
    FOREIGN KEY (receipt_ref) REFERENCES command_receipts(receipt_ref);

ALTER TABLE app_recovery_elevation_proofs
    ADD CONSTRAINT app_recovery_proof_receipt_fk
    FOREIGN KEY (receipt_ref) REFERENCES command_receipts(receipt_ref);

ALTER TABLE app_recovery_elevation_proofs
    ADD CONSTRAINT app_recovery_proof_auth_event_fk
    FOREIGN KEY (auth_event_ref) REFERENCES app_auth_events(event_id);

ALTER TABLE app_recovery_request_bindings
    ADD CONSTRAINT app_recovery_binding_operation_fk
    FOREIGN KEY (operation_id, canonicalization_version)
    REFERENCES authority_operation_registry(
        operation_id, canonicalization_version
    );

ALTER TABLE app_recovery_rotations
    ADD CONSTRAINT app_recovery_rotation_operation_fk
    FOREIGN KEY (operation_id, canonicalization_version)
    REFERENCES authority_operation_registry(
        operation_id, canonicalization_version
    );

ALTER TABLE app_recovery_elevation_proofs
    ADD CONSTRAINT app_recovery_proof_operation_fk
    FOREIGN KEY (operation_id, canonicalization_version)
    REFERENCES authority_operation_registry(
        operation_id, canonicalization_version
    );
~~~

The owning migration enforces:

- immutable receipt identity and envelope fields;
- no receipt deletion;
- only ACCEPTED → COMPLETED or ACCEPTED → REFUSED;
- no terminal-to-terminal or terminal-to-live transition;
- refusal immutability and no deletion;
- operation-appropriate receipt FKs for every downstream table; and
- canonical outcome and refusal taxonomies.

Same K and same digest discovers the standing receipt. Same K and a different
digest never creates a second receipt: it appends one convergent immutable
CommandRefusal carrying the attempted digest and linking request_ref to the
standing receipt. The composite FK prevents a refusal from linking another
operation or K. Linked and digest-mismatch refusals require attempted_digest.
refusal_identity is the canonical hash of operation id, K, attempted digest,
and standing receipt—reason classification is deliberately excluded so a
later wording/taxonomy change cannot create a second identity. The writer
re-proves that derivation, and UNIQUE (request_ref, attempted_digest) makes a
retry converge.

The request digest uses the repository’s existing pinned canonical JSON
algorithm: object keys sorted, no whitespace, array order preserved, and JSON
leaf rendering as implemented by the Store’s current
<code>canonical_json</code>/<code>sha256_of_canonical</code> path. P2A
factors that algorithm into shared code or proves exact equivalence with
cross-crate vectors and names the receipt version <code>gh-cjson-v1</code>.
RFC 8785 is not silently substituted; adopting it later requires a new
canonicalization version.

### 5.3 Receipt steps and outbox

R19 already needs resumable concurrency at P2A. Resumability is therefore
ACCEPTED plus pending steps under a receipt-scoped A.13-style lease, not a
fourth receipt status.

~~~sql
CREATE TABLE command_receipt_steps (
    receipt_ref          uuid NOT NULL REFERENCES command_receipts(receipt_ref),
    step_no              int NOT NULL CHECK (step_no > 0),
    step_name            text NOT NULL,
    required             boolean NOT NULL DEFAULT true,
    status               text NOT NULL CHECK (
        status IN ('PENDING','DONE','REFUSED')
    ),
    completion_fence     bigint NULL CHECK (
        completion_fence IS NULL OR completion_fence > 0
    ),
    result_digest        bytea NULL CHECK (
        result_digest IS NULL OR octet_length(result_digest) = 32
    ),
    completed_at         timestamptz NULL,
    PRIMARY KEY (receipt_ref, step_no),
    UNIQUE (receipt_ref, step_name),
    CHECK (
        (status = 'PENDING' AND completion_fence IS NULL
                            AND result_digest IS NULL
                            AND completed_at IS NULL) OR
        (status IN ('DONE','REFUSED') AND completion_fence IS NOT NULL
                                        AND completed_at IS NOT NULL)
    )
);

CREATE TABLE command_receipt_leases (
    receipt_ref       uuid PRIMARY KEY REFERENCES command_receipts(receipt_ref),
    claimant_id       text NOT NULL,
    lease_generation  bigint NOT NULL CHECK (lease_generation > 0),
    claimed_at        timestamptz NOT NULL,
    heartbeat_at      timestamptz NOT NULL,
    expires_at        timestamptz NOT NULL,
    CHECK (claimed_at <= heartbeat_at AND heartbeat_at < expires_at)
);

CREATE TABLE command_outbox (
    outbox_ref        uuid PRIMARY KEY,
    receipt_ref       uuid NOT NULL REFERENCES command_receipts(receipt_ref),
    work_identity     text NOT NULL,
    work_kind         text NOT NULL,
    required          boolean NOT NULL DEFAULT true,
    executor_id       text NULL
        REFERENCES authority_operation_registry(operation_id),
    payload           jsonb NOT NULL,
    payload_digest    bytea NOT NULL CHECK (octet_length(payload_digest) = 32),
    status            text NOT NULL CHECK (
        status IN ('PENDING','CLAIMED','DONE','REFUSED')
    ),
    claimant_id       text NULL,
    claim_generation  bigint NOT NULL DEFAULT 0 CHECK (claim_generation >= 0),
    claimed_at        timestamptz NULL,
    claim_expires_at  timestamptz NULL,
    completed_at      timestamptz NULL,
    UNIQUE (receipt_ref, work_identity),
    CHECK (
        (status = 'PENDING' AND claimant_id IS NULL
                            AND claim_generation = 0
                            AND claimed_at IS NULL
                            AND claim_expires_at IS NULL
                            AND completed_at IS NULL) OR
        (status = 'CLAIMED' AND claimant_id IS NOT NULL
                            AND claim_generation > 0
                            AND claimed_at IS NOT NULL
                            AND claim_expires_at > claimed_at
                            AND completed_at IS NULL) OR
        (status IN ('DONE','REFUSED') AND claimant_id IS NOT NULL
                                       AND claim_generation > 0
                                       AND claimed_at IS NOT NULL
                                       AND claim_expires_at > claimed_at
                                       AND completed_at IS NOT NULL)
    )
);
~~~

The P2A migration pins these mechanics:

- a resumer acquires or CAS-renews the receipt lease; expired leases may be
  reclaimed only by incrementing lease_generation;
- a step completion names the observed lease_generation and commits only if
  it still matches, fencing an expired claimant;
- a step implementation is independently idempotent on
  (receipt_ref, step_no), even while the lease prevents ordinary overlap;
- receipt steps are no-delete; their identity/name are immutable and their
  required flag is immutable; their
  only arc is PENDING → DONE|REFUSED under the matching completion fence;
- receipt leases are no-delete; heartbeat renewal preserves lease_generation,
  while every acquisition after expiry—including acquisition by the same
  claimant—requires exactly generation +1, so a delete/reinsert ABA cannot
  reset the fence;
- outbox rows are no-delete; payload, payload_digest, receipt_ref,
  work_identity, work_kind, required, and executor_id are immutable; their only arcs
  are PENDING → CLAIMED → DONE|REFUSED or an expired CLAIMED → CLAIMED
  re-claim at exactly claim_generation +1;
- outbox discovery includes PENDING and CLAIMED with an expired lease;
- reclaim increments claim_generation; completion CAS-matches that generation,
  so an expired claimant cannot complete after a re-claim; and
- executor_id may be NULL only for the exact R03 Gabriel/Lucy
  TRIAL_AUDITOR rows described next. Every other outbox row requires a
  non-null EXECUTOR-namespace id and a matching authority_executor_routes row
  for the receipt operation and work_kind; and
- R03 acceptance has a deferred constraint trigger that, before commit,
  requires exactly two and only two outbox rows for the receipt:
  <code>auditor:GABRIEL</code> and <code>auditor:LUCY</code>, both with
  work_kind <code>TRIAL_AUDITOR</code>, immutable payloads bound to the same
  TrialCycle/evidence set, and no executor_id. UNIQUE prevents duplication;
  the deferred exact-set/count gate proves the required cardinality. Their
  executor_id is null deliberately: Gabriel and Lucy are named auditor
  JobRecords, not aliases for any of the six served supervisor Executor
  operations.

A deferred receipt-terminalization constraint enforces the operation’s pinned
completion_contract from authority_operation_registry. The required
step/outbox manifest is inserted atomically with RCPT acceptance. A deferred
acceptance constraint canonicalizes the exact ordered step and outbox sets
(including required flags, work identities/kinds, executor ids, and payload
digests), checks them against the operation’s pinned acceptance_contract and
work_manifest_digest, and requires work_manifest_sealed_at before commit.
Child INSERT is permitted only while that receipt’s seal is unset in its
creation transaction; the seal is set once, can never be cleared, and later
child INSERT, identity UPDATE, or DELETE is forbidden. TX/DISPATCH require both
manifest fields NULL. Thus an ACCEPTED RCPT cannot commit with zero, missing,
extra, or late-added required children. An RCPT refused before acceptance
seals the operation’s pinned empty/refusal manifest and creates no live child.
COMPLETED requires every required step and outbox item to satisfy
that contract (normally DONE); REFUSED requires the contract’s refusal witness
and prevents undiscovered live work. A receipt therefore cannot become
terminal while a required PENDING/CLAIMED child remains, nor can a caller
delete a required child to manufacture completion.

The served executor’s phase pin supplies the lease duration, heartbeat,
reclaim interval, and terminal refusal policy. The mechanism is the canon
A.13 lease pattern; no bespoke unfenced expiry rule is permitted.

### 5.4 Completion and canon-event rules

Mode semantics are:

- **TX:** for receipt-bearing commands, the receipt is inserted terminal in
  the effects transaction. A01/A02 are also TX but use their single
  app_auth_events identity instead of a CommandReceipt.
- **DISPATCH:** the receipt is inserted terminal in the transaction that
  commits the consent or pending work. The later executor has its own stable
  Executor identity and fenced claim.
- **RCPT:** successful acceptance inserts ACCEPTED with its initial steps and
  outbox atomically; later work reaches one terminal state.
- **Precondition refusal:** only a receipt-eligible authenticated command may
  insert a terminal REFUSED receipt and CommandRefusal in its refusal
  transaction. A01/A02 write their constrained app_auth_events identity and
  can never enter command_receipts.

For successful R commands and A01/A03/A04/A05:

- TX and DISPATCH append COMMAND_ACCEPTED and COMMAND_COMPLETED in their one
  transaction;
- RCPT appends COMMAND_ACCEPTED at acceptance and COMMAND_COMPLETED only with
  its terminal completion; and
- a receipt-eligible R/A03/A04/A05 terminal refusal writes the existing refusal
  event/taxonomy and its CommandRefusal, not fictitious accepted/completed
  events. A01 instead writes only its constrained app-auth refusal event.

A01’s canon events cite its app-auth event because A01 has no CommandReceipt.
A02 emits neither canon command event. Auth detail remains in app_auth_events.

## 6. Notice, warning, and sovereign-judgment shapes

All historical occurrences and transitions below are immutable and no-delete.
Mutable state exists only in named projections. Each downstream receipt FK is
checked against the permitted operation id, COMPLETED status, exact accepted
outcome, subject identity, request digest, and actor—not merely against the
existence of some terminal receipt. A REFUSED receipt can never witness a
resolution, lift, or judgment. Each effect receipt is single-use for its
transition. Sections 6.1–6.2 explicitly preserve pre-substrate legacy facts
without fabricating historical receipts; every transition performed after the
P2B migration still obeys the completed-receipt rule.

### 6.1 AdmissionNotice

~~~sql
CREATE TABLE admission_notice_occurrences (
    notice_ref       uuid PRIMARY KEY,
    manifest_ref     uuid NOT NULL REFERENCES manifests(manifest_id),
    origin_kind      text NOT NULL CHECK (
        origin_kind IN ('NATIVE','LEGACY_0014')
    ),
    scope_digest     bytea NOT NULL CHECK (octet_length(scope_digest) = 32),
    threshold_facts  jsonb NULL,
    legacy_notice_text text NULL,
    legacy_source_digest bytea NULL CHECK (
        legacy_source_digest IS NULL OR
        octet_length(legacy_source_digest) = 32
    ),
    occurrence_no    int NOT NULL CHECK (occurrence_no > 0),
    minted_at        timestamptz NOT NULL DEFAULT now(),
    CHECK (
        (origin_kind = 'NATIVE' AND threshold_facts IS NOT NULL
                                AND legacy_notice_text IS NULL
                                AND legacy_source_digest IS NULL) OR
        (origin_kind = 'LEGACY_0014' AND threshold_facts IS NULL
                                     AND legacy_notice_text IS NOT NULL
                                     AND legacy_source_digest IS NOT NULL)
    ),
    UNIQUE (manifest_ref, scope_digest, occurrence_no),
    UNIQUE (notice_ref, manifest_ref, scope_digest)
);

CREATE TABLE admission_notice_state (
    notice_ref              uuid PRIMARY KEY
        REFERENCES admission_notice_occurrences(notice_ref),
    manifest_ref            uuid NOT NULL REFERENCES manifests(manifest_id),
    scope_digest            bytea NOT NULL CHECK (octet_length(scope_digest) = 32),
    state                   text NOT NULL CHECK (
        state IN ('STANDING','ACKNOWLEDGED','SILENCED')
    ),
    revision                bigint NOT NULL DEFAULT 1 CHECK (revision > 0),
    resolution_transition_ref uuid NULL UNIQUE,
    resolution_ref          uuid NULL REFERENCES command_receipts(receipt_ref),
    resolved_at             timestamptz NULL,
    UNIQUE (resolution_ref),
    CHECK (
        (state = 'STANDING' AND resolution_transition_ref IS NULL
                            AND resolution_ref IS NULL
                            AND resolved_at IS NULL) OR
        (state IN ('ACKNOWLEDGED','SILENCED')
         AND resolution_transition_ref IS NOT NULL
         AND resolution_ref IS NOT NULL
         AND resolved_at IS NOT NULL)
    ),
    FOREIGN KEY (notice_ref, manifest_ref, scope_digest)
        REFERENCES admission_notice_occurrences(
            notice_ref, manifest_ref, scope_digest
        )
);

CREATE UNIQUE INDEX admission_one_standing_per_scope
    ON admission_notice_state (manifest_ref, scope_digest)
    WHERE state = 'STANDING';

CREATE TABLE admission_notice_resolutions (
    transition_ref   uuid PRIMARY KEY,
    notice_ref       uuid NOT NULL UNIQUE
        REFERENCES admission_notice_occurrences(notice_ref),
    outcome          text NOT NULL CHECK (
        outcome IN ('ACKNOWLEDGED','SILENCED')
    ),
    receipt_ref      uuid NOT NULL UNIQUE REFERENCES command_receipts(receipt_ref),
    resolved_at      timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE admission_notice_state
    ADD CONSTRAINT admission_state_transition_fk
    FOREIGN KEY (resolution_transition_ref)
    REFERENCES admission_notice_resolutions(transition_ref);
~~~

The office-only mint gate inserts the immutable occurrence and STANDING
projection together. The sovereign-only R21 gate inserts exactly one immutable
resolution and performs the projection’s one legal transition with revision
+1. resolution_ref is set once, is non-null exactly when resolved,
and must equal the receipt_ref in the immutable row named by
resolution_transition_ref; state and resolved_at must equal that row as well.
That receipt
must be COMPLETED <code>reserved:R21:v1</code> work whose terminal outcome,
request digest, subject, action, and actor match the transition. The occurrence
and projection manifest/scope values must agree by a composite FK or an
equivalent trigger.

**P2B migration of the live 0014 field:** the migration preflights every
non-null <code>manifests.standing_notice</code>; it never invents threshold
facts. For each readable row it inserts one LEGACY_0014 occurrence at
occurrence_no 1, preserving the exact text and a domain-separated canonical
digest of manifest identity, exact notice text, and retained envelope fields.
That digest is used as the explicitly legacy scope_digest and
legacy_source_digest; it is a migration identity, not a claim that 0014 stored
the later empirical scope. The projection starts STANDING, while the original
manifest field remains untouched. If any non-null field cannot be reproduced
and verified, the migration aborts before changing any row.

The migration owner is the only non-office writer allowed to mint
LEGACY_0014 occurrences, and that path disappears after migration. R21 may
resolve one only when H binds the exact preserved text/digest and the request
explicitly acknowledges the legacy origin; its ordinary COMPLETED receipt and
immutable resolution then supply the first sovereign transition witness.
After P2B, any runtime path that would set manifests.standing_notice must
atomically create the corresponding NATIVE occurrence and STANDING projection,
or leave the legacy field null; a notice-bearing manifest cannot commit with
only the old text field.

### 6.2 Bias warning resolution, lift, and re-arm

The existing bias_warnings row becomes the mutable current projection. P2B
must add LIFTED, the current occurrence identity, separate resolution and lift
witnesses, and a legal arc trigger. Its present ON CONFLICT DO NOTHING writer
is replaced: while STANDING, ACKNOWLEDGED, or SILENCED it does not duplicate a
warning. ACKNOWLEDGED remains standing/counting for this purpose. After R22
has moved the projection to LIFTED, a later lawful warning inserts a new
immutable occurrence and advances the projection to that occurrence as
STANDING in the same transaction. A lift never overwrites the silence witness.

~~~sql
CREATE TABLE bias_warning_occurrences (
    occurrence_ref  uuid PRIMARY KEY,
    scope           text NOT NULL,
    origin_kind     text NOT NULL CHECK (
        origin_kind IN ('NATIVE','LEGACY_0009')
    ),
    occurrence_no   int NOT NULL CHECK (occurrence_no > 0),
    threshold_facts jsonb NULL,
    legacy_source_digest bytea NULL CHECK (
        legacy_source_digest IS NULL OR
        octet_length(legacy_source_digest) = 32
    ),
    raised_at       timestamptz NOT NULL DEFAULT now(),
    CHECK (
        (origin_kind = 'NATIVE' AND threshold_facts IS NOT NULL
                                AND legacy_source_digest IS NULL) OR
        (origin_kind = 'LEGACY_0009' AND threshold_facts IS NULL
                                     AND legacy_source_digest IS NOT NULL)
    ),
    UNIQUE (scope, occurrence_no),
    UNIQUE (occurrence_ref, scope, occurrence_no)
);

CREATE TABLE bias_warning_resolutions (
    transition_ref  uuid PRIMARY KEY,
    occurrence_ref  uuid NOT NULL UNIQUE
        REFERENCES bias_warning_occurrences(occurrence_ref),
    outcome         text NOT NULL CHECK (
        outcome IN ('ACKNOWLEDGED','SILENCED')
    ),
    receipt_ref     uuid NOT NULL UNIQUE REFERENCES command_receipts(receipt_ref),
    resolved_at     timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE bias_warning_lifts (
    lift_ref        uuid PRIMARY KEY,
    occurrence_ref  uuid NOT NULL UNIQUE
        REFERENCES bias_warning_occurrences(occurrence_ref),
    receipt_ref     uuid NOT NULL UNIQUE REFERENCES command_receipts(receipt_ref),
    lifted_at       timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE bias_warning_legacy_resolutions (
    legacy_resolution_ref uuid PRIMARY KEY,
    occurrence_ref        uuid NOT NULL UNIQUE
        REFERENCES bias_warning_occurrences(occurrence_ref),
    migrated_status       text NOT NULL CHECK (
        migrated_status IN ('ACKNOWLEDGED','SILENCED')
    ),
    source_digest         bytea NOT NULL
        CHECK (octet_length(source_digest) = 32),
    original_resolved_at  timestamptz NOT NULL,
    migrated_at           timestamptz NOT NULL DEFAULT now()
);

-- P2B projection migration, shown as target columns rather than executable
-- ALTER syntax because the owning migration must replace the old CHECK.
bias_warnings (
    scope                   text PRIMARY KEY,
    current_occurrence_ref  uuid NOT NULL
        REFERENCES bias_warning_occurrences(occurrence_ref),
    occurrence_no           int NOT NULL CHECK (occurrence_no > 0),
    status                  text NOT NULL CHECK (
        status IN ('STANDING','ACKNOWLEDGED','SILENCED','LIFTED')
    ),
    resolution_transition_ref uuid NULL UNIQUE
        REFERENCES bias_warning_resolutions(transition_ref),
    lift_transition_ref     uuid NULL UNIQUE
        REFERENCES bias_warning_lifts(lift_ref),
    legacy_resolution_ref  uuid NULL UNIQUE
        REFERENCES bias_warning_legacy_resolutions(legacy_resolution_ref),
    resolution_ref          uuid NULL REFERENCES command_receipts(receipt_ref),
    lift_ref                uuid NULL REFERENCES command_receipts(receipt_ref),
    raised_at               timestamptz NOT NULL,
    resolved_at             timestamptz NULL,
    lifted_at               timestamptz NULL,
    revision                bigint NOT NULL CHECK (revision > 0),
    schema_name             text NOT NULL,
    schema_version          text NOT NULL,
    produced_by             text NOT NULL,
    produced_at             timestamptz NOT NULL
    , UNIQUE (resolution_ref)
    , UNIQUE (lift_ref)
    , FOREIGN KEY (current_occurrence_ref, scope, occurrence_no)
        REFERENCES bias_warning_occurrences(
            occurrence_ref, scope, occurrence_no
        )
);
~~~

The R15 transition receipt must be COMPLETED
<code>reserved:R15:v1</code> and match the exact occurrence, scope, action,
digest, and actor. The R22 lift receipt must be COMPLETED
<code>reserved:R22:v1</code> with the same matching facts, and a lift may be inserted only for an
occurrence whose immutable resolution outcome is SILENCED. Projection
coherence requires:

- STANDING: neither transition, receipt, nor terminal timestamp exists;
- ACKNOWLEDGED: a native resolution transition/receipt or immutable legacy
  resolution and resolved_at exist; no lift exists;
- SILENCED: a native R15 transition/receipt or immutable legacy resolution and
  resolved_at exist; no lift exists;
- LIFTED: the native-or-legacy silence witness plus a distinct R22
  transition/receipt and both timestamps exist.

Occurrence, resolution, and lift tables are append-only. The projection has
only the named arcs and exact revision increment. Its transition references
must identify rows for current_occurrence_ref, and its resolution_ref,
lift_ref, states, actors, and timestamps must equal the corresponding
immutable transition rows; none can be written independently. The P2B
writer-path change and projection migration are part of this decision, not
optional cleanup.

**P2B migration of live 0009 rows:** schema_name, schema_version, produced_by,
and produced_at remain on the projection. The migration preflights every
existing row and matching retained log/provenance; it never fabricates
threshold_facts or a historical R15 receipt. It creates occurrence_no 1 with
origin LEGACY_0009 and a domain-separated canonical digest of the exact old
row plus retained matching log references. A legacy STANDING row has no
resolution. A coherent legacy ACKNOWLEDGED or SILENCED row receives one
immutable bias_warning_legacy_resolution preserving its old status,
resolved_at, and source digest; resolution_ref remains NULL. If an old row is
incoherent or cannot be verified, the migration aborts before changing any
row.

The runtime migration writer is then removed. A later R15 may resolve a
legacy STANDING occurrence normally. R22 may lift a legacy SILENCED occurrence
only through a fresh sovereign ceremony whose H binds the exact legacy
resolution/source digest and explicitly acknowledges the missing pre-P2B R15
receipt; R22 still writes its ordinary COMPLETED receipt and immutable lift.
Legacy ACKNOWLEDGED remains standing/counting. The projection-coherence trigger
accepts exactly one resolution lineage—native transition+receipt or immutable
legacy resolution—and requires any lift to point to that same occurrence.

### 6.3 SOVEREIGN_JUDGMENT

~~~sql
CREATE TABLE sovereign_judgments (
    judgment_ref             uuid PRIMARY KEY,
    return_ref               uuid NOT NULL REFERENCES returns(return_id),
    criterion_index          int NOT NULL CHECK (criterion_index >= 0),
    evidence_ref             uuid NOT NULL,
    verified_evidence_digest bytea NOT NULL
        CHECK (octet_length(verified_evidence_digest) = 32),
    verdict                  text NOT NULL CHECK (
        verdict IN ('PASSED','FAILED','RETURNED_FOR_REWORK')
    ),
    receipt_ref              uuid NOT NULL UNIQUE REFERENCES command_receipts(receipt_ref),
    rendered_by              text NOT NULL,
    rendered_at              timestamptz NOT NULL DEFAULT now(),
    UNIQUE (return_ref, criterion_index)
);
~~~

The R23 writer is sovereign-authenticated from birth and accepts only a
COMPLETED <code>reserved:R23:v1</code> receipt whose actor equals rendered_by
and whose terminal outcome, request digest, Return, criterion index, evidence
reference, verified digest, and verdict match this row. Before insert it locks
the Return and proves that it is flagged/immutable with a non-null content_sha,
re-proves that content_sha, and proves that the named completion entry:

- has the exact criterion_index;
- carries the exact mandatory evidence_ref;
- has <code>passed IS NULL</code>, the repository’s
  SOVEREIGN_JUDGMENT marker; and
- re-proves the stored evidence under verified_evidence_digest.

The row is immutable and set-once. Changing the evidence digest cannot create
a second judgment for the same Return criterion. RETURNED_FOR_REWORK does not
mutate the Return; it authorizes a separately represented successor workflow.

## 7. Recovery/bootstrap genesis and RecoveryContext rotation

The authority directory is exactly <code>&lt;data_dir&gt;/authority</code>.
The one live recovery credential is exactly
<code>&lt;data_dir&gt;/authority/recovery.credential</code>. Every genesis or
rotation stage file is in that same directory; no configuration may redirect
one half of this consistency unit independently of the other.

### 7.1 Deployment genesis: no generation → generation 1

The first recovery credential is a deployment substrate act, not an
authenticated application command: no RecoveryContext can exist until the
credential exists, and the design does not fabricate a CommandReceipt or
canon event for installation. The deployment-only writer is absent from every
runtime route and runs only while both app_recovery_generations and the live
app_recovery_genesis index are empty.

It uses the same two-substrate discipline:

1. **Claim genesis.** Insert one CLAIMED app_recovery_genesis row and reserve
   generation 1. Its stage_identity is
   <code>recovery.g1.&lt;genesis_ref&gt;.staged</code>.
2. **Stage generation 1 durably.** Generate the credential, write that
   same-directory protected file, flush file and directory, and verify token,
   digest, file type, and platform protection.
3. **Prepare the database.** In one transaction insert recovery generation 1
   as PREPARED with prepared_by_genesis, persist the verifier, and move the
   genesis row CLAIMED → PREPARED with the identical verifier.
4. **Promote the file.** Atomically replace the live recovery file with the
   staged file, sync the directory, and verify the live file.
5. **Activate genesis.** In one database transaction move generation 1
   PREPARED → ACTIVE and the genesis row PREPARED → ACTIVE.

Startup performs genesis reconciliation before exposing A01:

| Genesis observation | Action |
|---|---|
| CLAIMED; no generation row; staged file absent | resume generation/staging under the same genesis_ref |
| CLAIMED; matching staged file; no generation row | verify and run prepare |
| PREPARED generation/genesis; matching staged file | promote, then activate |
| PREPARED generation/genesis; live file matches generation 1 | activate |
| ACTIVE generation/genesis; live file matches | normal; a bound recovery-elevation/A05 rotation may now begin |
| PREPARED or ACTIVE database state without a matching durable file | fail closed and require the two-substrate restore; never invent a replacement verifier |
| unreferenced genesis stage file or ABANDONED claim | quarantine/remove after proving it is neither live nor referenced |

Before PREPARED, a failed CLAIMED attempt may become ABANDONED and a new
genesis claim may win. At or after PREPARED, the verifier is committed and the
routine never silently regenerates it. A01 is unavailable until generation 1
and the live file are both ACTIVE/matching. This deployment routine has
fault-injection coverage at every boundary and no LAN surface.

### 7.2 Bootstrap-secret genesis and lost delivery

Only after recovery generation 1 is ACTIVE/matching, the deployment-only host
path generates a canonical <code>gdh_boot_v1_</code> secret from 32 fresh OS
random bytes, commits bootstrap generation 1 as UNUSED with only its
class-separated digest, and then renders the plaintext exactly once to the
interactive host CLI. It never writes that plaintext to logs, Debug/panic
output, HTTP traces, model context, provenance, shell history generated by the
tool, or a repository file. There is no LAN delivery.

The database commit precedes display so an unregistered secret is never handed
out. Initial issuance and lost-display reissue are the same offline installer
subsystem, not runtime authentication controls. The installer:

1. runs as the named deployment/service principal while holding the pinned
   session-scoped PostgreSQL advisory lock for bootstrap delivery on a
   dedicated, non-pooled connection through the one-time display;
2. has no runtime route, network handler, authority-crate public API,
   RecoveryContext, OperationId, receipt, app-auth event, or canon event;
3. locks and rechecks that no client has ever existed, no bootstrap generation
   was CONSUMED, and exactly one current UNUSED row exists for reissue;
4. generates the replacement plaintext in memory, then in one transaction
   moves that locked row to SUPERSEDED with its adjacent successor witness and
   inserts generation N+1 UNUSED at the new digest; and
5. commits before displaying the new plaintext exactly once.

The same deployment lock serializes reissue against A01, so a secret cannot be
printed while another transaction consumes or supersedes it. If the process
dies after commit or the display is lost, the operator repeats the installer
step: the committed-but-undelivered generation is superseded and a later
adjacent generation is displayed. Old plaintext never becomes valid again.
This path changes only the insufficient deployment factor; A01 still requires
both that factor and a freshly minted RecoveryContext. It is therefore not a
hidden enrollment path.

The installer writer is structurally absent once any client exists or any
bootstrap generation is CONSUMED. After enrollment, sole-client credential
loss uses A05 rather than reopening deployment bootstrap. A01 is exposed only
when both an ACTIVE/matching recovery generation and one UNUSED bootstrap
generation exist.

A01 acquires that same advisory lock before it binds the UNUSED bootstrap
generation and holds the dedicated connection through recovery elevation and
the enrollment TX. Connection loss releases the lock and strands/abandons the
attempt according to §7.3. The advisory lock is not authority or a substitute
for the TX's row lock and generation recheck; it is cross-process exclusion
between the only two structurally admitted writers. P2A pins the numeric lock
key, proves the connection never returns to the pool while held, and races
disconnect/reissue/A01.

The upper-crate A01 wrapper pairs its core RecoveryContext with a non-clone
guard that owns this exact dedicated connection; the core leaf never depends
on a database handle. Only that pair may attempt the enrollment TX on the
guarded connection. Lock/connection loss before the TX strands the READY proof
and makes the wrapper unusable.

### 7.3 RecoveryContext mint and rotation: N → N+1

The recovery file and PostgreSQL are two substrates and cannot share a native
transaction. The design therefore completes rotation before a TX
RecoveryContext exists, while A05 makes rotation part of its already-ruled
RCPT work. A protected-token verifier match alone is not authority use: only a
READY proof, a move-only A05AcceptanceContext, or an accepted A05 continuation
can cross its corresponding typed control boundary.

#### 7.3.1 TX recovery-elevation preflight: A01/A03/A04

1. **Bind the attempted elevation.** After token grammar, class-separated
   digest, host-local source, ACTIVE-N, expected-state, and freshness checks,
   first discover any exact terminal event/receipt and return only that state;
   this idempotency read mints no context and delivers no bearer plaintext.
   Otherwise insert or discover the immutable `(operation_id,K)` request
   binding. A
   different digest/version appends ELEVATION_IDEMPOTENCY_MISMATCH and stops
   before context mint. A same-digest retry locks the binding and assigns the
   next attempt_no only after any prior attempt is ABANDONED, STRANDED, or
   command-terminal; a live rotation or READY proof is not redeemable by the
   retry. It then inserts one CLAIMED TX_ELEVATION rotation bound to
   the current process issuer epoch. No CommandReceipt, command event, or
   RecoveryContext exists yet.
2. **Stage N+1 durably.** Create
   <code>recovery.g&lt;N+1&gt;.&lt;rotation_ref&gt;.staged</code> in the authority
   directory with a fresh canonical credential. Apply the platform contract;
   flush file and directory; and verify type, protection, stage identity,
   generation, token grammar, and digest.
3. **Prepare rotation.** One database transaction CAS-validates the immutable
   binding, CLAIMED rotation, ACTIVE N, and absence of a prior transition. It
   marks N SPENT by rotation_ref, inserts N+1 PREPARED by the same rotation,
   inserts the adjacent immutable transition, and moves the rotation to
   PREPARED. It performs no A01/A03/A04 domain effect and writes no terminal
   command identity.
4. **Promote and activate.** Verify the staged file against the PREPARED
   rotation, atomically replace
   <code>&lt;data_dir&gt;/authority/recovery.credential</code>, durably sync the
   directory, and verify the live file. One database transaction then moves
   N+1 PREPARED → ACTIVE and the rotation PREPARED → ACTIVATED. If the original
   issuer epoch is still live and freshness still holds, that transaction also
   inserts the exact proof READY; otherwise it inserts the proof STRANDED.
5. **Mint and consume once.** Only a READY proof plus the still-live private
   issuer mints the non-serializable, proof-bound RecoveryContext. There is no
   response boundary before its typed wrapper runs. The A01/A03/A04 command TX
   conditionally moves READY → CONSUMED and, atomically, commits either its
   effects or its stale/precondition refusal together with the exact terminal
   app event/receipt, provenance, and applicable canon events. A crash during
   that TX is all-or-none. A refusal still used the context, so the completed
   N → N+1 rotation remains in force.

READY never authorizes startup reminting. Loss of the process issuer, wrapper,
or freshness occurrence moves it to STRANDED with no command anchor. A retry
must present the then-ACTIVE credential and perform another complete rotation;
it may reuse the same immutable K/digest binding while fresh, but receives a
strictly greater attempt_no. After binding freshness expires, retry is a new
logical request with a new K. Neither ABANDONED nor STRANDED can be rewritten
or resurrected.

#### 7.3.2 A05 RCPT rotation

A05's sealed authentication API first verifies the active recovery credential,
host-local source, exact operation/K/digest, expected state, and freshness into
a private non-authorizing verifier result. A failed acceptance precondition may
use only the receipt-eligible, no-effect refusal writer: it inserts the
terminal REFUSED receipt, linked CommandRefusal, and app event; it mints no
RecoveryContext and cannot reach recovery effects. If all preconditions pass,
the result is consumed to mint
one move-only A05AcceptanceContext bound to N and that exact request.

The context's sole acceptance wrapper consumes it in one transaction that
inserts the ACCEPTED receipt, required-step manifest, CLAIMED A05 rotation,
sequence-1 RECOVERY_ACCEPTED app event, and canon COMMAND_ACCEPTED. It also
locks the exact client, session, and token-family revisions and installs the
persisted A05 auth-state fence; every conflicting auth writer checks it. A
rollback commits none of these facts and the context cannot be reused. A
commit is the successful context-use boundary: the live rotation prevents N
from authorizing another use, and the receipt-derived continuation is
restricted to this rotation and pending step. From that point, revision drift
cannot turn the durable command into an unrotated refusal; operational failure
remains resumable or fails closed for restore.

The same staged-file operation as §7.3.1 step 2 follows. The prepare
transaction then CAS-validates N, the ACCEPTED receipt, request digest, and
expected state; applies the revocation/replacement-enrollment effects; marks N
SPENT and N+1 PREPARED by this rotation; inserts the adjacent transition; and
moves the rotation to PREPARED while the receipt remains ACCEPTED. After
atomic file promotion and verification, one activation transaction moves N+1
to ACTIVE, the rotation to ACTIVATED, its final step and receipt to COMPLETED,
and appends the application/canon completion events. That is the desk-ruled
five-stage A05 protocol; it never creates a TX elevation proof.

#### 7.3.3 Startup reconciliation before authority routes

| Database/filesystem observation | Deterministic startup action |
|---|---|
| CLAIMED TX_ELEVATION; N ACTIVE | mark ABANDONED, quarantine any attributable stage, and leave N active; a same-binding retry may create the next attempt |
| CLAIMED A05 + ACCEPTED receipt; N ACTIVE | acquire the fenced receipt lease and resume the pending staged-file step |
| PREPARED rotation; N SPENT; N+1 PREPARED; matching stage | promote and verify; activate under the immutable transition |
| PREPARED rotation; N SPENT; N+1 PREPARED; live file matches N+1 | activate under the immutable transition |
| TX_ELEVATION activated by startup/new issuer | insert STRANDED, never READY; do not execute or reconstruct an unstored TX payload |
| READY proof whose issuer epoch is not the current live epoch | mark STRANDED before exposing any authority route |
| ACTIVATED TX rotation + CONSUMED proof + terminal command anchor | normal; exact retry discovers the standing event/receipt and no bearer plaintext |
| ACTIVATED A05 rotation + COMPLETED receipt | normal; exact retry returns terminal state and no bearer plaintext |
| PREPARED/ACTIVATED database state without the matching durable live file | fail closed, retain evidence, and require two-substrate restore; never resurrect N or mint N+2 |
| unreferenced or mismatched stage/live file | quarantine only after proving it is not the referenced consistency member |

An immutable rotation/transition authorizes only deterministic file promotion
and activation. It does not authorize a TX command. A01/A03/A04 remain TX
because rotation and context mint finish before the command begins, then
effects, identity, provenance, and logs commit together. A05 remains RCPT
because its command becomes terminal only with activation.

### 7.4 POSIX contract

The P2A platform pin names the concrete filesystem/syscall APIs used to
realize and test this contract; this ADR fixes the required behavior, not a
training-memory crate choice.

- authority directory <code>&lt;data_dir&gt;/authority</code>: service UID,
  exact mode 0700;
- live and staged credential: regular non-symlink, service UID, exact mode
  0600;
- exclusive creation with no-follow behavior;
- staging in the same directory/filesystem;
- durable file flush, atomic replacement, directory sync, and post-operation
  metadata/content verification.

The implementation may account for umask while creating, but post-create
verification must still establish the exact final contract.

### 7.5 Windows contract

The same P2A pin names the concrete Windows creation, flush, replacement,
reparse inspection, and security-descriptor APIs and records their supported
versions.

- explicit protected DACL at creation, not an inherited default;
- allow entries for the named service principal and any deliberately
  authorized recovery-operator principal, with inheritance behavior stated;
- refusal of reparse points and unexpected file types;
- explicit same-directory replacement semantics and DACL-preservation policy;
- supported APIs that provide the pinned durable file-flush and replacement-
  persistence semantics; if no supported Windows API combination can provide
  them, the platform gate fails closed rather than weakening the contract; and
- post-create and post-replacement verification of DACL, file identity,
  generation, and content digest.

“Owner-only” is not a sufficient Windows access-control definition.

### 7.6 Verification and restore

P2A fault-injects a crash before and after every recovery/bootstrap genesis,
rotation, proof-mint/consume, and activation step and proves all startup-table
branches, including READY → STRANDED on issuer loss. It also races bootstrap
reissue against A01 and loses the display after each durable boundary. The
project's scheduled backup/restore test treats PostgreSQL
plus filesystem atoms as one consistency unit and explicitly includes the
active/prepared recovery generation, request binding, rotation, transition,
proof, and credential file. A restore with only one substrate must fail closed.

## 8. Taxonomy, amendment boundary, and event residence

- **A.5 names from matrix row :33:** COMMAND_ACCEPTED,
  COMMAND_COMPLETED, ADMISSION_NOTICE_RESOLVED, BIAS_SCOPE_LIFTED, and
  SOVEREIGN_JUDGMENT_RECORDED. Owning phases extend the A.5 CHECK constraint.
- **Authority-map split:** A01/A03/A04/A05 emit generic command events with
  their stable operation id. A02 remains application-side under the bounded
  Law V.1 reading in §4.3. Authentication detail never enters canon merely
  because a receipt exists.
- **A.4 UNSHIPPED_OPERATION:** CommandRefusal cites its command/request
  identity and never fabricates a job id. The canon sentence permitting that
  citation is amendment-process text.
- **Pairing sovereignty:** R17a/R17b and their structural seals are in the
  inventory. The A.10/IX.5/Handbook amendment text and the acknowledged
  pricing travel with P5.
- **Judgment text:** B.2/Handbook text describing the immutable judgment
  occurrence is amendment-process work; §6.3 fixes the record shape.
- **A.12:** no ConsentRecord amendment is implied. Notice, warning, and
  command resolution references cite CommandReceipt, not consent.

## 9. Acceptance criteria and phase handoff

P2A cannot pin until tests demonstrate:

1. token grammar, class-separated digesting, fixed-width storage, revocation,
   scanner fixtures, and all named redaction surfaces;
2. the dependency gate and recorded lockfile/platform evidence;
3. the crate DAG, five unforgeable context types, operation/digest/revision/
   occurrence binding, sovereign elevation, the restricted initial A05
   acceptance capability, wrong-issuer refusal, and receipt/lease-derived
   continuation after restart;
4. 26 Reserved + five Auth + six Executor ids in code and the DB mirror,
   with exact allowed-class sets, pre-context auth requirements,
   mode/envelope/completion agreement, sealed runtime registry writes, and
   permitted executor routes;
5. wrong-context and raw-call compile failures plus the sealed primitives;
6. deployment generates/digest-stores and host-delivers bootstrap generation
   1 exactly once; lost display reissues only through the locked, pre-client
   offline installer and serializes with A01; A01 still requires loopback/host
   CLI, the current secret, and a freshly elevated RecoveryContext, with no LAN
   route;
7. access and refresh lifecycles are independent; concurrent refresh has one
   winner; its successor is same-family generation +1 with one predecessor;
   replay revokes the family; A04 CASes session revision;
8. auth events are post-auth only, append-only, per-control constrained, and
   capable of identifying integer generations; A01–A04 have one root and A05
   follows its accepted/reconciled/terminal automaton;
9. receipts enforce one authenticated logical command identity, immutable
   envelope fields, three-state coherence, registry/class composite FKs,
   the exact 29-operation eligibility gate, operation/K-correct refusal
   references, and same-K/different-digest refusal without a second receipt;
10. R19 receipt leases and outbox claim generations fence stale workers, and
    no-delete/arc/terminal-child gates close ABA; R03 has exactly the Gabriel
    and Lucy items, not merely at-most-once identities;
11. A01/A02/A03/A05 lost responses return no plaintext and follow the named
    recovery paths;
12. recovery genesis, every A01/Recovery-A03/A04 elevation, and A05 rotate
    through one generation-wide rotation/adjacent-transition protocol and
    converge at every injected boundary on POSIX and Windows; initial A05
    context consumption atomically creates its receipt/rotation obligation,
    READY is
    process-bound and restart-stranded, bootstrap reissue remains deployment-
    only and races A01 safely, A01 stays unreachable until both genesis
    credentials are ready, and the restore test includes the recovery
    generation;
13. all unshipped rows refuse through the same authenticated perimeter; and
14. an enrolled operator reaches R19/R20 end to end before P2B.

P2B additionally proves the admission and bias occurrence/transition shapes,
COMPLETED subject/outcome-matching single-use receipt witnesses, projection-
transition equality, LIFTED projection/re-arm writer, and all six Executor
behaviors. Its 0014/0009 migration tests cover empty and populated legacy
states, preserve original fields/envelopes, abort on unverifiable data, and
never fabricate threshold facts or historical receipts. P5 proves the R23 flagged Return/criterion/evidence join,
content_sha re-proof, single-use completed receipt, and authenticated
rendered_by.

## 10. D7 matrix-row consumption map

| D7 row | Disposition |
|---|---|
| :31 CommandReceipt, step/outbox state, CommandRefusal | §§5.1–5.4; uniform authenticated command identity, three-state receipts, P2A lease/fencing, immutable outbox payload |
| :32 A.4 UNSHIPPED_OPERATION command/request identity | §§5.2, 8; shape fixed, canon sentence deferred |
| :33 five A.5 event names | §§5.4, 8; names preserved, A02 bounded application-side exception recorded |
| :34 B.2 / Handbook SOVEREIGN_JUDGMENT | §6.3; Return + criterion_index identity, mandatory evidence_ref/digest, R23 receipt; canon text deferred |
| :35 AdmissionNotice occurrence/revision/scope/resolution/re-arm | §6.1; immutable occurrence/resolution plus mutable projection and separate office/sovereign gates |
| :36 bias occurrence/lift lineage | §6.2; immutable resolution and lift witnesses, LIFTED projection, writer migration and lawful re-arm |
| :37 pairing sovereignty and operation-specific Devout/Doctor commands | §§3.3, 3.5, 8; inventory and seal fixed, canon text deferred, pricing travels with P5 |

The supporting operation-class mirror, authentication application records,
session split, staged recovery state, and Executor ids implement the adopted
D7 mechanics; they do not consume a different decision row.

## 11. Desk rulings and correction record (2026-07-10)

Reviewed under DESK_ADR3_ROUND2_CONSOLIDATED. The round-2 ruling sustains the
independent mechanism-composition review and supersedes round 1 on collisions.
The desk expressly retracts its earlier claims that formats/layout were
untouched and that A05’s file replacement could occur “in the recovery
transaction.” It also records the missed authority → domain → authority
cycle and the impossible one-witness silence/lift lineage.

This revision also retracts its own interim count of 25 Reserved ids. The
literal annex has 26 because both R02 and R17 split into a/b members; the
complete inventory is therefore 26 Reserved + five Auth + six Executor = 37.

A later composition check during this same rework refuted two more interim
sentences before return: bootstrap reissue cannot authenticate with a
RecoveryContext while remaining outside the operation/event inventory, and a
TX control cannot record terminal completion before its mandatory recovery
rotation is ACTIVE. Sections 3, 4, and 7 replace those mechanisms rather than
hiding the corrections: reissue is now a locked pre-client installer act with
no recovery authority, while A01/A03/A04 rotate before context mint and consume
a process-bound proof inside their TX. The discarded stage-claim draft is not
the governing mechanism.

The next frozen schema/coverage audit found three further omissions, all
applied here: receipt eligibility had been promised but not FK-enforced; the
generic refusal sentence accidentally included A01/A02; and A05 acceptance
referred to its receipt continuation without naming the initial context that
creates that receipt. The 29-row eligibility relation, narrowed refusal rule,
and move-only A05AcceptanceContext close those gaps. The same audit restricted
ABANDONED to TX_ELEVATION, so an accepted A05 receipt cannot escape its
rotation by changing a claim to terminal abandonment. These are corrections,
not a claim that this revision certifies itself.

Final dispositions, applied here:

- **Q1:** SHA-256 confirmed with narrowed KDF claim, canonical 32-byte token
  mechanics, binary verifier storage, validation-before-lookup, class
  separation, and a limited fixed-operand comparison claim.
- **Q2:** versioned prefixes adopted with an exact scanner grammar, fake
  fixtures, and cross-surface redaction criteria.
- **Q3:** deferral confirmed as a blocking P2A dependency gate with versions,
  features, entropy, MSRV, advisories, platform, failure, and vector evidence.
- **Q4:** two-crate dependency-leaf layout; operation-bound contexts; typed
  wrappers over a private dispatcher; a move-only initial A05 acceptance
  restriction; compile-fail and arch evidence.
- **Q5:** A01/A03/A04/A05 emit canon command events; A02 stays application-side
  under a bounded Law V.1 reading; app_auth_events is post-auth, constrained,
  versioned, generation-capable, correlated, and append-only, with the narrow
  recognized-consumed-refresh replay event attributed without minting context.
- **Q6:** ACCEPTED, COMPLETED, REFUSED only; full timestamp/outcome
  coherence, immutability, no-delete, one legal live-to-terminal walk, and
  created_at naming.
- **Q7:** one receipt per authenticated logical R/A03/A04/A05 command; A01/A02
  anchor in app_auth_events; same-K/different-digest refusal links the standing
  receipt; a pre-command recovery-elevation mismatch is separately typed and
  cannot create or stand in for a receipt; the 29-row eligibility FK excludes
  A01/A02/Executors; resolution FKs prove the permitted operation; R02b uses a
  persisted trigger occurrence.
- **Q8:** immutable occurrence and transition records plus mutable projection;
  admission has one resolution; bias has distinct resolution and lift
  witnesses and a required P2B projection/writer migration.
- **Q9:** terminal token rows retained for replay recognition; separate access
  and refresh records, A04 revision, TTL config revision, no redundant client
  field, conditional consume, no-delete, and live indexes.
- **Q10:** recovery genesis and every RecoveryContext use are crash-convergent.
  A05's initial restricted context atomically creates the ACCEPTED receipt and
  inescapable rotation, then remains the ruled RCPT five-stage protocol.
  A01/A03/A04 retain TX because
  N+1 is fully active before a process-bound proof can mint their context, and
  READY is consumed atomically with the command TX or stranded. All rotations
  share staged N+1, deterministic startup reconciliation, exact POSIX/Windows
  contracts, fault injection, and restore; bootstrap reissue is a separate
  pre-client installer act and uses no recovery authority.
- **Q11:** OperationId has Reserved/Auth/Executor namespaces in one inventory
  and arch test with distinct APIs; A01/A02 never use generic dispatch.

Additional adopted findings are applied in §§3–9: P2A receipt concurrency,
outbox semantic uniqueness, claim fencing, receipt authority/mode/freshness/
canonicalization fields, the operation-class DB mirror, lost-response
semantics, conjunctive A01, concrete sovereign elevation, corrected
SOVEREIGN_JUDGMENT identity, authenticated rendered_by, six Executor ids,
DependencyLossContext in the census, per-table enforcement requirements, and
the same-commit annex/status reconciliation.

---

*Accepted as a desk-reviewed, non-canonical ADR record on 2026-07-10. It
authorizes owning phases to implement these shapes under the project’s
two-commit lifecycle; it does not itself land code, migrations, canon text,
or live credentials.*
