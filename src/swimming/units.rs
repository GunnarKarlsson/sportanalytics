//! Length display units and a typed swim pace (seconds per metre).

use std::fmt;

use crate::Error;

/// International yard in metres (exactly 0.9144).
pub const METERS_PER_YARD: f64 = 0.9144;

/// One hundred international yards in metres.
pub const METERS_PER_100Y: f64 = 100.0 * METERS_PER_YARD;

/// Display / constructor unit for pace. Internal storage is seconds per metre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LengthUnit {
    /// Per 100 metres (default Display).
    #[default]
    Per100m,
    /// Per 100 international yards.
    Per100y,
}

/// Swim pace. Stored as seconds per metre.
///
/// Default [`std::fmt::Display`] is `m:ss.s /100m`. Use [`Self::display`] for
/// `/100y`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pace {
    sec_per_meter: f64,
}

impl Pace {
    /// Pace from seconds per metre.
    ///
    /// Returns [`Error::InvalidPace`] when `sec_per_meter` is non-finite or not
    /// strictly positive.
    pub fn from_sec_per_meter(sec_per_meter: f64) -> Result<Self, Error> {
        if sec_per_meter.is_finite() && sec_per_meter > 0.0 {
            Ok(Self { sec_per_meter })
        } else {
            Err(Error::InvalidPace)
        }
    }

    /// Pace from seconds per 100 metres.
    ///
    /// ```
    /// use sportanalytics::swimming::Pace;
    ///
    /// assert_eq!(Pace::per_100m(105.0).unwrap().to_string(), "1:45.0 /100m");
    /// ```
    pub fn per_100m(sec_per_100m: f64) -> Result<Self, Error> {
        Self::from_sec_per_meter(sec_per_100m / 100.0)
    }

    /// Pace from seconds per 100 international yards.
    pub fn per_100y(sec_per_100y: f64) -> Result<Self, Error> {
        Self::from_sec_per_meter(sec_per_100y / METERS_PER_100Y)
    }

    /// Seconds per metre.
    pub const fn sec_per_meter(self) -> f64 {
        self.sec_per_meter
    }

    /// Seconds per 100 metres.
    pub fn sec_per_100m(self) -> f64 {
        self.sec_per_meter * 100.0
    }

    /// Seconds per 100 international yards.
    pub fn sec_per_100y(self) -> f64 {
        self.sec_per_meter * METERS_PER_100Y
    }

    /// Velocity in metres per second.
    pub fn velocity_m_per_s(self) -> f64 {
        1.0 / self.sec_per_meter
    }

    /// Seconds for one unit of `unit`.
    pub fn seconds(self, unit: LengthUnit) -> f64 {
        match unit {
            LengthUnit::Per100m => self.sec_per_100m(),
            LengthUnit::Per100y => self.sec_per_100y(),
        }
    }

    /// Display this pace in `unit` (`/100m` or `/100y`).
    pub const fn display(self, unit: LengthUnit) -> PaceDisplay {
        PaceDisplay { pace: self, unit }
    }
}

impl fmt::Display for Pace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.display(LengthUnit::Per100m), f)
    }
}

/// A [`Pace`] paired with a [`LengthUnit`] for formatting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaceDisplay {
    pace: Pace,
    unit: LengthUnit,
}

impl fmt::Display for PaceDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.pace.seconds(self.unit);
        let suffix = match self.unit {
            LengthUnit::Per100m => "/100m",
            LengthUnit::Per100y => "/100y",
        };
        // One-tenth second is enough for zone edges; round to nearest tenth.
        let tenths = (total * 10.0).round() as i64;
        let whole = tenths / 10;
        let tenth = (tenths % 10).unsigned_abs();
        let mins = whole / 60;
        let secs = whole % 60;
        write!(f, "{mins}:{secs:02}.{tenth} {suffix}")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Pace {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Pace", 1)?;
        state.serialize_field("sec_per_meter", &self.sec_per_meter)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Pace {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Helper {
            sec_per_meter: f64,
        }
        let helper = Helper::deserialize(deserializer)?;
        Pace::from_sec_per_meter(helper.sec_per_meter).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_100m_display() {
        assert_eq!(Pace::per_100m(105.0).unwrap().to_string(), "1:45.0 /100m");
        assert_eq!(Pace::per_100m(100.0).unwrap().to_string(), "1:40.0 /100m");
    }

    #[test]
    fn hundred_m_to_velocity_roundtrip() {
        let pace = Pace::per_100m(100.0).unwrap();
        assert!((pace.velocity_m_per_s() - 1.0).abs() < 1e-12);
        assert!((pace.sec_per_100m() - 100.0).abs() < 1e-12);
        let back = Pace::from_sec_per_meter(1.0 / pace.velocity_m_per_s()).unwrap();
        assert!((back.sec_per_100m() - 100.0).abs() < 1e-12);
    }

    #[test]
    fn per_100y_converts() {
        let pace = Pace::per_100y(90.0).unwrap();
        assert!((pace.sec_per_100y() - 90.0).abs() < 1e-12);
        let s = pace.display(LengthUnit::Per100y).to_string();
        assert!(s.contains("/100y"), "got {s}");
    }

    #[test]
    fn constructors_reject_non_positive() {
        assert_eq!(Pace::per_100m(0.0), Err(Error::InvalidPace));
        assert_eq!(Pace::per_100m(f64::NAN), Err(Error::InvalidPace));
        assert_eq!(Pace::per_100y(-1.0), Err(Error::InvalidPace));
        assert_eq!(Pace::from_sec_per_meter(0.0), Err(Error::InvalidPace));
    }

    #[test]
    fn length_unit_default_is_per_100m() {
        assert_eq!(LengthUnit::default(), LengthUnit::Per100m);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_pace_roundtrip() {
        let pace = Pace::per_100m(105.0).unwrap();
        let json = serde_json::to_string(&pace).unwrap();
        let back: Pace = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pace);
    }
}
