use std::error::Error as StdError;
use std::fmt;

/// Errors returned by constructors and fallible analytics helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// A race time was zero, negative, or non-finite.
    NonPositiveTime,
    /// [`crate::running::vo2max_from_races`] was called with an empty slice.
    EmptyRaces,
    /// A VDOT value was non-positive or non-finite.
    InvalidVdot,
    /// A custom distance was zero, negative, or non-finite.
    InvalidDistance,
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
    }

    #[test]
    fn implements_std_error() {
        let err: Box<dyn StdError> = Box::new(Error::EmptyRaces);
        assert!(err.source().is_none());
    }
}
