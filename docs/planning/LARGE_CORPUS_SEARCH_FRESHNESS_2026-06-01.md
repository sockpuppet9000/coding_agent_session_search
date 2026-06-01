# Large Corpus Search Freshness Follow-up, 2026-06-01

Status: code fix landed in this work block, pending a future real index run on
the live archive.

## Context

Three earlier large-corpus blockers are separate from this note:

- DB-resident SQLite FTS is no longer a hard prerequisite for ordinary
  CLI/search behavior; Tantivy/frankensearch is the derived lexical search
  surface.
- Semantic backfill can append/resume across DB fingerprint changes.
- TUI/recent-conversation startup paths no longer depend on an unindexed
  message-date browse query.

The 2026-06-01 recheck then found a different problem on the local large
archive. `cass health --json` reported a fresh-ish completed lexical rebuild
checkpoint from the index side, while `cass search --robot-meta` reported an
older `meta.last_indexed_at` value from the canonical DB. That made health and
search disagree about freshness even though the published Tantivy index existed
and lexical search could answer.

## Root Cause

DB-authoritative readonly lexical rebuild paths can complete and publish a new
Tantivy index without ever opening the canonical DB writable in the normal final
metadata path. In those early-return paths, the derived lexical index is
refreshed, but DB metadata such as `meta.last_indexed_at` remains stale.

The affected paths are:

- readonly force rebuild from an existing populated canonical DB;
- readonly resume of an incomplete non-resumable lexical rebuild.

## Fix

`src/indexer/mod.rs` now refreshes final index-run metadata after both readonly
DB-authoritative rebuild paths complete. The helper opens a fresh short-timeout
writer only for status metadata, reuses the existing concurrent/ephemeral writer
retry behavior, and preserves `last_scan_ts` for lexical-only rebuilds so source
scan watermarks are not advanced by derived-index maintenance.

This fixes future runs. It does not mutate an existing live archive until a
normal index/maintenance command is run deliberately.

## Verification

Passed locally with `RUSTUP_TOOLCHAIN=nightly` and
`CARGO_TARGET_DIR=/tmp/cass-check-target`:

- `cargo fmt --check`
- `cargo test --lib indexer::tests::persist_final_index_run_metadata_from_fresh_storage_updates_lexical_resume_marker -- --exact`
- `cargo test --lib indexer::tests::readonly_canonical_force_rebuild_updates_indexed_marker_without_scan_watermark -- --exact`
- `git diff --check`
- `cargo check --all-targets`

`rch` was not available in this local environment, so the broad compile check
was run locally instead of via remote compute.

## Remaining Work

- Run a deliberate maintenance/index command against the live archive when it
  is acceptable to mutate CASS derived metadata.
- Re-run `cass health --json` and `cass search --robot-meta` against the same
  explicit binary and data dir; their `last_indexed_at`/checkpoint view should
  converge.
- Re-test the target ChatGPT import corpus
  `019e6f57-0e03-75a3-acda-338c6de08aaa` and distinguish weak rollout-reference
  hits from actual imported ChatGPT conversation hits.
- Separately diagnose any remaining default/hybrid pre-search timeout. That is
  a search startup/cost issue, not the health-vs-search metadata mismatch
  fixed here.
