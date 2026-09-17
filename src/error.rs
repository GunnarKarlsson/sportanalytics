use std::error::Error as StdError;
use std::fmt;

/// Shared input failures constructed by more than one sport for the same reason.
///
/// Sport-specific model failures live on [`crate::running::Error`] and
/// [`crate::cycling::Error`]. Analytics functions return the sport’s error type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    /// A duration was zero, negative, or non-finite.
    NonPositiveTime,
    /// `from_hms` was given minutes or seconds ≥ 60.
    InvalidHms,
    /// Age is outside the supported range for the helper that was called.
    ///
    /// Running age grading uses 5..=99; cycling age helpers use 15..=90.
    AgeOutOfRange {
        /// Inclusive lower bound for the helper.
        min: u16,
        /// Inclusive upper bound for the helper.
        max: u16,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositiveTime => f.write_str("duration must be positive"),
            Self::InvalidHms => f.write_str("minutes and seconds must be less than 60"),
            Self::AgeOutOfRange { min, max } => {
                write!(f, "age is outside the supported range ({min}–{max})")
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
            "duration must be positive"
        );
        assert_eq!(
            Error::InvalidHms.to_string(),
            "minutes and seconds must be less than 60"
        );
        assert_eq!(
            Error::AgeOutOfRange { min: 5, max: 99 }.to_string(),
            "age is outside the supported range (5–99)"
        );
        assert_eq!(
            Error::AgeOutOfRange { min: 15, max: 90 }.to_string(),
            "age is outside the supported range (15–90)"
        );
    }

    #[test]
    fn implements_std_error() {
        let err: Box<dyn StdError> = Box::new(Error::NonPositiveTime);
        assert!(err.source().is_none());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_error_roundtrip() {
        let json = serde_json::to_string(&Error::NonPositiveTime).unwrap();
        assert_eq!(
            serde_json::from_str::<Error>(&json).unwrap(),
            Error::NonPositiveTime
        );
        let age = Error::AgeOutOfRange { min: 5, max: 99 };
        let age_json = serde_json::to_string(&age).unwrap();
        assert_eq!(serde_json::from_str::<Error>(&age_json).unwrap(), age);
    }
}
