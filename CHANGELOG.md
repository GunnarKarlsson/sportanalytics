# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Pace` and `LengthUnit` for kilometre/mile I/O (`METERS_PER_MILE` = 1609.344).
  Default `Display` is `/km`; use `.display(LengthUnit::Mile)` for `/mi`.
- `Distance::from_km` / `from_miles` / `kilometers` / `miles`, and `FromStr`
  suffixes `mi` / `mile` / `miles` (for example `"8mi"`).
- `RaceTime::from_pace` and `RaceTime::pace`.
- `Error::InvalidPace` for non-finite or non-positive pace constructors.
- Checked-in VDOT fixtures (`tests/fixtures/vdot_*.csv`) plus an ignored
  regenerator; `RIEGEL_EXPONENT` and `PredictedTimes::riegel_exponent`.

### Changed

- **Breaking:** `PaceRange` edges are `Pace` instead of raw `f64` seconds/km.
- `format_pace` is deprecated; prefer `Pace` / `Display`.
- MSRV is 1.71 so CI can resolve current `serde`/`quote`/`serde_json` (they declare rust-version 1.71).
- VDOT fixtures and tighter solver tests; no coefficient change. Docs clarify
  20:00 5K ⇒ VDOT ≈ 49.8 (equation implementation, not printed table grids).

## [0.1.0] - 2026-09-16

### Added

- Running module with Daniels–Gilbert VDOT / effective VO2max.
- Race-time prediction via Daniels VDOT inversion, Riegel (`k = 1.06`), and Cameron.
- Daniels-style training zones derived by inverting the oxygen-cost equations at fixed %VDOT (not copied pace tables).
- Official USATF MLDR 2025 road age grading (Alan Jones / Tom Bernhard, CC0) with
  open-equivalent and age-equivalent times; embedded generated table plus raw
  RunScore provenance under `data/age_grade/2025/`.
- `AgeGradeTable`, `age_grade_with` / `age_factor_with` / `age_equivalent_with` /
  `open_standard_secs_with`.
- `Error::AgeOutOfRange` and `Error::UnsupportedAgeGradeDistance`.
- Typed public surface: `Distance`, `RaceTime`, `Vdot`, `Error`.
- Custom race distances via `Distance::from_meters` / `Distance::custom`.
- `Copy` / `Display` on core result types; `FromStr` for `Distance` (`"5K"`, `"HM"`, `"marathon"`).
- Optional `serde` feature for serializing public types. Default builds stay dependency-free.
- Zero runtime dependencies (`std` only) unless `serde` is enabled.
- Rustdoc examples on public running functions (`vdot`, `predict_times`, `training_zones`, `age_grade`, and the rest of the module API).
- `examples/from_5k.rs` (VDOT, predictions, zones) and `examples/age_grade.rs` (42-year-old 5K).

### Changed

- **Breaking:** `age_grade` / `age_equivalent` / `age_factor` / `open_standard_secs`
  return `Result`. `age_factor` requires a `Distance` (per-event factors). Ages
  outside 5..=99 error (`AgeOutOfRange`); no clamping. `Distance::ThreeK` age
  grading errors (`UnsupportedAgeGradeDistance`). `AgeGradeResult` adds `table`
  and `age_standard_secs`.
- `predict_times` no longer accepts unused `age` / `gender` arguments. Age adjustment is `age_grade` / `age_equivalent`.
- Crate root re-exports only `Error`. Running types live under `sportanalytics::running` (or `sportanalytics::prelude`).
- `RaceTime::from_hms` rejects minutes or seconds ≥ 60 (`InvalidHms`).
- `time_from_vdot` returns `Result` (`UnsolvableTime`) instead of clamping to the 2–12 min/km bisection bracket. `predict_times` and `predict_daniels_and_cameron` do the same.
- `Gender` is documented as WMA/USATF male/female table standards, not a general gender model.
- `PerformanceLevel` documented as informal community bands, not official awards.
- Training-zone docs note that published Daniels *Running Formula* charts will differ by a few seconds/km.
- `running` module rustdoc includes the README function and zone tables so docs.rs stands alone.
- docs.rs builds with `--cfg docsrs` and `doc_cfg`; CI rustdoc fails on warnings.
