use std::hash::{Hash, Hasher};

use crate::Error;

/// Road / track distance, including a caller-supplied custom length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distance {
    /// 3,000 metres (track-adjacent; there is no official road 3K standard).
    ThreeK,
    /// 5,000 metres.
    FiveK,
    /// 10,000 metres.
    TenK,
    /// Half marathon, 21,097.5 metres.
    HalfMarathon,
    /// Marathon, 42,195 metres.
    Marathon,
    /// Any positive finite length. Prefer [`Distance::from_meters`] or
    /// [`Distance::custom`] so invalid values are rejected.
    Custom {
        /// Length in metres.
        meters: f64,
        /// Short display label (for example `"8K"`).
        label: &'static str,
    },
}

impl Eq for Distance {}

impl Hash for Distance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        if let Self::Custom { meters, label } = *self {
            meters.to_bits().hash(state);
            label.hash(state);
        }
    }
}

impl Distance {
    /// Named road / track distances, shortest to longest.
    pub const fn all() -> [Distance; 5] {
        [
            Self::ThreeK,
            Self::FiveK,
            Self::TenK,
            Self::HalfMarathon,
            Self::Marathon,
        ]
    }

    /// A custom distance labelled `"custom"`.
    pub fn from_meters(meters: f64) -> Result<Self, Error> {
        Self::custom(meters, "custom")
    }

    /// A custom distance with a display label such as `"8K"` or `"10 mile"`.
    pub fn custom(meters: f64, label: &'static str) -> Result<Self, Error> {
        if meters.is_finite() && meters > 0.0 {
            Ok(Self::Custom { meters, label })
        } else {
            Err(Error::InvalidDistance)
        }
    }

    /// Distance in metres.
    pub const fn meters(self) -> f64 {
        match self {
            Self::ThreeK => 3_000.0,
            Self::FiveK => 5_000.0,
            Self::TenK => 10_000.0,
            Self::HalfMarathon => 21_097.5,
            Self::Marathon => 42_195.0,
            Self::Custom { meters, .. } => meters,
        }
    }

    /// Short label used in display output (`3K`, `5K`, `10K`, `HM`, `FM`, or the
    /// custom label).
    pub const fn label(self) -> &'static str {
        match self {
            Self::ThreeK => "3K",
            Self::FiveK => "5K",
            Self::TenK => "10K",
            Self::HalfMarathon => "HM",
            Self::Marathon => "FM",
            Self::Custom { label, .. } => label,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meters_match_official_road_distances() {
        assert_eq!(Distance::ThreeK.meters(), 3_000.0);
        assert_eq!(Distance::FiveK.meters(), 5_000.0);
        assert_eq!(Distance::TenK.meters(), 10_000.0);
        assert_eq!(Distance::HalfMarathon.meters(), 21_097.5);
        assert_eq!(Distance::Marathon.meters(), 42_195.0);
    }

    #[test]
    fn all_is_shortest_to_longest() {
        let all = Distance::all();
        assert_eq!(all.len(), 5);
        for pair in all.windows(2) {
            assert!(pair[0].meters() < pair[1].meters());
        }
    }

    #[test]
    fn labels_are_stable() {
        assert_eq!(Distance::ThreeK.label(), "3K");
        assert_eq!(Distance::FiveK.label(), "5K");
        assert_eq!(Distance::TenK.label(), "10K");
        assert_eq!(Distance::HalfMarathon.label(), "HM");
        assert_eq!(Distance::Marathon.label(), "FM");
    }

    #[test]
    fn custom_from_meters() {
        let eight = Distance::custom(8_000.0, "8K").unwrap();
        assert_eq!(eight.meters(), 8_000.0);
        assert_eq!(eight.label(), "8K");
        let anon = Distance::from_meters(1_500.0).unwrap();
        assert_eq!(anon.label(), "custom");
        assert_eq!(anon.meters(), 1_500.0);
    }

    #[test]
    fn custom_rejects_non_positive() {
        assert_eq!(Distance::from_meters(0.0), Err(Error::InvalidDistance));
        assert_eq!(Distance::from_meters(-1.0), Err(Error::InvalidDistance));
        assert_eq!(Distance::from_meters(f64::NAN), Err(Error::InvalidDistance));
        assert_eq!(
            Distance::from_meters(f64::INFINITY),
            Err(Error::InvalidDistance)
        );
    }
}
