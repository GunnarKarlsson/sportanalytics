# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-09-16

### Added

- Running module with Daniels–Gilbert VDOT / effective VO2max.
- Race-time prediction via Daniels VDOT inversion, Riegel (`k = 1.06`), and Cameron.
- Daniels-style training zones derived by inverting the oxygen-cost equations at fixed %VDOT (not copied pace tables).
- Compact WMA-style age-grade approximation with open-equivalent and age-equivalent times.
- Typed public surface: `Distance`, `RaceTime`, `Vdot`, `Error`.
- Zero runtime dependencies (`std` only).

### Changed

- `predict_times` no longer accepts unused `age` / `gender` arguments. Age adjustment is `age_grade` / `age_equivalent`.
- Crate root re-exports only `Error`. Running types live under `sportanalytics::running` (or `sportanalytics::prelude`).
