# sportanalytics

[![Crates.io](https://img.shields.io/crates/v/sportanalytics.svg)](https://crates.io/crates/sportanalytics)
[![Docs.rs](https://docs.rs/sportanalytics/badge.svg)](https://docs.rs/sportanalytics)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/GunnarKarlsson/sports-analytics/actions/workflows/ci.yml/badge.svg)](https://github.com/GunnarKarlsson/sports-analytics/actions)

Running analytics in Rust: Daniels–Gilbert VDOT (effective VO2max), race-time
prediction, training zones, and a compact WMA-style age-grade model.

The published crate name is **`sportanalytics`**. This repository is
`sports-analytics`. The layout is modular so other sports can be added later;
0.1 only ships **running**.

## Install

```bash
cargo add sportanalytics
```

```toml
[dependencies]
sportanalytics = "0.1"
# Optional JSON/API support (off by default; zero deps otherwise):
# sportanalytics = { version = "0.1", features = ["serde"] }
```

```rust
use sportanalytics::running::*;
```

The same items are also available from `sportanalytics::prelude`. The crate root
re-exports only `sportanalytics::Error`.

## Quick start

```rust
use sportanalytics::running::{
    format_pace, predict_times, training_zones, vdot, Distance, PredictionModel, RaceTime,
};

fn main() -> Result<(), sportanalytics::Error> {
    let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0)?;
    println!("VDOT {:.1}", vdot(five).value());

    let pred = predict_times(five, PredictionModel::DanielsVdot)?;
    println!(
        "HM {}  FM {}",
        pred.formatted(Distance::HalfMarathon)?,
        pred.formatted(Distance::Marathon)?
    );

    let z = training_zones(five);
    println!(
        "E {}–{}  T {}–{}",
        format_pace(z.easy.easy_end),
        format_pace(z.easy.hard_end),
        format_pace(z.threshold.easy_end),
        format_pace(z.threshold.hard_end)
    );

    Ok(())
}
```

A 20:00 5K is about VDOT 50.

Runnable programs (also listed on docs.rs):

```text
cargo run --example from_5k      # VDOT, predictions, training zones from a 20:00 5K
cargo run --example age_grade    # 42-year-old 5K
```

## Features

| Function | What it uses |
|---|---|
| `vdot` / `vo2max_from_races` | Daniels–Gilbert 1979 VDOT (effective VO2max) |
| `predict_times` | Daniels invert, Riegel `T2 = T1 * (D2/D1)^1.06`, or Cameron |
| `predict_daniels_and_cameron` | Daniels and Cameron in one call |
| `training_zones` / `training_zones_from_vdot` | Daniels %VDOT pace bands (E/M/T/I/R) |
| `age_grade` / `age_equivalent` | Compact WMA-style age factors + open standards |

Age and gender are used only by age grading. Predict first, then pass a predicted
time into `age_equivalent` if you need an age-adjusted figure.

Named distances are 3K, 5K, 10K, half marathon, and marathon; other lengths use
`Distance::from_meters` / `Distance::custom`. Training zones are inverted from
the oxygen-cost equations at fixed %VDOT (E 59–74, M 75–84, T 83–88, I 95–100,
R ~105–110). This crate does not ship Daniels’ copyrighted lookup tables.

Full types and formulas: [docs.rs/sportanalytics](https://docs.rs/sportanalytics).

## MSRV

Rust **1.71** (edition 2021).

## Accuracy / non-goals

- VDOT is *effective* VO2max (economy included), not a lab test.
- Age factors are a WMA-style *approximation*, not official World Masters Athletics or USATF scoring tables. Open 5K–marathon times are 2025-era road world records (USATF MLDR 2025 open standards), so percentages can run a few points high versus older championship tables.
- Predictions assume a flat, all-out effort and similar training specificity.
- Published Daniels *Running Formula* charts will differ by a few seconds/km.
- Default builds have no crate dependencies (`std` only). Enable `serde` for `Serialize`/`Deserialize`.
- 0.1 does not include cycling, swimming, or other sports. Add those as sibling modules later; do not dump new sports onto the crate root.

## Attribution

The running module implements published equations. It is not copied from another
crate or from copyrighted pace tables.

- **VDOT / equivalents / training intensities:** Jack Daniels and Jimmy Gilbert, *Oxygen Power* (1979) — oxygen cost of running and sustainable %VO2max versus duration. Training zones invert those equations at fixed % of VDOT.
- **Riegel:** Pete Riegel (1977, *Runner’s World*; 1981, *American Scientist*) — `T2 = T1 * (D2/D1)^1.06`.
- **Cameron:** David Cameron’s road-race fit — `T2 = T1 * (D2/D1) * f(D1)/f(D2)`.
- **Age grading:** compact interpolated factors in the spirit of WMA/USATF road age grading. Open 5K–marathon times are 2025-era World Athletics road world records (USATF MLDR 2025 open standards compiled by Alan Jones). Percentages are estimates, not championship scores.

## License

MIT
