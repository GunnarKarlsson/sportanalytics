# sportanalytics

Crate name: `sportanalytics`. Repository: `sports-analytics`.

Multi-sport analytics crate. First module: **running**.

Std-only helpers for widely used running formulas: Daniels–Gilbert VDOT (effective VO2max), race-time prediction (Daniels, Riegel, Cameron), Daniels training zones, and a compact WMA-style age-grade model.

Add the crate to `Cargo.toml`:

```toml
[dependencies]
sportanalytics = "0.1"
```

Then:

```rust
use sportanalytics::running::*;
```

The same items are also available from `sportanalytics::prelude`. The crate root re-exports only `sportanalytics::Error`.

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

**`Distance`** — `ThreeK`, `FiveK`, `TenK`, `HalfMarathon`, `Marathon`. Metres come from `Distance::meters()` (half marathon is 21,097.5 m; marathon is 42,195 m).

**`RaceTime`** — a distance plus a positive finish time. Construct with `from_hms`, `from_secs`, or `new` (`std::time::Duration`). Invalid times return `Error::NonPositiveTime`.

**`Vdot`** — newtype around a positive finite Daniels VDOT. Inner math is still `f64`; the wrapper is used at API edges (`vdot`, `time_from_vdot`, `Vo2Estimate`, `training_zones_from_vdot`) so a VDOT is not confused with seconds or m/min. Cameron/Riegel times are *not* VDOT values.

**`Error`** — `NonPositiveTime`, `EmptyRaces`, `InvalidVdot`. Implements `std::error::Error`.

### VDOT / VO2max

VDOT from one race, or a mean/best over several:

```rust
use sportanalytics::running::{vo2max_from_races, Distance, RaceTime};

let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
let ten = RaceTime::from_hms(Distance::TenK, 0, 42, 0).unwrap();

let vo2 = vo2max_from_races(&[five, ten]).unwrap();
println!("VDOT mean {:.1}, best {:.1}", vo2.mean.value(), vo2.best.value());
```

A 20:00 5K is about VDOT 50. Use `vo2.best` (or `vo2.mean`) when several results disagree.

Predicted finish time at another distance from a known VDOT:

```rust
use sportanalytics::running::{time_from_vdot, vdot, Distance, RaceTime};

let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
let hm_secs = time_from_vdot(vdot(five), Distance::HalfMarathon);
```

Advanced helpers: `oxygen_cost(v_m_per_min)`, `percent_vo2max(t_min)`, `velocity_from_vo2(vo2)`.

### Race prediction

Pick one model, or ask for Daniels and Cameron together. Riegel remains available on the enum.

```rust
use sportanalytics::running::{
    predict_daniels_and_cameron, predict_times, Distance, PredictionModel, RaceTime,
};

let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();

let pred = predict_times(five, PredictionModel::DanielsVdot);
println!("HM {}  FM {}", pred.formatted(Distance::HalfMarathon), pred.formatted(Distance::Marathon));

let both = predict_daniels_and_cameron(five);
println!(
    "VDOT {:.1}  HM Daniels {}  Cameron {}",
    both.vdot.value(),
    both.daniels.formatted(Distance::HalfMarathon),
    both.cameron.formatted(Distance::HalfMarathon),
);
```

- **DanielsVdot** (recommended): invert Daniels–Gilbert so every distance is an equivalent VDOT.
- **Riegel**: `T2 = T1 * (D2/D1)^1.06` (optional `riegel_with_exponent` if you want another `k`).
- **Cameron**: `T2 = T1 * (D2/D1) * f(D1)/f(D2)` with Cameron’s `f(x)` in metres.

`PredictedTimes::seconds(distance)` returns raw seconds; `formatted` returns `m:ss` or `h:mm:ss`.

### Training zones

Daniels maps VDOT to paces at fixed %VO2max. Cameron/Riegel times do not define zones — use the VDOT from the race (or mean/best of several).

| Zone | % of VDOT | Use |
|---|---|---|
| Easy (E) | 59–74% | easy / long run |
| Marathon (M) | 75–84% | marathon pace |
| Threshold (T) | 83–88% | tempo / cruise intervals |
| Interval (I) | 95–100% | 3–5 min VO2 reps |
| Repetition (R) | ~105–110% | short fast reps |

```rust
use sportanalytics::running::{format_pace, training_zones, training_zones_from_vdot, Distance, RaceTime};

let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
let z = training_zones(five);
println!(
    "E {}–{}  T {}–{}  I {}–{}",
    format_pace(z.easy.easy_end),
    format_pace(z.easy.hard_end),
    format_pace(z.threshold.easy_end),
    format_pace(z.threshold.hard_end),
    format_pace(z.interval.easy_end),
    format_pace(z.interval.hard_end),
);

// Several races: take the best (or mean) VDOT first.
// let z = training_zones_from_vdot(vo2.best);
```

Each `PaceRange` has `easy_end` (slower, sec/km) and `hard_end` (faster, sec/km). Paces are computed from the oxygen-cost equations at the percentages above; this crate does not ship Daniels’ copyrighted lookup tables. A 20:00 5K (VDOT ~50) is about E 4:54–5:53/km, T 4:16–4:28/km, I 3:51–4:01/km.

### Age grading

```rust
use sportanalytics::running::{age_grade, Distance, Gender, RaceTime};

let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
let ag = age_grade(five, 42, Gender::Male, Some(25));
println!(
    "{:.1}% {} | open eq {} | as 25yo {}",
    ag.percent,
    ag.level.label(),
    ag.open_equivalent_hms(),
    ag.equivalent_at_age_hms().unwrap()
);
```

`age_equivalent(race, age, gender, target_age)` returns only the equivalent finish time in seconds.

Performance bands: ≥100% world-record level, ≥90% world class, ≥80% national, ≥70% regional, ≥60% local, ≥50% recreational, else developing.

`age_factor` and `open_standard_secs` are public if you need the pieces. 3K uses a track-adjacent open standard; there is no official road 3K table.

## Notes

- VDOT is *effective* VO2max (economy included), not a lab test.
- Age factors are a WMA-style *approximation*, not official World Masters Athletics or USATF scoring tables.
- Predictions assume a flat, all-out effort and similar training specificity.
- No crate dependencies; all math is `std`.
- `Cargo.lock` is committed so clones and CI share a pinned graph. Dependents of the library still ignore it and resolve from `Cargo.toml`.

## Attribution

The running module implements published equations. It is not copied from another crate or from copyrighted pace tables.

- **VDOT / equivalents / training intensities:** Jack Daniels and Jimmy Gilbert, *Oxygen Power* (1979) — oxygen cost of running and sustainable %VO2max versus duration. Training zones here invert those equations at fixed % of VDOT; they are not a transcription of Daniels’ published pace charts.
- **Riegel:** Pete Riegel (1977, *Runner’s World*; 1981, *American Scientist*) — `T2 = T1 * (D2/D1)^1.06`.
- **Cameron:** David Cameron’s road-race fit — `T2 = T1 * (D2/D1) * f(D1)/f(D2)`.
- **Age grading:** compact interpolated factors in the spirit of WMA/USATF road age grading. Percentages are estimates, not championship scores.

## Layout for more sports

Add `src/cycling/`, `src/swimming/`, etc., and export them as `sportanalytics::cycling` / `sportanalytics::swimming`. Do not dump new sports onto the crate root — `Distance` will not stay unique.

## License

MIT
