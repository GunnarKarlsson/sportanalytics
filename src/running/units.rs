//! Length display units and a typed pace newtype (seconds per metre).

use std::fmt;

use crate::Error;

/// International mile in metres.
pub const METERS_PER_MILE: f64 = 1609.344;

/// Unit used when reading or displaying a pace at the I/O edge.
///
/// Internal math stays in metres and seconds. The default is kilometre so
/// existing `/km` strings stay valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LengthUnit {
    /// Pace relative to one kilometre (1000 m).
    #[default]
    Kilometer,
    /// Pace relative to one international mile ([`METERS_PER_MILE`] m).
    Mile,
}

/// Running pace stored as seconds per metre.
///
/// Construct from km or mile inputs; convert with [`Self::sec_per_km`],
/// [`Self::sec_per_mile`], or [`Self::seconds`]. Default [`Display`] is
/// `m:ss /km`; use [`Self::display`] for `/mi`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pace {
    sec_per_meter: f64,
}

impl Pace {
    /// Pace from seconds per kilometre.
    ///
    /// Returns [`Error::InvalidPace`] when `sec_per_km` is non-finite or not
    /// strictly positive.
    ///
    /// ```
    /// use sportanalytics::running::Pace;
    ///
    /// assert_eq!(Pace::per_km(294.0).unwrap().to_string(), "4:54 /km");
    /// ```
    pub fn per_km(sec_per_km: f64) -> Result<Self, Error> {
        Self::from_sec_per_meter(sec_per_km / 1000.0)
    }

    /// Pace from seconds per international mile.
    ///
    /// Returns [`Error::InvalidPace`] when `sec_per_mile` is non-finite or not
    /// strictly positive.
    pub fn per_mile(sec_per_mile: f64) -> Result<Self, Error> {
        Self::from_sec_per_meter(sec_per_mile / METERS_PER_MILE)
    }

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

    /// Pace from hours, minutes, and seconds per kilometre.
    ///
    /// Minutes and seconds must be `< 60` ([`Error::InvalidHms`]). The total
    /// must be strictly positive ([`Error::InvalidPace`]).
    ///
    /// ```
    /// use sportanalytics::running::Pace;
    ///
    /// let pace = Pace::from_hms_per_km(0, 5, 0).unwrap();
    /// assert_eq!(pace.sec_per_km(), 300.0);
    /// ```
    pub fn from_hms_per_km(hours: u64, minutes: u64, seconds: u64) -> Result<Self, Error> {
        Self::per_km(hms_to_secs(hours, minutes, seconds)?)
    }

    /// Pace from hours, minutes, and seconds per international mile.
    ///
    /// Minutes and seconds must be `< 60` ([`Error::InvalidHms`]). The total
    /// must be strictly positive ([`Error::InvalidPace`]).
    pub fn from_hms_per_mile(hours: u64, minutes: u64, seconds: u64) -> Result<Self, Error> {
        Self::per_mile(hms_to_secs(hours, minutes, seconds)?)
    }

    /// Seconds per metre.
    pub const fn sec_per_meter(self) -> f64 {
        self.sec_per_meter
    }

    /// Seconds per kilometre.
    pub fn sec_per_km(self) -> f64 {
        self.sec_per_meter * 1000.0
    }

    /// Seconds per international mile.
    pub fn sec_per_mile(self) -> f64 {
        self.sec_per_meter * METERS_PER_MILE
    }

    /// Seconds for one unit of `unit`.
    pub fn seconds(self, unit: LengthUnit) -> f64 {
        match unit {
            LengthUnit::Kilometer => self.sec_per_km(),
            LengthUnit::Mile => self.sec_per_mile(),
        }
    }

    /// Display this pace in `unit` (`/km` or `/mi`).
    pub const fn display(self, unit: LengthUnit) -> PaceDisplay {
        PaceDisplay { pace: self, unit }
    }
}

impl fmt::Display for Pace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.display(LengthUnit::Kilometer), f)
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
        let total = self.pace.seconds(self.unit).round() as u64;
        let suffix = match self.unit {
            LengthUnit::Kilometer => "/km",
            LengthUnit::Mile => "/mi",
        };
        write!(f, "{}:{:02} {suffix}", total / 60, total % 60)
    }
}

fn hms_to_secs(hours: u64, minutes: u64, seconds: u64) -> Result<f64, Error> {
    if minutes >= 60 || seconds >= 60 {
        return Err(Error::InvalidHms);
    }
    Ok((hours * 3600 + minutes * 60 + seconds) as f64)
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
    fn five_min_km_converts_to_about_eight_oh_two_mile() {
        let pace = Pace::from_hms_per_km(0, 5, 0).unwrap();
        let sec_mi = pace.sec_per_mile();
        assert!((sec_mi - 482.0).abs() < 1.0, "got {sec_mi}");
        let back = Pace::per_mile(sec_mi).unwrap();
        assert!((back.sec_per_km() - 300.0).abs() < 1e-9);
    }

    #[test]
    fn per_km_display_matches_format_pace() {
        assert_eq!(Pace::per_km(294.0).unwrap().to_string(), "4:54 /km");
    }

    #[test]
    fn display_mile_uses_mi_suffix() {
        let pace = Pace::per_km(294.0).unwrap();
        let s = pace.display(LengthUnit::Mile).to_string();
        assert!(s.contains("/mi"), "got {s}");
    }

    #[test]
    fn constructors_reject_zero_nan_and_overflow_hms() {
        assert_eq!(Pace::per_km(0.0), Err(Error::InvalidPace));
        assert_eq!(Pace::per_km(f64::NAN), Err(Error::InvalidPace));
        assert_eq!(Pace::per_mile(0.0), Err(Error::InvalidPace));
        assert_eq!(Pace::from_sec_per_meter(-1.0), Err(Error::InvalidPace));
        assert_eq!(Pace::from_hms_per_km(0, 90, 0), Err(Error::InvalidHms));
        assert_eq!(Pace::from_hms_per_mile(0, 0, 60), Err(Error::InvalidHms));
        assert_eq!(Pace::from_hms_per_km(0, 0, 0), Err(Error::InvalidPace));
    }

    #[test]
    fn seconds_accessor_matches_unit() {
        let pace = Pace::per_km(300.0).unwrap();
        assert_eq!(pace.seconds(LengthUnit::Kilometer), 300.0);
        assert!((pace.seconds(LengthUnit::Mile) - 300.0 * 1.609344).abs() < 1e-9);
    }

    #[test]
    fn length_unit_default_is_kilometer() {
        assert_eq!(LengthUnit::default(), LengthUnit::Kilometer);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_pace_roundtrip_as_sec_per_meter() {
        let pace = Pace::per_km(294.0).unwrap();
        let json = serde_json::to_string(&pace).unwrap();
        assert!(json.contains("sec_per_meter"));
        let back: Pace = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pace);
        assert!(serde_json::from_str::<Pace>(r#"{"sec_per_meter":0}"#).is_err());
    }
}
