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

### Changed

- **Breaking:** `PaceRange` edges are `Pace` instead of raw `f64` seconds/km.
- `format_pace` is deprecated; prefer `Pace` / `Display`.
- MSRV is 1.71 so CI can resolve current `serde`/`quote`/`serde_json` (they declare rust-version 1.71).

## [0.1.0] - 2026-09-16

### Added

- Running module with Daniels–Gilbert VDOT / effective VO2max.
- Race-time prediction via Daniels VDOT inversion, Riegel (`k = 1.06`), and Cameron.
- Daniels-style training zones derived by inverting the oxygen-cost equations at fixed %VDOT (not copied pace tables).
- Compact WMA-style age-grade approximation with open-equivalent and age-equivalent times.
- Typed public surface: `Distance`, `RaceTime`, `Vdot`, `Error`.
- Custom race distances via `Distance::from_meters` / `Distance::custom`.
- `Copy` / `Display` on core result types; `FromStr` for `Distance` (`"5K"`, `"HM"`, `"marathon"`).
- Optional `serde` feature for serializing public types. Default builds stay dependency-free.
- Zero runtime dependencies (`std` only) unless `serde` is enabled.
- Rustdoc examples on public running functions (`vdot`, `predict_times`, `training_zones`, `age_grade`, and the rest of the module API).
- `examples/from_5k.rs` (VDOT, predictions, zones) and `examples/age_grade.rs` (42-year-old 5K).

### Changed

- `predict_times` no longer accepts unused `age` / `gender` arguments. Age adjustment is `age_grade` / `age_equivalent`.
- Crate root re-exports only `Error`. Running types live under `sportanalytics::running` (or `sportanalytics::prelude`).
- `RaceTime::from_hms` rejects minutes or seconds ≥ 60 (`InvalidHms`).
- `time_from_vdot` returns `Result` (`UnsolvableTime`) instead of clamping to the 2–12 min/km bisection bracket. `predict_times` and `predict_daniels_and_cameron` do the same.
- `Gender` is documented as WMA male/female table standards, not a general gender model.
- `open_standard_secs` documents 5K–marathon times as 2025-era road world records (USATF MLDR 2025 open standards), so percentages can run high versus older championship tables.
- Training-zone docs note that published Daniels *Running Formula* charts will differ by a few seconds/km.
- `running` module rustdoc includes the README function and zone tables so docs.rs stands alone.
- docs.rs builds with `--cfg docsrs` and `doc_cfg`; CI rustdoc fails on warnings.
