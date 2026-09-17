use std::error::Error as StdError;
use std::fmt;

/// Failures from running constructors and analytics helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    /// A duration was zero, negative, or non-finite.
    NonPositiveTime,
    /// `from_hms` was given minutes or seconds ≥ 60.
    InvalidHms,
    /// Age is outside the supported range for the helper that was called (5..=99).
    AgeOutOfRange {
        /// Inclusive lower bound for the helper.
        min: u16,
        /// Inclusive upper bound for the helper.
        max: u16,
    },
    /// [`super::vo2max_from_races`] was called with an empty slice.
    EmptyRaces,
    /// A VDOT value was non-positive or non-finite.
    InvalidVdot,
    /// A custom distance was zero, negative, or non-finite.
    InvalidDistance,
    /// A distance string was not a named distance or a positive length.
    UnrecognizedDistance,
    /// A pace value was zero, negative, or non-finite.
    InvalidPace,
    /// Daniels inversion found no finish time in the 2–12 min/km pace bracket.
    UnsolvableTime,
    /// Distance has no official age-grade row and cannot be interpolated
    /// (outside the official span, or an unsupported named distance such as 3K).
    UnsupportedAgeGradeDistance,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositiveTime => f.write_str("duration must be positive"),
            Self::InvalidHms => f.write_str("minutes and seconds must be less than 60"),
            Self::AgeOutOfRange { min, max } => {
                write!(f, "age is outside the supported range ({min}–{max})")
            }
            Self::EmptyRaces => f.write_str("at least one race time is required"),
            Self::InvalidVdot => f.write_str("VDOT must be a positive finite value"),
            Self::InvalidDistance => {
                f.write_str("distance must be a positive finite number of metres")
            }
            Self::UnrecognizedDistance => f.write_str("unrecognized distance"),
            Self::InvalidPace => f.write_str("pace must be a positive finite value"),
            Self::UnsolvableTime => {
                f.write_str("no finish time in the 2–12 min/km VDOT solver bracket")
            }
            Self::UnsupportedAgeGradeDistance => {
                f.write_str("distance is not supported for age grading")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_match_constructors() {
        assert_eq!(
            Error::NonPositiveTime.to_string(),
            "duration must be positive"
        );
        assert_eq!(
            Error::EmptyRaces.to_string(),
            "at least one race time is required"
        );
        assert_eq!(
            Error::InvalidVdot.to_string(),
            "VDOT must be a positive finite value"
        );
        assert_eq!(
            Error::InvalidDistance.to_string(),
            "distance must be a positive finite number of metres"
        );
        assert_eq!(
            Error::UnrecognizedDistance.to_string(),
            "unrecognized distance"
        );
        assert_eq!(
            Error::InvalidHms.to_string(),
            "minutes and seconds must be less than 60"
        );
        assert_eq!(
            Error::InvalidPace.to_string(),
            "pace must be a positive finite value"
        );
        assert_eq!(
            Error::UnsolvableTime.to_string(),
            "no finish time in the 2–12 min/km VDOT solver bracket"
        );
        assert_eq!(
            Error::AgeOutOfRange { min: 5, max: 99 }.to_string(),
            "age is outside the supported range (5–99)"
        );
        assert_eq!(
            Error::UnsupportedAgeGradeDistance.to_string(),
            "distance is not supported for age grading"
        );
    }

    #[test]
    fn implements_std_error() {
        let err: Box<dyn StdError> = Box::new(Error::EmptyRaces);
        assert!(err.source().is_none());
        assert!(Error::NonPositiveTime.source().is_none());
        assert!(Error::AgeOutOfRange { min: 5, max: 99 }.source().is_none());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_error_roundtrip() {
        let json = serde_json::to_string(&Error::EmptyRaces).unwrap();
        assert_eq!(
            serde_json::from_str::<Error>(&json).unwrap(),
            Error::EmptyRaces
        );
        let age = Error::AgeOutOfRange { min: 5, max: 99 };
        let age_json = serde_json::to_string(&age).unwrap();
        assert_eq!(serde_json::from_str::<Error>(&age_json).unwrap(), age);
    }
}
