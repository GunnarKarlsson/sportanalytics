//! Power, mass, work, and a single maximal or test effort.

use std::fmt;
use std::time::Duration;

use crate::Error;

/// Mean power in watts.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Power(f64);

impl Power {
    /// Construct from a positive finite wattage.
    ///
    /// ```
    /// use sportanalytics::cycling::Power;
    ///
    /// assert_eq!(Power::new(250.0).unwrap().watts(), 250.0);
    /// assert!(Power::new(0.0).is_err());
    /// ```
    pub fn new(watts: f64) -> Result<Self, Error> {
        if watts.is_finite() && watts > 0.0 {
            Ok(Self(watts))
        } else {
            Err(Error::InvalidPower)
        }
    }

    /// Power in watts.
    pub const fn watts(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Power {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.0} W", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Power {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Power {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let watts = <f64 as serde::Deserialize>::deserialize(deserializer)?;
        Power::new(watts).map_err(serde::de::Error::custom)
    }
}

/// Mechanical work in joules.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Work(f64);

impl Work {
    /// Construct from a positive finite energy in joules.
    pub fn new(joules: f64) -> Result<Self, Error> {
        if joules.is_finite() && joules > 0.0 {
            Ok(Self(joules))
        } else {
            Err(Error::InvalidWork)
        }
    }

    /// Work in joules.
    pub const fn joules(self) -> f64 {
        self.0
    }

    /// Work in kilojoules.
    pub fn kj(self) -> f64 {
        self.0 / 1000.0
    }
}

impl fmt::Display for Work {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} kJ", self.kj())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Work {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Work {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let joules = <f64 as serde::Deserialize>::deserialize(deserializer)?;
        Work::new(joules).map_err(serde::de::Error::custom)
    }
}

/// Mass in kilograms (rider, or rider + bike + kit).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass(f64);

impl Mass {
    /// Construct from a positive finite mass in kilograms.
    ///
    /// ```
    /// use sportanalytics::cycling::Mass;
    ///
    /// assert_eq!(Mass::from_kg(75.0).unwrap().kg(), 75.0);
    /// ```
    pub fn from_kg(kg: f64) -> Result<Self, Error> {
        if kg.is_finite() && kg > 0.0 {
            Ok(Self(kg))
        } else {
            Err(Error::InvalidMass)
        }
    }

    /// Mass in kilograms.
    pub const fn kg(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Mass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} kg", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Mass {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Mass {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let kg = <f64 as serde::Deserialize>::deserialize(deserializer)?;
        Mass::from_kg(kg).map_err(serde::de::Error::custom)
    }
}

/// Specific power in watts per kilogram.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WattsPerKg(f64);

impl WattsPerKg {
    /// Construct from a positive finite W/kg value.
    pub fn new(wkg: f64) -> Result<Self, Error> {
        if wkg.is_finite() && wkg > 0.0 {
            Ok(Self(wkg))
        } else {
            Err(Error::InvalidPower)
        }
    }

    /// Value in W/kg.
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for WattsPerKg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} W/kg", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for WattsPerKg {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for WattsPerKg {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wkg = <f64 as serde::Deserialize>::deserialize(deserializer)?;
        WattsPerKg::new(wkg).map_err(serde::de::Error::custom)
    }
}

/// Mean power divided by mass.
///
/// ```
/// use sportanalytics::cycling::{watts_per_kg, Mass, Power};
///
/// let wkg = watts_per_kg(Power::new(300.0).unwrap(), Mass::from_kg(75.0).unwrap());
/// assert!((wkg.value() - 4.0).abs() < 1e-12);
/// ```
pub fn watts_per_kg(power: Power, mass: Mass) -> WattsPerKg {
    WattsPerKg(power.watts() / mass.kg())
}

/// A single timed effort: duration and mean power.
///
/// Use mean maximal power (MMP) or a protocol test mean. Internal units are
/// watts and seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effort {
    duration: Duration,
    power: Power,
}

impl Effort {
    /// Build from power and a positive [`Duration`].
    ///
    /// Returns [`Error::NonPositiveTime`] when `duration` is zero.
    pub fn new(power: Power, duration: Duration) -> Result<Self, Error> {
        if duration.as_secs_f64() <= 0.0 {
            return Err(Error::NonPositiveTime);
        }
        Ok(Self { duration, power })
    }

    /// Build from watts and duration in seconds.
    ///
    /// ```
    /// use sportanalytics::cycling::Effort;
    ///
    /// let twenty = Effort::from_watts_secs(280.0, 1200.0).unwrap();
    /// assert_eq!(twenty.power().watts(), 280.0);
    /// assert_eq!(twenty.seconds(), 1200.0);
    /// ```
    pub fn from_watts_secs(watts: f64, seconds: f64) -> Result<Self, Error> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err(Error::NonPositiveTime);
        }
        Self::new(Power::new(watts)?, Duration::from_secs_f64(seconds))
    }

    /// Build from watts and hours / minutes / seconds.
    ///
    /// Minutes and seconds must be `< 60` ([`Error::InvalidHms`]).
    ///
    /// ```
    /// use sportanalytics::cycling::Effort;
    ///
    /// let twenty = Effort::from_hms(280.0, 0, 20, 0).unwrap();
    /// assert_eq!(twenty.seconds(), 1200.0);
    /// ```
    pub fn from_hms(watts: f64, hours: u64, minutes: u64, seconds: u64) -> Result<Self, Error> {
        if minutes >= 60 || seconds >= 60 {
            return Err(Error::InvalidHms);
        }
        let total = hours * 3600 + minutes * 60 + seconds;
        Self::new(Power::new(watts)?, Duration::from_secs(total))
    }

    /// Mean power.
    pub const fn power(self) -> Power {
        self.power
    }

    /// Effort duration.
    pub const fn duration(self) -> Duration {
        self.duration
    }

    /// Duration in seconds.
    pub fn seconds(self) -> f64 {
        self.duration.as_secs_f64()
    }

    /// Mechanical work `P × t` in joules.
    pub fn work(self) -> Work {
        Work(self.power.watts() * self.seconds())
    }

    /// Specific power for a given mass.
    pub fn watts_per_kg(self, mass: Mass) -> WattsPerKg {
        watts_per_kg(self.power, mass)
    }
}

impl fmt::Display for Effort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} for {:.0}s", self.power, self.seconds())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Effort {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Effort", 2)?;
        state.serialize_field("watts", &self.power.watts())?;
        state.serialize_field("seconds", &self.seconds())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Effort {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Helper {
            watts: f64,
            seconds: f64,
        }
        let helper = Helper::deserialize(deserializer)?;
        Effort::from_watts_secs(helper.watts, helper.seconds).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_mass_work_constructors() {
        assert_eq!(Power::new(250.0).unwrap().watts(), 250.0);
        assert_eq!(Power::new(0.0), Err(Error::InvalidPower));
        assert_eq!(Power::new(-1.0), Err(Error::InvalidPower));
        assert_eq!(Power::new(f64::NAN), Err(Error::InvalidPower));
        assert_eq!(Power::new(250.0).unwrap().to_string(), "250 W");

        assert_eq!(Mass::from_kg(0.0), Err(Error::InvalidMass));
        assert_eq!(Work::new(0.0), Err(Error::InvalidWork));
        assert!((Work::new(18_000.0).unwrap().kj() - 18.0).abs() < 1e-12);
    }

    #[test]
    fn effort_from_watts_secs_and_hms() {
        let a = Effort::from_watts_secs(280.0, 1200.0).unwrap();
        let b = Effort::from_hms(280.0, 0, 20, 0).unwrap();
        assert_eq!(a.seconds(), b.seconds());
        assert_eq!(a.power().watts(), 280.0);
        assert!((a.work().joules() - 280.0 * 1200.0).abs() < 1e-9);
        let mass = Mass::from_kg(70.0).unwrap();
        assert!((a.watts_per_kg(mass).value() - 4.0).abs() < 1e-12);
        assert_eq!(a.watts_per_kg(mass).to_string(), "4.00 W/kg");
    }

    #[test]
    fn effort_rejects_bad_inputs() {
        assert_eq!(
            Effort::from_watts_secs(280.0, 0.0),
            Err(Error::NonPositiveTime)
        );
        assert_eq!(
            Effort::from_watts_secs(0.0, 1200.0),
            Err(Error::InvalidPower)
        );
        assert_eq!(Effort::from_hms(280.0, 0, 90, 0), Err(Error::InvalidHms));
        assert_eq!(Effort::from_hms(280.0, 0, 0, 60), Err(Error::InvalidHms));
        assert_eq!(
            Effort::new(Power::new(100.0).unwrap(), Duration::ZERO),
            Err(Error::NonPositiveTime)
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_effort_roundtrip() {
        let effort = Effort::from_watts_secs(280.0, 1200.0).unwrap();
        let back: Effort = serde_json::from_str(&serde_json::to_string(&effort).unwrap()).unwrap();
        assert_eq!(back, effort);
        assert!(serde_json::from_str::<Effort>(r#"{"watts":0,"seconds":1200}"#).is_err());
    }
}
