use std::error::Error as StdError;
use std::fmt;

/// Errors returned by constructors and fallible analytics helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    /// A race time was zero, negative, or non-finite.
    NonPositiveTime,
    /// [`crate::running::vo2max_from_races`] was called with an empty slice.
    EmptyRaces,
    /// A VDOT value was non-positive or non-finite.
    InvalidVdot,
    /// A custom distance was zero, negative, or non-finite.
    InvalidDistance,
    /// A distance string was not a named distance or a positive length.
    UnrecognizedDistance,
    /// `from_hms` was given minutes or seconds ≥ 60.
    InvalidHms,
    /// A pace value was zero, negative, or non-finite.
    InvalidPace,
    /// Daniels inversion found no finish time in the 2–12 min/km pace bracket.
    UnsolvableTime,
    /// Age is outside the published table range (USATF MLDR 2025: 5..=99).
    AgeOutOfRange,
    /// Distance has no official age-grade row and cannot be interpolated
    /// (outside the official span, or an unsupported named distance such as 3K).
    UnsupportedAgeGradeDistance,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositiveTime => f.write_str("race time must be positive"),
            Self::EmptyRaces => f.write_str("at least one race time is required"),
            Self::InvalidVdot => f.write_str("VDOT must be a positive finite value"),
            Self::InvalidDistance => {
                f.write_str("distance must be a positive finite number of metres")
            }
            Self::UnrecognizedDistance => f.write_str("unrecognized distance"),
            Self::InvalidHms => f.write_str("minutes and seconds must be less than 60"),
            Self::InvalidPace => f.write_str("pace must be a positive finite value"),
            Self::UnsolvableTime => {
                f.write_str("no finish time in the 2–12 min/km VDOT solver bracket")
            }
            Self::AgeOutOfRange => {
                f.write_str("age must be within the published age-grade table range (5–99)")
            }
            Self::UnsupportedAgeGradeDistance => {
                f.write_str("distance is not supported for age grading")
            }
        }
    }
}

impl StdError for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_match_constructors() {
        assert_eq!(
            Error::NonPositiveTime.to_string(),
            "race time must be positive"
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
            Error::AgeOutOfRange.to_string(),
            "age must be within the published age-grade table range (5–99)"
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
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_error_roundtrip() {
        let json = serde_json::to_string(&Error::EmptyRaces).unwrap();
        assert_eq!(
            serde_json::from_str::<Error>(&json).unwrap(),
            Error::EmptyRaces
        );
    }
}
