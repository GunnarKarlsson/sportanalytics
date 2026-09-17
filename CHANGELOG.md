# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-09-17

### Added

- `sportanalytics::cycling`: Effort, FTP protocols, Coggan zones, W/kg,
  2-parameter critical power, power–duration prediction, ACSM relative VO2
  (plus Hawley–Noakes absolute).
- Martin 1998 power–speed, VAM, constant-grade course time, `air_density` /
  `Environment::from_altitude_celsius`.
- Age-factor helper for FTP (trained-endurance decline curve; no official tables).
- `examples/cycling/from_20min.rs`, `examples/cycling/from_tt.rs`.
- Cycling fixtures under `tests/fixtures/cycling/`; running fixtures under
  `tests/fixtures/running/`.
- Examples live under `examples/running/` and `examples/cycling/` (Cargo
  `[[example]]` paths; `cargo run --example <name>` unchanged).

### Changed

- Split `sportanalytics::Error` into shared `Error` plus sport-local
  `running::Error` and `cycling::Error`. Shared covers only
  `NonPositiveTime`, `InvalidHms`, and `AgeOutOfRange { min, max }`.
  Sport helpers return the sport error (`Shared` wraps crate `Error`).
- `AgeOutOfRange` now carries `{ min, max }` (running 5–99, cycling 15–90).
- Cycling variants (`InvalidPower`, `InvalidMass`, `InvalidWork`,
  `InsufficientEfforts`, `DurationOutOfModelRange`, `UnsolvablePowerDuration`,
  `InvalidPhysicsParam`, `UnsolvableSpeed`) live on `cycling::Error`.
- Serde tags for the old flat error enum are gone; `Shared` changes wire shape.
- `prelude` no longer re-exports `Error` (running grab-bag only).
- Shared `Error` display strings are sport-neutral: `duration must be positive`,
  `minutes and seconds must be less than 60`,
  `age is outside the supported range (min–max)`.
- Cycling relative VO2 is attributed to ACSM (`10.8 × W/kg + 7`), not Hawley &
  Noakes; Hawley–Noakes 1992 is the separate absolute L/min equation.
- `predict_power` takes an effort slice; redundant prediction aliases and
  physics/factor constant dumps are no longer re-exported from `cycling`.
- FTP% single-effort mapping requires a protocol duration window (no nearest
  of 5/20/60 heuristics). EightMin (6–10 min) wins over MAP; auto-MAP is
  3–6 min exclusive so 8 min @ 300 W → 270 W, not 250 W.
- Age-factor decline after 35 is linear 0.5%/year (matches documented rate).

## [0.1.1] - 2026-09-17

### Fixed

- With `serde`, `Distance::Custom` no longer leaks labels via `Box::leak` when
  deserializing.

### Changed

- Package `repository` / `homepage` URLs match the published crate name
  (`sportanalytics`).
- Crate package includes `data/age_grade/2025/SOURCE.txt`; raw RunScore table
  files remain git-only provenance.
- README and rustdoc clarifications (age-grade `Result` / ThreeK errors, badges,
  list formatting).

## [0.1.0] - 2026-09-16

### Added

- First release: official USATF MLDR 2025 road age-grade tables and checked-in
  VDOT equation fixtures.
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
- `Pace` and `LengthUnit` for kilometre/mile I/O (`METERS_PER_MILE` = 1609.344).
  Default `Display` is `/km`; use `.display(LengthUnit::Mile)` for `/mi`.
- `Distance::from_km` / `from_miles` / `kilometers` / `miles`, and `FromStr`
  suffixes `mi` / `mile` / `miles` (for example `"8mi"`).
- `RaceTime::from_pace` and `RaceTime::pace`.
- `Error::InvalidPace` for non-finite or non-positive pace constructors.
- Checked-in VDOT fixtures (`tests/fixtures/vdot_*.csv`) plus an ignored
  regenerator; `RIEGEL_EXPONENT` and `PredictedTimes::riegel_exponent`.

### Changed

- **Breaking:** `age_grade` / `age_equivalent` / `age_factor` / `open_standard_secs`
  return `Result`. `age_factor` requires a `Distance` (per-event factors). Ages
  outside 5..=99 error (`AgeOutOfRange`); no clamping. `Distance::ThreeK` age
  grading errors (`UnsupportedAgeGradeDistance`). `AgeGradeResult` adds `table`
  and `age_standard_secs`.
- **Breaking:** `PaceRange` edges are `Pace` instead of raw `f64` seconds/km.
- With `serde`, `Distance::Custom` serializes `{ meters }` only and deserializes
  with label `"custom"` (no `Box::leak`; in-process custom labels are not
  preserved across serde).
- `predict_times` no longer accepts unused `age` / `gender` arguments. Age adjustment is `age_grade` / `age_equivalent`.
- Crate root re-exports only `Error`. Running types live under `sportanalytics::running` (or `sportanalytics::prelude`).
- `RaceTime::from_hms` rejects minutes or seconds ≥ 60 (`InvalidHms`).
- `time_from_vdot` returns `Result` (`UnsolvableTime`) instead of clamping to the 2–12 min/km bisection bracket. `predict_times` and `predict_daniels_and_cameron` do the same.
- `Gender` is documented as WMA/USATF male/female table standards, not a general gender model.
- `PerformanceLevel` documented as informal community bands, not official awards.
- Training-zone docs note that published Daniels *Running Formula* charts will differ by a few seconds/km.
- `running` module rustdoc includes the README function and zone tables so docs.rs stands alone.
- docs.rs builds with `--cfg docsrs` and `doc_cfg`; CI rustdoc fails on warnings.
- `format_pace` is deprecated; prefer `Pace` / `Display`.
- MSRV is 1.71 so CI can resolve current `serde`/`quote`/`serde_json` (they declare rust-version 1.71).
- VDOT fixtures and tighter solver tests; no coefficient change. Docs clarify
  20:00 5K ⇒ VDOT ≈ 49.8 (equation implementation, not printed table grids).
