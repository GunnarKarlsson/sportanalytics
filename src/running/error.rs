use std::error::Error as StdError;
use std::fmt;

/// Failures from running constructors and analytics helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    /// Shared input failure (time / HMS / age range).
    Shared(crate::Error),
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

impl From<crate::Error> for Error {
    fn from(e: crate::Error) -> Self {
        Self::Shared(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shared(e) => fmt::Display::fmt(e, f),
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

impl StdError for Error {}

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
            Error::Shared(crate::Error::InvalidHms).to_string(),
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
            Error::Shared(crate::Error::AgeOutOfRange { min: 5, max: 99 }).to_string(),
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
    }

    #[test]
    fn from_shared() {
        let e: Error = crate::Error::NonPositiveTime.into();
        assert_eq!(e, Error::Shared(crate::Error::NonPositiveTime));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_error_roundtrip() {
        let json = serde_json::to_string(&Error::EmptyRaces).unwrap();
        assert_eq!(
            serde_json::from_str::<Error>(&json).unwrap(),
            Error::EmptyRaces
        );
        let shared = Error::Shared(crate::Error::AgeOutOfRange { min: 5, max: 99 });
        let shared_json = serde_json::to_string(&shared).unwrap();
        assert_eq!(serde_json::from_str::<Error>(&shared_json).unwrap(), shared);
    }
}
