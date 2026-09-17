//! Functional threshold power (FTP) from common field protocols.
//!
//! FTP here is an **operational** training anchor (zones, IF-style load), not a
//! laboratory lactate threshold. Protocols and factors are published coaching
//! conventions.

use std::fmt;

use super::critical_power::CriticalPower;
use super::effort::{Effort, Power};
use crate::Error;

/// Functional threshold power in watts.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ftp(f64);

impl Ftp {
    /// Construct from a positive finite wattage.
    pub fn new(watts: f64) -> Result<Self, Error> {
        if watts.is_finite() && watts > 0.0 {
            Ok(Self(watts))
        } else {
            Err(Error::InvalidPower)
        }
    }

    pub(crate) fn from_raw(watts: f64) -> Self {
        Self(watts)
    }

    /// FTP in watts.
    pub const fn watts(self) -> f64 {
        self.0
    }

    /// As a [`Power`].
    pub fn power(self) -> Power {
        Power::new(self.0).expect("Ftp invariants imply positive finite watts")
    }
}

impl fmt::Display for Ftp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.0} W FTP", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Ftp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Ftp {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let watts = <f64 as serde::Deserialize>::deserialize(deserializer)?;
        Ftp::new(watts).map_err(serde::de::Error::custom)
    }
}

/// Field-test protocol used to estimate FTP from a single effort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FtpProtocol {
    /// 60-minute maximal mean: `FTP = P`.
    SixtyMin,
    /// ~20-minute test: `FTP = 0.95 × P`.
    TwentyMin,
    /// ~8-minute test: `FTP = 0.90 × P`.
    EightMin,
    /// Mean power treated as MAP / Wpeak (`FTP = MAP / 1.20`).
    ///
    /// Duration window is **3–8 min**. Input is the mean power of that MAP /
    /// Wpeak effort — **not** the TrainingPeaks / Wahoo “75% of 1-min ramp peak”
    /// ramp-test convention. Do not silently treat an arbitrary 3–8 min effort
    /// as MAP unless that is what you measured.
    RampMap,
    /// Treat effort power as CP: `FTP = 0.96 × P`.
    CriticalPower,
}

/// Factor applied to a 20-minute maximal mean (`0.95`).
pub const TWENTY_MIN_FACTOR: f64 = 0.95;
/// Factor applied to an 8-minute maximal mean (`0.90`).
pub const EIGHT_MIN_FACTOR: f64 = 0.90;
/// MAP → FTP: `FTP = MAP × MAP_TO_FTP` with `MAP_TO_FTP = 1/1.20`.
pub const MAP_TO_FTP: f64 = 1.0 / 1.20;
/// CP → FTP factor (`0.96`).
pub const CP_TO_FTP: f64 = 0.96;

fn duration_in_range(seconds: f64, lo: f64, hi: f64) -> Result<(), Error> {
    if seconds >= lo && seconds <= hi {
        Ok(())
    } else {
        Err(Error::DurationOutOfModelRange)
    }
}

/// FTP from a ~20-minute maximal effort (`0.95 × P`).
///
/// Duration must be 15–25 minutes ([`Error::DurationOutOfModelRange`] otherwise).
///
/// ```
/// use sportanalytics::cycling::{ftp_from_20min, Effort};
///
/// let twenty = Effort::from_watts_secs(280.0, 1200.0).unwrap();
/// assert!((ftp_from_20min(twenty).unwrap().watts() - 266.0).abs() < 1e-9);
/// ```
pub fn ftp_from_20min(effort: Effort) -> Result<Ftp, Error> {
    ftp_from_protocol(effort, FtpProtocol::TwentyMin)
}

/// FTP from a ~60-minute maximal effort (`FTP = P`).
///
/// Duration must be 45–75 minutes.
pub fn ftp_from_60min(effort: Effort) -> Result<Ftp, Error> {
    ftp_from_protocol(effort, FtpProtocol::SixtyMin)
}

/// FTP from a maximal effort using `protocol`.
///
/// Duration windows: TwentyMin 15–25 min, SixtyMin 45–75 min, EightMin 6–10 min,
/// RampMap 3–8 min (MAP / Wpeak mean — see [`FtpProtocol::RampMap`]).
/// [`FtpProtocol::CriticalPower`] has no duration window (effort power is
/// treated as CP).
pub fn ftp_from_protocol(effort: Effort, protocol: FtpProtocol) -> Result<Ftp, Error> {
    let t = effort.seconds();
    let p = effort.power().watts();
    let watts = match protocol {
        FtpProtocol::SixtyMin => {
            duration_in_range(t, 45.0 * 60.0, 75.0 * 60.0)?;
            p
        }
        FtpProtocol::TwentyMin => {
            duration_in_range(t, 15.0 * 60.0, 25.0 * 60.0)?;
            TWENTY_MIN_FACTOR * p
        }
        FtpProtocol::EightMin => {
            duration_in_range(t, 6.0 * 60.0, 10.0 * 60.0)?;
            EIGHT_MIN_FACTOR * p
        }
        FtpProtocol::RampMap => {
            duration_in_range(t, 3.0 * 60.0, 8.0 * 60.0)?;
            MAP_TO_FTP * p
        }
        FtpProtocol::CriticalPower => CP_TO_FTP * p,
    };
    Ftp::new(watts)
}

/// FTP ≈ `0.96 × CP`.
pub fn ftp_from_cp(cp: CriticalPower) -> Ftp {
    Ftp::from_raw(CP_TO_FTP * cp.cp.watts())
}

/// FTP from maximal aerobic power: `FTP = MAP / 1.20`.
pub fn ftp_from_map(map: Power) -> Result<Ftp, Error> {
    Ftp::new(MAP_TO_FTP * map.watts())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cycling")
    }

    #[test]
    fn fixture_ftp_protocols() {
        let text = fs::read_to_string(fixtures_dir().join("ftp.csv")).unwrap();
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }
            let cols: Vec<_> = line.split(',').collect();
            let protocol = match cols[0] {
                "twenty" => FtpProtocol::TwentyMin,
                "sixty" => FtpProtocol::SixtyMin,
                "eight" => FtpProtocol::EightMin,
                "ramp" => FtpProtocol::RampMap,
                other => panic!("unknown protocol {other}"),
            };
            let watts: f64 = cols[1].parse().unwrap();
            let seconds: f64 = cols[2].parse().unwrap();
            let expected: f64 = cols[3].parse().unwrap();
            let effort = Effort::from_watts_secs(watts, seconds).unwrap();
            let ftp = ftp_from_protocol(effort, protocol).unwrap();
            assert!(
                (ftp.watts() - expected).abs() < 1e-9,
                "row {i}: got {} want {expected}",
                ftp.watts()
            );
        }
    }

    #[test]
    fn duration_guards() {
        let short = Effort::from_watts_secs(280.0, 600.0).unwrap();
        assert_eq!(ftp_from_20min(short), Err(Error::DurationOutOfModelRange));
        let ok = Effort::from_watts_secs(280.0, 1200.0).unwrap();
        assert!(ftp_from_20min(ok).is_ok());
    }

    #[test]
    fn map_and_cp_helpers() {
        let map = Power::new(300.0).unwrap();
        assert!((ftp_from_map(map).unwrap().watts() - 250.0).abs() < 1e-9);
        let cp = CriticalPower {
            cp: Power::new(250.0).unwrap(),
            w_prime: super::super::effort::Work::new(18_000.0).unwrap(),
        };
        assert!((ftp_from_cp(cp).watts() - 240.0).abs() < 1e-9);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_ftp_roundtrip() {
        let ftp = Ftp::new(266.0).unwrap();
        let back: Ftp = serde_json::from_str(&serde_json::to_string(&ftp).unwrap()).unwrap();
        assert_eq!(back, ftp);
    }
}
