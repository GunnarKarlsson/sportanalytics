# Contributing

Crate name: `sportanalytics`. Repository: `sportanalytics`.

Bug reports and feature ideas: use the GitHub issue templates. Pull requests
are welcome for tests, docs, and formula/API work that matches the crate’s
scope (running + cycling sibling modules in 0.2.0; further sports as siblings later).
Do not re-export sports from the crate root. Prelude stays running-only.

This project follows the [Rust Code of Conduct](CODE_OF_CONDUCT.md).

## Quality gate

Match CI before opening a PR (`RUSTFLAGS` / `RUSTDOCFLAGS` are `-D warnings`):

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --doc
cargo doc --no-deps --all-features --document-private-items
```

MSRV is **1.71** (`rust-version` in `Cargo.toml`). CI also runs tests on that
toolchain. Do not raise MSRV without a reason in the PR.

## Code

- Every formula change needs a test (table-driven when there is a known point).
- Keep default builds dependency-free. New runtime deps or features need an
  issue first.
- Do not copy copyrighted Daniels pace charts. Zones stay inverted from the
  published oxygen-cost equations.
- Do not re-export new sports from the crate root. Add `src/<sport>/` and
  `sportanalytics::<sport>`.
- Sport failures live in `src/<sport>/error.rs` as that sport’s complete enum
  (`running::Error`, `cycling::Error`, …). Do not add a crate-root error. Duplicate
  a variant name if two sports need the same idea (`NonPositiveTime`,
  `AgeOutOfRange`). Promote to a shared type only if you later have a real shared
  *module* both sports use — not before.
- User-visible API or behavior changes: a `[Unreleased]` note in `CHANGELOG.md`.
- Age grading: the published crate ships the generated table
  (`src/running/age_grade/mldr_2025.rs`); raw `data/age_grade/2025/AgeGrade.*`
  files are provenance only. Regenerate via `scripts/gen_age_grade.py` — do not
  hand-edit `mldr_2025.rs`. `SOURCE.txt` is packaged for citation; the RunScore
  files are not.

## Release

Tag `vX.Y.Z` must match `Cargo.toml`. Pushing that tag creates a GitHub Release
whose notes are the matching `CHANGELOG.md` section. Do the first
`cargo publish` by hand; do not put a crates.io token in GitHub Actions yet.

1. `Cargo.toml` version is `X.Y.Z`.
2. Move `[Unreleased]` items into `## [X.Y.Z] - YYYY-MM-DD` in `CHANGELOG.md`.
3. `cargo publish --dry-run`, then `cargo publish`.
4. `git tag vX.Y.Z && git push origin vX.Y.Z`
