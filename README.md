# sportanalytics

Crate name: `sportanalytics`. Repository: `sports-analytics`.

Multi-sport analytics crate. First module: **running**.

Std-only helpers for widely used running formulas: Daniels–Gilbert VDOT (effective VO2max), race-time prediction (Daniels, Riegel, Cameron), Daniels training zones, and a compact WMA-style age-grade model.

Add the crate to `Cargo.toml`:

```toml
[dependencies]
sportanalytics = "0.1"
# Optional JSON/API support (off by default; zero deps otherwise):
# sportanalytics = { version = "0.1", features = ["serde"] }
```

Then:

```rust
use sportanalytics::running::*;
```

The same items are also available from `sportanalytics::prelude`. The crate root re-exports only `sportanalytics::Error`.

Runnable programs (also listed on docs.rs):

```text
cargo run --example from_5k      # VDOT, predictions, training zones from a 20:00 5K
cargo run --example age_grade    # 42-year-old 5K
```

## Running

| Function | What it uses |
|---|---|
| `vdot` / `vo2max_from_races` | Daniels–Gilbert 1979 VDOT (effective VO2max) |
| `predict_times` | Daniels invert, Riegel `T2=T1*(D2/D1)^1.06`, or Cameron |
| `predict_daniels_and_cameron` | Daniels and Cameron in one call |
| `training_zones` / `training_zones_from_vdot` | Daniels %VDOT pace bands (E/M/T/I/R) |
| `age_grade` / `age_equivalent` | Compact WMA-style age factors + open standards |

Age and gender are used only by age grading. Daniels, Riegel, and Cameron predictions do not take them. Predict first, then pass a predicted time into `age_equivalent` if you need an age-adjusted figure.

### Types

**`Distance`** — `ThreeK`, `FiveK`, `TenK`, `HalfMarathon`, `Marathon`, or `Distance::from_meters` / `Distance::custom` for other lengths (1500 m, 8K, 10 mile, …). Metres come from `Distance::meters()` (half marathon is 21,097.5 m; marathon is 42,195 m).

**`RaceTime`** — a distance plus a positive finish time. Construct with `from_hms`, `from_secs`, or `new` (`std::time::Duration`). `from_hms` requires minutes and seconds `< 60`. Invalid times return `Error::NonPositiveTime` or `Error::InvalidHms`.

**`Vdot`** — newtype around a positive finite Daniels VDOT. Inner math is still `f64`; the wrapper is used at API edges (`vdot`, `time_from_vdot`, `Vo2Estimate`, `training_zones_from_vdot`) so a VDOT is not confused with seconds or m/min. Cameron/Riegel times are *not* VDOT values.

**`Error`** — `NonPositiveTime`, `EmptyRaces`, `InvalidVdot`, `InvalidDistance`, `UnrecognizedDistance`, `InvalidHms`, `UnsolvableTime`. Implements `std::error::Error`.

A 20:00 5K is about VDOT 50. Daniels inversion searches **2–12 min/km** and returns `Error::UnsolvableTime` when there is no root. See `examples/from_5k.rs` for VDOT, `predict_times`, and `training_zones`.

### Race prediction

- **DanielsVdot** (recommended): invert Daniels–Gilbert so every distance is an equivalent VDOT.
- **Riegel**: `T2 = T1 * (D2/D1)^1.06` (optional `riegel_with_exponent` if you want another `k`).
- **Cameron**: `T2 = T1 * (D2/D1) * f(D1)/f(D2)` with Cameron’s `f(x)` in metres.

### Training zones

Daniels maps VDOT to paces at fixed %VO2max. Cameron/Riegel times do not define zones — use the VDOT from the race (or mean/best of several).

| Zone | % of VDOT | Use |
|---|---|---|
| Easy (E) | 59–74% | easy / long run |
| Marathon (M) | 75–84% | marathon pace |
| Threshold (T) | 83–88% | tempo / cruise intervals |
| Interval (I) | 95–100% | 3–5 min VO2 reps |
| Repetition (R) | ~105–110% | short fast reps |

Each `PaceRange` has `easy_end` (slower, sec/km) and `hard_end` (faster, sec/km). Paces are computed from the oxygen-cost equations at the percentages above; this crate does not ship Daniels’ copyrighted lookup tables. Published Daniels *Running Formula* charts will differ by a few seconds/km. A 20:00 5K (VDOT ~50) is about E 4:54–5:53/km, T 4:16–4:28/km, I 3:51–4:01/km.

### Age grading

See `examples/age_grade.rs` for a 42-year-old 20:00 5K. `age_equivalent(race, age, gender, target_age)` returns only the equivalent finish time in seconds. `Gender` selects WMA male/female table standards, not a general gender model.

Performance bands: ≥100% world-record level, ≥90% world class, ≥80% national, ≥70% regional, ≥60% local, ≥50% recreational, else developing.

`age_factor` and `open_standard_secs` are public if you need the pieces. 5K–marathon open times are 2025-era World Athletics road world records (men’s marathon 2:00:35), matching USATF MLDR 2025 open standards; they are faster than 2015/2020 championship tables, so age-grade % runs a few points high versus those. 3K uses a rounded track-adjacent stand-in; there is no official road 3K table.

## Notes

- VDOT is *effective* VO2max (economy included), not a lab test.
- Age factors are a WMA-style *approximation*, not official World Masters Athletics or USATF scoring tables. Open 5K–marathon times are 2025-era road world records (USATF MLDR 2025 open standards).
- Predictions assume a flat, all-out effort and similar training specificity.
- No crate dependencies by default; all math is `std`. Optional `serde` feature for `Serialize`/`Deserialize`.
- `Cargo.lock` is committed so clones and CI share a pinned graph. Dependents of the library still ignore it and resolve from `Cargo.toml`.

## Attribution

The running module implements published equations. It is not copied from another crate or from copyrighted pace tables.

- **VDOT / equivalents / training intensities:** Jack Daniels and Jimmy Gilbert, *Oxygen Power* (1979) — oxygen cost of running and sustainable %VO2max versus duration. Training zones here invert those equations at fixed % of VDOT; they are not a transcription of Daniels’ published pace charts. Published *Running Formula* charts will differ by a few seconds/km.
- **Riegel:** Pete Riegel (1977, *Runner’s World*; 1981, *American Scientist*) — `T2 = T1 * (D2/D1)^1.06`.
- **Cameron:** David Cameron’s road-race fit — `T2 = T1 * (D2/D1) * f(D1)/f(D2)`.
- **Age grading:** compact interpolated age factors in the spirit of WMA/USATF road tables. Open 5K–marathon times are 2025-era World Athletics road world records (USATF MLDR 2025 open standards compiled by Alan Jones), not a copy of the official lookup grid. Percentages are estimates, not championship scores.

## Layout for more sports

Add `src/cycling/`, `src/swimming/`, etc., and export them as `sportanalytics::cycling` / `sportanalytics::swimming`. Do not dump new sports onto the crate root — `Distance` will not stay unique.

## License

MIT
