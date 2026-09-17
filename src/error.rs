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
    ///
    /// Cycling age helpers use a narrower window (15..=90); see
    /// [`crate::cycling::age_factor`].
    AgeOutOfRange,
    /// Distance has no official age-grade row and cannot be interpolated
    /// (outside the official span, or an unsupported named distance such as 3K).
    UnsupportedAgeGradeDistance,
    /// Power was zero, negative, or non-finite.
    InvalidPower,
    /// Mass was zero, negative, or non-finite.
    InvalidMass,
    /// Work (joules) was zero, negative, or non-finite.
    InvalidWork,
    /// Critical-power fit or CP prediction needs at least two maximal efforts.
    InsufficientEfforts,
    /// Effort or target duration is outside the model's valid window.
    DurationOutOfModelRange,
    /// Power–duration model could not be solved (non-positive CP/W′, or P ≤ CP).
    UnsolvablePowerDuration,
    /// A physics parameter (CdA, Crr, η, air density, speed) is out of range.
    InvalidPhysicsParam,
    /// Power↔speed bisection did not bracket a root.
    UnsolvableSpeed,
    /// Two CSS trials used the same distance, or a non-positive distance gap.
    TrialsSameDistance,
    /// The longer trial was not slower than the shorter trial (`T_long <= T_short`).
    TrialsNotOrdered,
    /// CSS slope was zero, negative, or non-finite.
    UnsolvableCss,
    /// World Aquatics points requested for an event/course/sex with no base time.
    UnsupportedWaEvent,
    /// Stroke count was zero, negative, or non-finite (SWOLF / DPS).
    InvalidStrokeCount,
    /// Hundredths were ≥ 100 in a swim `from_hms_cents` constructor.
    InvalidCents,
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
            Self::InvalidPower => f.write_str("power must be a positive finite number of watts"),
            Self::InvalidMass => f.write_str("mass must be a positive finite number of kilograms"),
            Self::InvalidWork => f.write_str("work must be a positive finite number of joules"),
            Self::InsufficientEfforts => {
                f.write_str("at least two maximal efforts are required for critical power")
            }
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
            Self::TrialsSameDistance => {
                f.write_str("CSS trials must use distinct positive distances")
            }
            Self::TrialsNotOrdered => {
                f.write_str("longer CSS trial must be slower than the shorter trial")
            }
            Self::UnsolvableCss => f.write_str("CSS slope was zero, negative, or non-finite"),
            Self::UnsupportedWaEvent => {
                f.write_str("no World Aquatics base time for this event/course/sex")
            }
            Self::InvalidStrokeCount => f.write_str("stroke count must be a positive finite value"),
            Self::InvalidCents => f.write_str("hundredths must be less than 100"),
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
            "at least two maximal efforts are required for critical power"
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
        assert_eq!(
            Error::TrialsSameDistance.to_string(),
            "CSS trials must use distinct positive distances"
        );
        assert_eq!(
            Error::TrialsNotOrdered.to_string(),
            "longer CSS trial must be slower than the shorter trial"
        );
        assert_eq!(
            Error::UnsolvableCss.to_string(),
            "CSS slope was zero, negative, or non-finite"
        );
        assert_eq!(
            Error::UnsupportedWaEvent.to_string(),
            "no World Aquatics base time for this event/course/sex"
        );
        assert_eq!(
            Error::InvalidStrokeCount.to_string(),
            "stroke count must be a positive finite value"
        );
        assert_eq!(
            Error::InvalidCents.to_string(),
            "hundredths must be less than 100"
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
        let cycling = serde_json::to_string(&Error::InvalidPower).unwrap();
        assert_eq!(
            serde_json::from_str::<Error>(&cycling).unwrap(),
            Error::InvalidPower
        );
    }
}
