# sportanalytics

[![Crates.io](https://img.shields.io/crates/v/sportanalytics.svg)](https://crates.io/crates/sportanalytics)
[![Docs.rs](https://docs.rs/sportanalytics/badge.svg)](https://docs.rs/sportanalytics)
[![MSRV](https://img.shields.io/badge/MSRV-1.71+-blue.svg)](https://blog.rust-lang.org/2023/07/13/Rust-1.71.0/)
[![Rust](https://img.shields.io/badge/Rust-edition%202021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/GunnarKarlsson/sportanalytics/actions/workflows/ci.yml/badge.svg)](https://github.com/GunnarKarlsson/sportanalytics/actions)

Running and cycling analytics in Rust. 

**0.2.0** ships Daniels–Gilbert VDOT, race-time prediction, training zones, and USATF MLDR 2025 road age grading for running, plus FTP, critical power, Coggan zones, ACSM / Hawley–Noakes VO2, and Martin road power–speed for cycling.

The published crate name is **`sportanalytics`**. Sports live in sibling
modules (`sportanalytics::running`, `sportanalytics::cycling`); the crate root
does not re-export `Error`. Sport helpers return `running::Error` /
`cycling::Error`.

## 📦 Install

```bash
cargo add sportanalytics
```

Default (no extra dependencies):
```toml
[dependencies]
sportanalytics = "0.2.0"
```

Optional JSON support:
```toml
[dependencies]
sportanalytics = { version = "0.2.0", features = ["serde"] }
```

Then in code:
```rust
use sportanalytics::running::*;
// or: use sportanalytics::cycling::*;
```

- Running items are also available from `sportanalytics::prelude` (running-only).
- Sport functions return `running::Error` / `cycling::Error` (no crate-root
  `Error`).
- Mix sports with `Box<dyn std::error::Error>`.

## 🏃‍➡️ Running Metrics

```rust
use sportanalytics::running::{
    predict_times, training_zones, vdot, Distance, LengthUnit, PredictionModel, RaceTime,
};

fn main() -> Result<(), sportanalytics::running::Error> {
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

Run Examples:

```text
cargo run --example from_5k      # examples/running/from_5k.rs
cargo run --example age_grade    # examples/running/age_grade.rs
```

### Features

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

fn main() -> Result<(), sportanalytics::running::Error> {
    let race_time = RaceTime::from_hms(Distance::FiveK, 0, 20, 0)?;
    let ag = age_grade(race_time, 42, Gender::Male, Some(25))?;
    println!(
        "{:.1}% {} | open eq {} | table {:?}",
        ag.percent,
        ag.level.label(),
        ag.open_equivalent_hms(),
        ag.table
    );

    let as_25 = age_equivalent(race_time, 42, Gender::Male, 25)?;
    println!("equivalent at 25: {as_25:.1}s");
    Ok(())
}
```

- Named distances are 3K, 5K, 10K, half marathon, and marathon; other lengths use
  `Distance::from_meters` / `Distance::from_km` / `Distance::from_miles` /
  `Distance::custom`, or parse strings such as `"8k"` and `"8mi"`.
- Training zones are inverted from the oxygen-cost equations at fixed %VDOT
  (E 0.59–0.74, M 0.75–0.84, T 0.83–0.88, I 0.95–1.00, R 1.05–1.10).
- Zone edges are typed `Pace` values.
- This crate does not ship Daniels’ copyrighted lookup tables.

Full types and formulas: [docs.rs/sportanalytics](https://docs.rs/sportanalytics).

### Accuracy / non-goals

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

## 🚴‍♀️ Cycling Metrics

Example:

```rust
use sportanalytics::cycling::{
    critical_power, ftp_from_20min, predict_power_from_cp, training_zones, Effort,
};
use std::time::Duration;

fn main() -> Result<(), sportanalytics::cycling::Error> {
    let twenty = Effort::from_watts_secs(280.0, 20.0 * 60.0)?;
    let five = Effort::from_watts_secs(340.0, 5.0 * 60.0)?;

    let ftp = ftp_from_20min(twenty)?;
    let zones = training_zones(ftp);
    let cp = critical_power(&[five, twenty])?;
    let hour = predict_power_from_cp(cp.model, Duration::from_secs(3600))?;

    println!("FTP {} W, CP {:.0} W, 60 min {:.0} W", ftp.watts(), cp.model.cp.watts(), hour.watts());
    println!("Z2 {}", zones.endurance);
    Ok(())
}
```

Run Examples:

```text
cargo run --example from_20min   # examples/cycling/from_20min.rs
cargo run --example from_tt      # examples/cycling/from_tt.rs
```

### Road speed from power

Example:

```rust
use sportanalytics::cycling::{speed_for_power, time_for_distance, Environment, Mass, Power, RiderBike};

fn main() -> Result<(), sportanalytics::cycling::Error> {
    let rider = RiderBike::road_default(Mass::from_kg(83.0)?);
    let env = Environment::flat_calm_sea_level();
    let v = speed_for_power(rider, env, Power::new(250.0)?)?;
    let t = time_for_distance(rider, env, Power::new(250.0)?, 40_000.0)?;
    println!("{:.1} km/h — 40 km in {:.0}s", v * 3.6, t.as_secs_f64());
    Ok(())
}
```

### Features

| Function | What data or formula it uses |
|---|---|
| `ftp_from_20min` / `ftp_from_protocol` | Operational FTP from field protocols |
| `critical_power` | Two-parameter CP + W′ (~2–30 min; hour from short pairs biased high) |
| `predict_power` / `predict_power_from_cp` | CP, %FTP curve, or power Riegel |
| `training_zones` | Coggan Z1–Z7 + sweet spot (%FTP) |
| `estimated_vo2max` | ACSM MAP → relative VO2 field estimate |
| `power_for_speed` / `speed_for_power` | Martin et al. 1998 power balance |
| `age_equivalent_ftp` | Trained-endurance decline curve (not official tables) |

FTP is an operational training anchor, not laboratory lactate threshold.
Estimated VO2 is a field estimate, not gas analysis. Age helpers use ages
**15..=90** and are not USATF/VTTA age grading.

## 🛠️ Builds

- Default builds have no crate dependencies (`std` only). Enable `serde` for
  `Serialize`/`Deserialize`.
- Crate 0.2.0 ships **running** and **cycling**. Further sports should be sibling
  modules; do not dump new sports onto the crate root or into `prelude`.

## 📋 Attribution

The running and cycling modules implement published equations. They are not
copied from another crate or from copyrighted pace/power charts.

### Running

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

### Cycling

- **FTP protocols / Coggan %FTP zones:** public coaching conventions (operational
  threshold and training bands), not laboratory LT and not copyrighted power-profile
  charts.
- **Critical power:** two-parameter `P = CP + W′/t` linear fit (valid ~2–30 min;
  hour power from a short+medium pair is biased high).
- **VO2 estimate:** ACSM relative `10.8 × W/kg + 7`; optional Hawley & Noakes
  1992 absolute `0.01141 × Wpeak + 0.435` L/min.
- **Road power–speed:** Martin et al. 1998 cycling power balance (CdA, Crr, grade,
  wind, drivetrain η); steady-state only (no KE, drafting, or coasting).
- **Age factor:** trained-endurance decline approximation (linear ~0.5%/year after
  35; Tanaka & Seals-style); sex unused; not VTTA/CTT/WMA tables.

## 🦀 MSRV

Rust **1.71** (edition 2021).

## ✍ Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). This project follows the
[Rust Code of Conduct](CODE_OF_CONDUCT.md).

## 📜 License

MIT for crate code. Embedded USATF MLDR 2025 age-grade table data is CC0-1.0
(Alan Jones / Tom Bernhard).
