//! Coggan-style power zones from FTP.
//!
//! Percentages are public coaching convention (same role as Daniels %VDOT for
//! running). This crate does **not** ship copyrighted Coggan power-profile
//! charts.

use std::fmt;

use super::effort::{Mass, Power, WattsPerKg};
use super::ftp::{ftp_from_protocol, Ftp, FtpProtocol};
use super::Effort;
use super::Error;

/// A power band with a softer and a harder edge (watts).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerRange {
    /// Softer (lower) edge.
    pub easy_end: Power,
    /// Harder (upper) edge.
    pub hard_end: Power,
}

impl PowerRange {
    /// Both edges as W/kg for `mass`.
    ///
    /// ```
    /// use sportanalytics::cycling::{training_zones, Ftp, Mass};
    ///
    /// let z = training_zones(Ftp::new(250.0).unwrap());
    /// let (lo, hi) = z.endurance.watts_per_kg(Mass::from_kg(70.0).unwrap());
    /// assert!((lo.value() - 2.0).abs() < 0.01);
    /// assert!((hi.value() - 187.5 / 70.0).abs() < 1e-12);
    /// ```
    pub fn watts_per_kg(self, mass: Mass) -> (WattsPerKg, WattsPerKg) {
        (
            WattsPerKg::new(self.easy_end.watts() / mass.kg()).expect("positive power and mass"),
            WattsPerKg::new(self.hard_end.watts() / mass.kg()).expect("positive power and mass"),
        )
    }
}

impl fmt::Display for PowerRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `f64::round` is half-away-from-zero (137.5 → 138); `{:.0}` alone is
        // ties-to-even and would print 262.5 as 262.
        write!(
            f,
            "{:.0}–{:.0} W",
            self.easy_end.watts().round(),
            self.hard_end.watts().round()
        )
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for PowerRange {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PowerRange", 2)?;
        state.serialize_field("easy_end", &self.easy_end.watts())?;
        state.serialize_field("hard_end", &self.hard_end.watts())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for PowerRange {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Helper {
            easy_end: f64,
            hard_end: f64,
        }
        let helper = Helper::deserialize(deserializer)?;
        Ok(PowerRange {
            easy_end: Power::new(helper.easy_end).map_err(serde::de::Error::custom)?,
            hard_end: Power::new(helper.hard_end).map_err(serde::de::Error::custom)?,
        })
    }
}

/// Coggan seven zones plus sweet spot, derived from FTP.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerZones {
    /// FTP used to compute the zones.
    pub ftp: Ftp,
    /// Z1 active recovery, ≤ 55% FTP.
    pub recovery: PowerRange,
    /// Z2 endurance, 56–75% FTP.
    pub endurance: PowerRange,
    /// Z3 tempo, 76–90% FTP.
    pub tempo: PowerRange,
    /// Sweet spot, 88–94% FTP.
    pub sweet_spot: PowerRange,
    /// Z4 threshold, 91–105% FTP.
    pub threshold: PowerRange,
    /// Z5 VO2max, 106–120% FTP.
    pub vo2max: PowerRange,
    /// Z6 anaerobic capacity, 121–150% FTP.
    pub anaerobic: PowerRange,
    /// Z7 neuromuscular floor, ≥ 151% FTP.
    pub neuromuscular_floor: Power,
}

impl fmt::Display for PowerZones {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FTP {} | Z1 {} | Z2 {} | Z3 {} | SS {} | Z4 {} | Z5 {} | Z6 {} | Z7 ≥ {:.0} W",
            self.ftp.watts().round(),
            self.recovery,
            self.endurance,
            self.tempo,
            self.sweet_spot,
            self.threshold,
            self.vo2max,
            self.anaerobic,
            self.neuromuscular_floor.watts()
        )
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for PowerZones {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PowerZones", 9)?;
        state.serialize_field("ftp", &self.ftp)?;
        state.serialize_field("recovery", &self.recovery)?;
        state.serialize_field("endurance", &self.endurance)?;
        state.serialize_field("tempo", &self.tempo)?;
        state.serialize_field("sweet_spot", &self.sweet_spot)?;
        state.serialize_field("threshold", &self.threshold)?;
        state.serialize_field("vo2max", &self.vo2max)?;
        state.serialize_field("anaerobic", &self.anaerobic)?;
        state.serialize_field("neuromuscular_floor", &self.neuromuscular_floor)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for PowerZones {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Helper {
            ftp: Ftp,
            recovery: PowerRange,
            endurance: PowerRange,
            tempo: PowerRange,
            sweet_spot: PowerRange,
            threshold: PowerRange,
            vo2max: PowerRange,
            anaerobic: PowerRange,
            neuromuscular_floor: Power,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(PowerZones {
            ftp: h.ftp,
            recovery: h.recovery,
            endurance: h.endurance,
            tempo: h.tempo,
            sweet_spot: h.sweet_spot,
            threshold: h.threshold,
            vo2max: h.vo2max,
            anaerobic: h.anaerobic,
            neuromuscular_floor: h.neuromuscular_floor,
        })
    }
}

fn range_from_pct(ftp_w: f64, lo: f64, hi: f64) -> PowerRange {
    // Recovery / Z1 starts at a tiny positive floor so Power::new stays valid.
    let easy = if lo <= 0.0 {
        f64::MIN_POSITIVE.max(ftp_w * 1e-6)
    } else {
        ftp_w * lo
    };
    PowerRange {
        easy_end: Power::new(easy).expect("FTP-derived power"),
        hard_end: Power::new(ftp_w * hi).expect("FTP-derived power"),
    }
}

/// Coggan zones from an already-known FTP.
///
/// | Zone | % FTP |
/// |---|---|
/// | Z1 recovery | 0–55 |
/// | Z2 endurance | 56–75 |
/// | Z3 tempo | 76–90 |
/// | Sweet spot | 88–94 |
/// | Z4 threshold | 91–105 |
/// | Z5 VO2max | 106–120 |
/// | Z6 anaerobic | 121–150 |
/// | Z7 neuromuscular | ≥ 151 |
///
/// One-watt gaps such as 55% → 56% are the printed Coggan convention, not
/// missing watts.
///
/// ```
/// use sportanalytics::cycling::{training_zones, Ftp};
///
/// let z = training_zones(Ftp::new(250.0).unwrap());
/// assert_eq!(z.endurance.to_string(), "140–188 W");
/// assert_eq!(z.threshold.to_string(), "228–263 W");
/// ```
pub fn training_zones(ftp: Ftp) -> PowerZones {
    let w = ftp.watts();
    PowerZones {
        ftp,
        recovery: range_from_pct(w, 0.0, 0.55),
        endurance: range_from_pct(w, 0.56, 0.75),
        tempo: range_from_pct(w, 0.76, 0.90),
        sweet_spot: range_from_pct(w, 0.88, 0.94),
        threshold: range_from_pct(w, 0.91, 1.05),
        vo2max: range_from_pct(w, 1.06, 1.20),
        anaerobic: range_from_pct(w, 1.21, 1.50),
        neuromuscular_floor: Power::new(w * 1.51).expect("FTP-derived power"),
    }
}

/// Zones from a single effort via [`ftp_from_protocol`].
pub fn training_zones_from_effort(
    effort: Effort,
    protocol: FtpProtocol,
) -> Result<PowerZones, Error> {
    Ok(training_zones(ftp_from_protocol(effort, protocol)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cycling")
    }

    fn rounded_edge(p: Power) -> i64 {
        // Half-away-from-zero, matching PowerRange Display.
        p.watts().round() as i64
    }

    #[test]
    fn fixture_coggan_zones() {
        let text = fs::read_to_string(fixtures_dir().join("zones.csv")).unwrap();
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }
            let c: Vec<_> = line.split(',').collect();
            let ftp_w: f64 = c[0].parse().unwrap();
            let z = training_zones(Ftp::new(ftp_w).unwrap());
            let expect = |idx: usize| -> i64 { c[idx].parse().unwrap() };
            assert_eq!(
                rounded_edge(z.recovery.hard_end),
                expect(1),
                "Z1 hi row {i}"
            );
            assert_eq!(
                rounded_edge(z.endurance.easy_end),
                expect(2),
                "Z2 lo row {i}"
            );
            assert_eq!(
                rounded_edge(z.endurance.hard_end),
                expect(3),
                "Z2 hi row {i}"
            );
            assert_eq!(rounded_edge(z.tempo.easy_end), expect(4));
            assert_eq!(rounded_edge(z.tempo.hard_end), expect(5));
            assert_eq!(rounded_edge(z.sweet_spot.easy_end), expect(6));
            assert_eq!(rounded_edge(z.sweet_spot.hard_end), expect(7));
            assert_eq!(rounded_edge(z.threshold.easy_end), expect(8));
            assert_eq!(rounded_edge(z.threshold.hard_end), expect(9));
            assert_eq!(rounded_edge(z.vo2max.easy_end), expect(10));
            assert_eq!(rounded_edge(z.vo2max.hard_end), expect(11));
            assert_eq!(rounded_edge(z.anaerobic.easy_end), expect(12));
            assert_eq!(rounded_edge(z.anaerobic.hard_end), expect(13));
            assert_eq!(
                rounded_edge(z.neuromuscular_floor),
                expect(14),
                "Z7 row {i}"
            );
        }
    }

    #[test]
    fn ftp_250_display_matches_table() {
        let z = training_zones(Ftp::new(250.0).unwrap());
        assert_eq!(z.recovery.to_string(), "0–138 W");
        assert_eq!(z.endurance.to_string(), "140–188 W");
        assert_eq!(z.tempo.to_string(), "190–225 W");
        assert_eq!(z.sweet_spot.to_string(), "220–235 W");
        assert_eq!(z.threshold.to_string(), "228–263 W");
        assert_eq!(z.vo2max.to_string(), "265–300 W");
        assert_eq!(z.anaerobic.to_string(), "303–375 W");
        assert_eq!(
            format!("{:.0}", z.neuromuscular_floor.watts().round()),
            "378"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_zones_roundtrip() {
        let z = training_zones(Ftp::new(250.0).unwrap());
        let back: PowerZones = serde_json::from_str(&serde_json::to_string(&z).unwrap()).unwrap();
        assert_eq!(back.ftp, z.ftp);
        assert_eq!(
            back.endurance.easy_end.watts(),
            z.endurance.easy_end.watts()
        );
    }
}
