# sportanalytics

[![Crates.io](https://img.shields.io/crates/v/sportanalytics.svg)](https://crates.io/crates/sportanalytics)
[![Docs.rs](https://docs.rs/sportanalytics/badge.svg)](https://docs.rs/sportanalytics)
[![MSRV](https://img.shields.io/badge/MSRV-1.71+-blue.svg)](https://blog.rust-lang.org/2023/07/13/Rust-1.71.0/)
[![Rust](https://img.shields.io/badge/Rust-edition%202021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/GunnarKarlsson/sports-analytics/actions/workflows/ci.yml/badge.svg)](https://github.com/GunnarKarlsson/sports-analytics/actions)

Running analytics in Rust: Daniels–Gilbert VDOT (effective VO2max), race-time
prediction, training zones, and USATF MLDR 2025 road age grading.

The published crate name is **`sportanalytics`**. This repository is
`sports-analytics`. The layout is modular so other sports can be added later;
0.1 only ships **running**.

## Install

```bash
cargo add sportanalytics
```

Zero deps core lib:
```toml
[dependencies]
sportanalytics = "0.1" 
```

Add serde dep for JSON parsing:
```toml
[dependencies]
# sportanalytics = { version = "0.1", features = ["serde"] }
```

Use:
```rust
use sportanalytics::running::*;
```

The same items are also available from `sportanalytics::prelude`. 
The crate root re-exports only `sportanalytics::Error`.

## Quick start

```rust
use sportanalytics::running::{
    predict_times, training_zones, vdot, Distance, LengthUnit, PredictionModel, RaceTime,
};

fn main() -> Result<(), sportanalytics::Error> {
    // Use a 5K race time of 00:20:00 as example:
    let five_k_race_time = RaceTime::from_hms(Distance::FiveK, 0, 20, 0)?;
    println!("VDOT {:.1}", vdot(five_k_race_time).value());

    let prediction = predict_times(five_k_race_time, PredictionModel::DanielsVdot)?;
    println!(
        "HM {}  FM {}",
        prediction.formatted(Distance::HalfMarathon)?,
        prediction.formatted(Distance::Marathon)?
    );

    let zones = training_zones(five_k_race_time);
    println!("E {}  T {}", zones.easy, zones.threshold);
    println!("E miles {}", zones.easy.display(LengthUnit::Mile));

    Ok(())
}
```

- Pace and zone `Display` default to **`/km`**.
- For miles, call `.display(LengthUnit::Mile)`.
- Distances accept kilometres or international miles (`Distance::from_km`,
  `Distance::from_miles`, or `"8mi".parse::<Distance>()`). Internal math is
  in metres and seconds.

Runnable example programs (also listed on docs.rs):

```text
cargo run --example from_5k      # VDOT, predictions, training zones from a 20:00 5K
cargo run --example age_grade    # 42-year-old 5K
```

## Features

| Function | What data or formula it uses |
|---|---|
| `vdot` / `vo2max_from_races` | Daniels–Gilbert 1979 VDOT (effective VO2max) |
| `predict_times` | Daniels invert, Riegel, or Cameron |
| `predict_daniels_and_cameron` | Daniels and Cameron in one call |
| `training_zones` / `training_zones_from_vdot` | Daniels %VDOT pace bands (E/M/T/I/R) |
| `age_grade` / `age_equivalent` | USATF MLDR 2025 single-year road tables (CC0); both return `Result` |

- Age and gender are used only by age grading.
- If you need an age-adjusted figure, run the prediction first, then pass a
  predicted time into `age_equivalent`.
- Ages outside 5..=99 and unsupported distances result in error: use `?` or
  `.unwrap()`.

### Age grading

Example:

```rust
use sportanalytics::running::{
    age_equivalent, age_grade, Distance, Gender, RaceTime,
};

let race_time = RaceTime::from_hms(Distance::FiveK, 0, 20, 0)?;
let age_grade = age_grade(race_time, 42, Gender::Male, Some(25))?;
println!(
    "{:.1}% {} | open eq {} | table {:?}",
    age_grade.percent,
    age_grade.level.label(),
    age_grade.open_equivalent_hms(),
    age_grade.table
);

let as_25 = age_equivalent(race_time, 42, Gender::Male, 25)?;
println!("equivalent at 25: {as_25:.1}s");
   
```

- Named distances are 3K, 5K, 10K, half marathon, and marathon; other lengths use
  `Distance::from_meters` / `Distance::from_km` / `Distance::from_miles` /
  `Distance::custom`, or parse strings such as `"8k"` and `"8mi"`.
- Training zones are inverted from the oxygen-cost equations at fixed %VDOT
  (E 0.59–0.74, M 0.75–0.84, T 0.83–0.88, I 0.95–1.00, R 1.05–1.10).
- Zone edges are typed `Pace` values.
- This crate does not ship Daniels’ copyrighted lookup tables.

Full types and formulas: [docs.rs/sportanalytics](https://docs.rs/sportanalytics).


## Accuracy / non-goals

### Formulas
- VDOT is *effective* VO2max (economy included). This crate implements the
  Daniels–Gilbert *Oxygen Power* (1979) equations, not the copyrighted printed
  lookup tables.
- Daniels predictions are VDOT-equivalent performances. Riegel is a power law
  (`k = 1.06`). Cameron is a distance-weighted road fit. None include hills,
  heat, or wind.
- Age grading looks up the official **USATF MLDR 2025** road tables
  (approved 2025-01-10). Ages **5–99** (`AgeOutOfRange` otherwise). Off-grid
  distances interpolate age standards in log-distance between neighbouring
  official events (Jones 2025). Road 3K returns `UnsupportedAgeGradeDistance`.
  This crate uses the same published table as the Howard Grubb MLDR 2025
  calculator.
- Predictions assume a flat, all-out effort and similar training specificity.
- Published Daniels *Running Formula* charts will differ by a few seconds/km
  from equation output.

### Builds
- Default builds have no crate dependencies (`std` only). Enable `serde` for
  `Serialize`/`Deserialize`.
- Crate 0.1.0 does not include cycling, swimming, or other sports. Add those as
  sibling modules later; do not dump new sports onto the crate root.

## Attribution

The running module implements published equations. It is not copied from another
crate or from copyrighted pace tables.

- **VDOT / equivalents / training intensities:** Jack Daniels and Jimmy Gilbert,
  *Oxygen Power* (1979): oxygen cost of running and sustainable %VO2max versus
  duration. This crate implements those equations (not copyrighted printed pace
  grids). Training zones invert the oxygen-cost curve at fixed % of VDOT
  (E 0.59–0.74, M 0.75–0.84, T 0.83–0.88, I 0.95–1.00, R 1.05–1.10).
- **Riegel:** Pete Riegel (1977, *Runner’s World*; 1981, *American Scientist*) —
  `T2 = T1 * (D2/D1)^1.06`. The exponent `1.06` is the published default. It is
  not fitted per athlete.
- **Cameron:** David Cameron’s published road-race fit (commonly dated late
  1990s): `T2 = T1 * (D2/D1) * f(D1)/f(D2)` with
  `f(x) = 13.49681 - 0.000030363 x + 835.7114 / x^0.7905` (`x` in metres).
- **Age grading:** USATF Masters Long Distance Running (MLDR) 2025 road tables
  by Alan Jones and Tom Bernhard (approved 2025-01-10). Source:
  [AlanLyttonJones/Age-Grade-Tables](https://github.com/AlanLyttonJones/Age-Grade-Tables)
  (`2025 Files/AgeGrade.zip`). Table data is **CC0-1.0**; the crate code is MIT.

## MSRV

Rust **1.71** (edition 2021).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). This project follows the
[Rust Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT for crate code. Embedded USATF MLDR 2025 age-grade table data is CC0-1.0
(Alan Jones / Tom Bernhard).
