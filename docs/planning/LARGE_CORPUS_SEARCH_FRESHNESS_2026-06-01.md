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

Follow-up: a later live check proved there is another valid source of freshness
truth. The local archive had a completed lexical checkpoint at
`2026-06-01T11:33:55.164Z` while the canonical DB still stored
`meta.last_indexed_at=2026-05-29T23:38:11.493Z`. Status/search metadata now
uses a completed checkpoint for the active DB as the effective lexical
`last_indexed_at` when it is newer than the DB marker, and search-triggered
lexical repairs now refresh DB metadata after rebuilding from the canonical DB.
Direct Doctor and archive-purge rebuild callers do the same.

## Verification

Passed locally with `RUSTUP_TOOLCHAIN=nightly` and
`CARGO_TARGET_DIR=/tmp/cass-check-target`:

- `cargo fmt --check`
- `cargo test --lib indexer::tests::persist_final_index_run_metadata_from_fresh_storage_updates_lexical_resume_marker -- --exact`
- `cargo test --lib indexer::tests::readonly_canonical_force_rebuild_updates_indexed_marker_without_scan_watermark -- --exact`
- `git diff --check`
- `cargo check --all-targets`

Additional follow-up verification:

- `cargo test --lib search::asset_state::tests::lexical_state_uses_completed_checkpoint_timestamp_when_db_marker_lags -- --exact`
- `cargo test --lib search_lexical_self_heal_tests::search_self_heal_rebuilds_missing_lexical_index_from_canonical_db -- --exact`
- `cargo test --lib indexer::tests::persist_final_index_run_metadata_from_fresh_storage_updates_lexical_resume_marker -- --exact`
- `cargo fmt --check`
- `git diff --check`
- `cargo check --all-targets`
- `cargo build --release`

Live read-only verification with the explicit rebuilt binary showed:

- `cass health --json` now reports the completed checkpoint timestamp
  `2026-06-01T11:33:55.164Z` instead of the old DB marker, but remains stale
  under the 300 second health threshold once that checkpoint is older than five
  minutes.
- `cass search ... --robot-meta --timeout 60000` for
  `019e6f57-0e03-75a3-acda-338c6de08aaa` returned successfully in about 6.3s;
  search metadata reported lexical `status=ready`, `fresh=true`,
  `pending_sessions=0`, and the same completed checkpoint timestamp under the
  search surface's 1800 second freshness threshold.
- The canonical DB row is still old until a deliberate maintenance/index run:
  `last_indexed_at=1780097891493`, `last_scan_ts=1780089414052`.

2026-06-01 status follow-up:

- `cass health --json` no longer reports an active Doctor repair owner for the
  old empty lock file: `doctor_summary.active_repair.active=false`.
- `cass status --json` previously still did not return within roughly 50s on
  the live large archive. The slow path was not DB counts; `counts_skipped=true`
  was already pinned. The remaining expensive work was the structured status
  payload's inline Doctor coverage collection, which scanned archive/source/raw
  mirror state intended for `cass doctor --json`.
- `cass status --json` now keeps checked inline coverage only for small regular
  archive DBs. Large or malformed DB paths use the fast coverage summary
  (`coverage_risk.status=unchecked_fast_health`,
  `doctor_summary.coverage_source.source=status-fast-state`) and point operators
  at `cass doctor --json` for the expensive ledger.
- Live read-only verification with the explicit rebuilt binary against
  `/Users/seitz/Library/Application Support/com.coding-agent-search.coding-agent-search`
  returned in about 1.4s. The payload reported `status=unhealthy` only because
  the lexical index was older than the 300s status threshold; coverage was
  deliberately `unchecked`, remote archive sync was not archive-checked, and DB
  counts remained skipped.

`rch` was not available in this local environment, so the broad compile check
was run locally instead of via remote compute.

## Remaining Work

- Run a deliberate maintenance/index command against the live archive when it
  is acceptable to mutate CASS derived metadata.
- Re-run `cass health --json` and `cass search --robot-meta` against the same
  explicit binary and data dir; their `last_indexed_at`/checkpoint view should
  converge.
- If exact source/raw-mirror/archive coverage is needed on the large corpus, use
  `cass doctor --json`; `cass status --json` intentionally stays a fast
  readiness surface.
- Re-test the target ChatGPT import corpus
  `019e6f57-0e03-75a3-acda-338c6de08aaa` and distinguish weak rollout-reference
  hits from actual imported ChatGPT conversation hits.
- Separately diagnose any remaining default/hybrid pre-search timeout. That is
  a search startup/cost issue, not the health-vs-search metadata mismatch
  fixed here.
