# CASS Maintained Fork — Backlog

This backlog tracks work specific to
`sockpuppet9000/coding_agent_session_search`. Upstream product work should remain
in the upstream issue/project system unless the fork deliberately owns a
long-term divergence.

Real coding-agent histories, databases, raw mirrors, exports, recovery artifacts,
model caches, logs, screenshots, credentials, local paths, and search results are
private data and must never be committed as fork fixtures or uploaded to public
CI/issues/releases.

## CASS-FORK-001 — Define the fork's purpose and lifetime

- Priority: P0 governance
- Status: Needs decision

- [ ] Decide whether this fork exists for temporary upstreamable patches, long-lived private compatibility changes, CI reproduction, or an independently supported distribution.
- [ ] Name the maintainer and support expectations for each retained fork-only feature.
- [ ] Define conditions for deleting/archiving the fork, resetting it to upstream, or intentionally diverging.
- [ ] Record the decision in `FORK_STATUS.md` and repository metadata.
- [ ] Avoid branding the fork as a separate product until this decision is explicit.

## CASS-FORK-002 — Establish an upstream synchronization contract

- Priority: P0
- Status: Open

- [ ] Record the exact upstream remote and last synchronized upstream commit.
- [ ] Maintain a machine-readable ledger of fork-only commits and their upstream issue/PR/disposition.
- [ ] Define merge versus rebase policy and how rewritten upstream history is handled.
- [ ] Run a clean comparison after every upstream sync and classify generated/release-only drift separately.
- [ ] Prevent upstream synchronization from silently including local databases, model files, build output, `.env`, or recovery artifacts.
- [ ] Add an operator checklist for fetch, inspect, branch, sync, verify, and rollback.

## CASS-FORK-003 — Resolve the two long-lived internal PRs

- Priority: P0
- Status: Open

- [ ] Re-evaluate PR #1 (DB-resident FTS repair) against current `main` and current schema/index behavior.
- [ ] Re-evaluate PR #2 (semantic append/resume indexing) against current `main` and current semantic checkpoint logic.
- [ ] Separate still-needed fixes from commits already superseded upstream or on `main`.
- [ ] Forward-port only minimal current patches with new focused tests.
- [ ] Close superseded PRs with a concise evidence note rather than leaving ambiguous release branches open indefinitely.
- [ ] Never treat the old local validation notes as current-main proof without rerunning the exact relevant gates.

## CASS-FORK-004 — Create a fork-only change ledger

- Priority: P0
- Status: Open

- [ ] Add a versioned table of fork-only behavior, rationale, source issue, first commit, current owner, tests, compatibility risk, and upstream status.
- [ ] Distinguish source changes, documentation overlays, local build wrappers, release infrastructure, and machine-only configuration.
- [ ] Mark every change as upstreamed, pending upstream, intentionally divergent, experimental, or obsolete.
- [ ] Generate release notes from this ledger rather than from raw commit counts.
- [ ] Add a check that a fork release cannot proceed with undocumented divergence.

## CASS-FORK-005 — Separate portable policy from machine overlays

- Priority: P0 maintainability
- Status: Open

- [ ] Split portable contributor/project rules from maintainer-machine paths and commands currently present in `AGENTS.md`.
- [ ] Move `/Users/...`, `/Volumes/...`, `/data/projects/...`, SSD wrappers, local sibling checkouts, and host-specific release machinery into ignored or separately scoped local documentation.
- [ ] Keep portable fallback commands for contributors without the maintainer's wrappers.
- [ ] Define precedence among upstream instructions, fork policy, and local overlays.
- [ ] Add a CI check preventing personal absolute paths from entering portable docs/config unless explicitly allowlisted.

## CASS-FORK-006 — Clarify release authority and versioning

- Priority: P0 before fork releases
- Status: Open

- [ ] Decide whether the fork will ever publish binaries, packages, installer scripts, Homebrew/Scoop metadata, containers, or GitHub Releases.
- [ ] Use an explicit fork version suffix or independent version namespace.
- [ ] Bind every artifact to exact commit, Cargo.lock, features, target triple, compiler, linker, dependency sources, and build workflow.
- [ ] Publish fork-owned checksums, signatures/attestations, provenance, SBOM, changelog, and install/uninstall instructions.
- [ ] Never publish fork artifacts under an upstream tag or reuse upstream attestations.
- [ ] Define how `main` and any legacy `master` mirror are synchronized without making owner-specific branch policy a general contributor requirement.
- [ ] Add rollback/revocation guidance for compromised or incorrect fork artifacts.

## CASS-FORK-007 — Retire or re-scope stale release ledgers

- Priority: P1
- Status: Open

- [ ] Mark `RELEASE_TODO.md` with creation date, covered version range, status, and superseding document.
- [ ] Separate historical evidence from current release gates.
- [ ] Reconcile the reviewed package version with latest fork tags/releases and current branch state.
- [ ] Move one-off host incidents and old queue/rate-limit observations into dated postmortem notes when still useful.
- [ ] Ensure no stale checklist is interpreted as current proof merely because every box is checked.

## CASS-FORK-008 — Review license and contribution compatibility

- Priority: P0 policy/legal
- Status: Open

- [ ] Review the “MIT License with OpenAI/Anthropic Rider” with qualified counsel or organizational policy owners before making open-source/OSI/compatibility claims.
- [ ] Ensure all fork distributions preserve the exact required notices and rider.
- [ ] Define whether inbound contributions require a contributor attestation or additional agreement.
- [ ] Review compatibility with dependencies, package registries, downstream distributions, automated build services, and potential contributors.
- [ ] Avoid labeling the repository simply “MIT” in badges, metadata, or release pages if that omits material restrictions.
- [ ] Document non-legal operational guidance separately from legal conclusions.

## CASS-FORK-009 — Define private-data and threat-model gates

- Priority: P0 security/privacy
- Status: Open

- [ ] Maintain a data inventory for source sessions, SQLite, lexical/semantic indexes, raw mirrors, quarantine, analytics, exports, encryption metadata, sync state, model files, logs, and backups.
- [ ] Define file modes, ownership, symlink/path containment, quotas, retention, cleanup, backup, encryption, and incident response.
- [ ] Add synthetic hostile fixtures for credentials, private URLs, ANSI/OSC, bidi controls, huge records, malformed encodings, path traversal, and hostile HTML/Markdown.
- [ ] Verify private/share-safe output tiers for robot APIs, diagnostics, exports, support bundles, and screenshots.
- [ ] Document every optional network effect, model download, sync destination, and release/update request.
- [ ] Ensure CI and public bug reports cannot ingest real local histories.

## CASS-FORK-010 — Validate fork CI independently

- Priority: P1
- Status: Open

- [ ] Define the minimum fork PR matrix and the larger release matrix.
- [ ] Ensure workflows run against the exact fork branch/commit and do not rely on upstream-only secrets or infrastructure.
- [ ] Reconcile the currently documented UBS gate with actual baseline/noise policy and record truthful pass/fail status.
- [ ] Add fresh-clone, install, robot-schema, migration, index-rebuild, encryption/export, cross-platform, baseline-CPU, and release-artifact checks as applicable.
- [ ] Keep privileged, private-history, performance-host, and real-sync tests as explicit private gates.
- [ ] Retain logs/artifacts only long enough for debugging and scan them for private data.

## CASS-FORK-011 — Verify dependency and build provenance

- Priority: P1 supply chain
- Status: Open

- [ ] Inventory crates.io, Git-revision, vendored binary, installer, model, native library, and workflow dependencies.
- [ ] Verify pinned Git revisions remain available and match expected source.
- [ ] Record ONNX/runtime, OpenSSL, model, and platform-specific binary provenance.
- [ ] Add dependency policy for wildcard/range updates, yanked releases, compromised upstreams, and emergency pinning.
- [ ] Produce an SBOM and source manifest for any fork artifact.
- [ ] Test that local `[patch]`/sibling checkout overrides cannot leak into releases.

## CASS-FORK-012 — Keep upstream presentation and fork identity distinct

- Priority: P2
- Status: Open

- [ ] Retain upstream artwork/screenshots only with correct attribution and repository context.
- [ ] Add a small fork badge/banner only if fork-built artifacts or support are offered.
- [ ] Use synthetic histories, paths, repositories, agents, and search results for fork-specific screenshots.
- [ ] Do not imply upstream endorsement of fork changes.
- [ ] Link prominently to upstream and to `FORK_STATUS.md` from any future fork release page.

## Decisions recorded

- The upstream README remains the product README; this PR does not rewrite it.
- Fork-specific maintenance and publication facts live in `FORK_STATUS.md` and this backlog.
- Historical release checklists are evidence, not current release status by default.
- A package version is not proof that matching fork artifacts were published.
- Real coding-agent histories never belong in public fork fixtures, CI, issues, or screenshots.
- The license includes material additional restrictions and must not be summarized as unmodified MIT.