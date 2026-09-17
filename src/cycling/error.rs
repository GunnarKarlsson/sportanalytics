use std::error::Error as StdError;
use std::fmt;

/// Failures from cycling constructors and analytics helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    /// Shared input failure (time / HMS / age range).
    Shared(crate::Error),
    /// Power was zero, negative, or non-finite.
    InvalidPower,
    /// Mass was zero, negative, or non-finite.
    InvalidMass,
    /// Work (joules) was zero, negative, or non-finite.
    InvalidWork,
    /// Not enough maximal efforts for the requested model (e.g. CP needs ≥ 2).
    InsufficientEfforts,
    /// Effort or target duration is outside the model's valid window.
    DurationOutOfModelRange,
    /// Power–duration model could not be solved (non-positive CP/W′, or P ≤ CP).
    UnsolvablePowerDuration,
    /// A physics parameter (CdA, Crr, η, air density, speed) is out of range.
    InvalidPhysicsParam,
    /// Power↔speed bisection did not bracket a root.
    UnsolvableSpeed,
}

impl From<crate::Error> for Error {
    fn from(e: crate::Error) -> Self {
        Self::Shared(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shared(e) => fmt::Display::fmt(e, f),
            Self::InvalidPower => f.write_str("power must be a positive finite number of watts"),
            Self::InvalidMass => f.write_str("mass must be a positive finite number of kilograms"),
            Self::InvalidWork => f.write_str("work must be a positive finite number of joules"),
            Self::InsufficientEfforts => f.write_str("not enough maximal efforts for this model"),
            Self::DurationOutOfModelRange => {
                f.write_str("duration is outside the model's valid window")
            }
            Self::UnsolvablePowerDuration => {
                f.write_str("power–duration model could not be solved")
            }
            Self::InvalidPhysicsParam => {
                f.write_str("physics parameter is zero, negative, non-finite, or out of range")
            }
            Self::UnsolvableSpeed => {
                f.write_str("no speed in the solver bracket matches the given power")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Shared(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_match_constructors() {
        assert_eq!(
            Error::Shared(crate::Error::NonPositiveTime).to_string(),
            "duration must be positive"
        );
        assert_eq!(
            Error::Shared(crate::Error::InvalidHms).to_string(),
            "minutes and seconds must be less than 60"
        );
        assert_eq!(
            Error::Shared(crate::Error::AgeOutOfRange { min: 15, max: 90 }).to_string(),
            "age is outside the supported range (15–90)"
        );
        assert_eq!(
            Error::InvalidPower.to_string(),
            "power must be a positive finite number of watts"
        );
        assert_eq!(
            Error::InvalidMass.to_string(),
            "mass must be a positive finite number of kilograms"
        );
        assert_eq!(
            Error::InvalidWork.to_string(),
            "work must be a positive finite number of joules"
        );
        assert_eq!(
            Error::InsufficientEfforts.to_string(),
            "not enough maximal efforts for this model"
        );
        assert_eq!(
            Error::DurationOutOfModelRange.to_string(),
            "duration is outside the model's valid window"
        );
        assert_eq!(
            Error::UnsolvablePowerDuration.to_string(),
            "power–duration model could not be solved"
        );
        assert_eq!(
            Error::InvalidPhysicsParam.to_string(),
            "physics parameter is zero, negative, non-finite, or out of range"
        );
        assert_eq!(
            Error::UnsolvableSpeed.to_string(),
            "no speed in the solver bracket matches the given power"
        );
    }

    #[test]
    fn implements_std_error() {
        let err: Box<dyn StdError> = Box::new(Error::InvalidPower);
        assert!(err.source().is_none());
        let shared = Error::Shared(crate::Error::InvalidHms);
        assert!(shared.source().is_some());
        assert_eq!(
            shared.source().unwrap().to_string(),
            crate::Error::InvalidHms.to_string()
        );
    }

    #[test]
    fn from_shared() {
        let e: Error = crate::Error::InvalidHms.into();
        assert_eq!(e, Error::Shared(crate::Error::InvalidHms));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_error_roundtrip() {
        let json = serde_json::to_string(&Error::InvalidPower).unwrap();
        assert_eq!(
            serde_json::from_str::<Error>(&json).unwrap(),
            Error::InvalidPower
        );
        let shared = Error::Shared(crate::Error::AgeOutOfRange { min: 15, max: 90 });
        let shared_json = serde_json::to_string(&shared).unwrap();
        assert_eq!(serde_json::from_str::<Error>(&shared_json).unwrap(), shared);
    }
}
