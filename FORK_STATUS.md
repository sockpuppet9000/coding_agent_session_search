# Maintained Fork Status — `sockpuppet9000/coding_agent_session_search`

This repository is a **public maintained fork** of
[`Dicklesworthstone/coding_agent_session_search`](https://github.com/Dicklesworthstone/coding_agent_session_search),
not a separately branded replacement product.

The upstream project name, package metadata, release links, install scripts,
screenshots, documentation, and primary support expectations remain upstream-owned
unless a fork-specific decision explicitly says otherwise.

> [!IMPORTANT]
> The main `README.md` is the upstream product README. Read this file before
> treating the fork as a release source, installation authority, support channel,
> or independent distribution.

> [!CAUTION]
> CASS indexes local coding-agent histories. Depending on enabled connectors and
> features, its databases, raw mirrors, exports, logs, search results, recovery
> artifacts, model state, and sync material can contain prompts, source code,
> commands, tool output, credentials, private paths, repository names, URLs,
> account identifiers, and work-pattern metadata. Fork maintenance must never use
> real local history as a public fixture, CI artifact, issue attachment, or
> screenshot.

## Current fork snapshot

| Area | Reviewed state |
|---|---|
| Repository role | Maintained public fork |
| Canonical upstream | `Dicklesworthstone/coding_agent_session_search` |
| Default branch | `main` |
| Package version on reviewed `main` | `0.6.5` |
| Product maturity label in upstream README | Alpha |
| Upstream-facing package/repository metadata | Still points to the canonical upstream |
| Fork-specific release authority | Not defined |
| Fork-specific changelog | Not defined |
| Fork-specific support policy | Not defined |
| Open internal fork PRs | Two long-lived engineering PRs |
| Fork-specific CI/release acceptance | Not defined as a separate policy |
| Product artwork/screenshots | Already present upstream; do not fork casually |

## Why this document exists

A maintained fork has two distinct responsibilities:

1. preserve a usable, reviewable relationship to upstream; and
2. make local/fork-only changes, release claims, and support promises explicit.

Without that split, users can easily mistake:

- upstream documentation for proof that this fork was tested;
- a fork tag for an upstream release;
- a local machine convention for a portable contributor requirement;
- an internal CI branch for a supported public feature;
- a package version in `Cargo.toml` for proof that matching artifacts were
  published by this fork;
- a successful local test for evidence that the complete upstream release matrix
  passed.

## Current documentation drift

### `README.md`

The main README is comprehensive and presentation-ready, but it is an upstream
product document. Its install examples, badges, release links, repository URLs,
Homebrew/Scoop references, screenshots, and support expectations point to the
canonical upstream project.

That is appropriate while this remains a maintained fork. Fork-specific claims
should live in this file or another clearly named fork document rather than
silently rewriting upstream identity throughout a very large README.

### `RELEASE_TODO.md`

`RELEASE_TODO.md` is a detailed historical operational ledger centered on the
v0.4.x release sequence. The reviewed package metadata is already at v0.6.5.
Therefore the file is valuable evidence, but it is **not a current release status
page** unless it is dated, scoped, and explicitly superseded or refreshed.

### `AGENTS.md`

`AGENTS.md` contains important engineering rules, but also includes machine- and
owner-specific conventions such as local wrapper paths, external-volume paths,
`/data/projects` sibling checkouts, release machinery, and branch-mirroring
instructions. Those instructions may be correct for the maintainer's environment,
but they are not portable fork contribution requirements by default.

A future cleanup should separate:

- upstream project invariants;
- fork maintenance policy;
- maintainer-machine overlays;
- temporary release/runbook notes.

## Open pull-request state

At review time, this fork has two open internal engineering pull requests:

- PR #1 — DB-resident FTS repair behavior;
- PR #2 — semantic append/resume indexing.

Both branches contain substantial histories and are reported by GitHub as not
currently mergeable against the reviewed `main`. Before treating either as active
release work, the fork needs an explicit decision to rebase/forward-port, split,
replace, or close it. Their prior local test notes remain useful evidence but do
not establish compatibility with the current `main` version.

## Release and installation boundary

Until a fork-specific release policy exists:

- use upstream installation instructions only for upstream releases;
- do not present fork branches or tags as upstream artifacts;
- do not publish fork binaries under upstream version names;
- do not reuse upstream checksums, signatures, attestations, or release notes for
  fork-built artifacts;
- do not claim the full upstream CI matrix passed unless the exact fork commit ran
  it successfully;
- bind every fork artifact to the exact commit, feature set, dependency lock,
  build environment, and source repository;
- keep experimental/internal PR builds clearly labeled and separate from user
  releases.

A fork release should use an explicit version suffix or independent versioning
policy, publish its own checksums/provenance/SBOM, and document its divergence.

## License boundary

The repository license is titled **“MIT License (with OpenAI/Anthropic Rider)”**
and contains additional restrictions that exclude named parties and require the
rider to remain unmodified in distributions and derivative works.

This is not the unmodified MIT license and should not be described merely as
“MIT” or assumed to meet a particular open-source definition without a deliberate
legal/policy review. Fork maintainers must preserve notices and must not make
compatibility, redistribution, contribution, or relicensing claims that have not
been reviewed.

This document is operational guidance, not legal advice.

## Privacy and security boundary

Even a local-first search product can have external effects through installation,
package/model acquisition, optional sync, GitHub releases, dependency downloads,
or user-configured transports. Documentation should distinguish:

- local indexing and search;
- optional model downloads;
- optional export/encryption;
- optional multi-machine sync;
- package/release downloads;
- any network-capable connector or transport.

Security review should cover at least:

- secret-bearing prompts, source, commands, and tool outputs;
- raw-mirror and quarantine retention;
- HTML/export encryption and key handling;
- local database and index permissions;
- symlink/path traversal and untrusted session files;
- hostile markup, terminal controls, and malformed agent records;
- optional network/sync destinations;
- installer/update provenance;
- release artifact integrity and dependency supply chain.

## Recommended fork workflow

```text
identify upstream base
        ↓
record fork-only commits and purpose
        ↓
refresh/rebase in a dedicated branch
        ↓
run focused tests and the supported matrix
        ↓
record exact evidence against one commit
        ↓
submit upstream or retain as explicit fork divergence
        ↓
publish only under a fork-specific release policy
```

Do not mix unrelated upstream synchronization, local environment changes, large
feature branches, and release metadata into one opaque PR.

## Marketing and presentation

The upstream project already has strong branding, an illustration, screenshots,
badges, and extensive product documentation. The fork does not need another logo
or a competing marketing surface merely to exist.

If the fork later becomes a distinct distribution:

- use clearly fork-specific naming and non-confusing artwork;
- state the upstream relationship prominently;
- use synthetic agent histories and repositories in screenshots;
- do not expose real prompts, paths, account data, model usage, raw mirrors, or
  search results;
- do not imply upstream endorsement;
- do not reuse upstream release proof for fork artifacts.

See [`FORK_BACKLOG.md`](FORK_BACKLOG.md) for the durable maintenance and
publication queue.