use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use super::units::METERS_PER_MILE;
use crate::Error;

/// Road / track distance, including a caller-supplied custom length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distance {
    /// 3,000 metres (track-adjacent; no USATF MLDR 2025 road age-grade row).
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
    ///
    /// With the `serde` feature, only `meters` is serialized. Deserialization
    /// always uses the label `"custom"` (in-process labels from
    /// [`Distance::custom`] are not preserved across serde). This keeps
    /// [`Distance`] [`Copy`] without leaking strings.
    Custom {
        /// Length in metres.
        meters: f64,
        /// Short display label (for example `"8K"`). Not preserved by serde.
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
    ///
    /// ```
    /// use sportanalytics::running::Distance;
    ///
    /// assert_eq!(Distance::all()[0], Distance::ThreeK);
    /// ```
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
    ///
    /// ```
    /// use sportanalytics::running::Distance;
    ///
    /// let d = Distance::from_meters(1_500.0).unwrap();
    /// assert_eq!(d.meters(), 1_500.0);
    /// ```
    pub fn from_meters(meters: f64) -> Result<Self, Error> {
        Self::custom(meters, "custom")
    }

    /// A custom distance from kilometres.
    ///
    /// ```
    /// use sportanalytics::running::Distance;
    ///
    /// let eight = Distance::from_km(8.0).unwrap();
    /// assert_eq!(eight.meters(), 8_000.0);
    /// ```
    pub fn from_km(km: f64) -> Result<Self, Error> {
        Self::from_meters(km * 1_000.0)
    }

    /// A custom distance from international miles ([`METERS_PER_MILE`] m each).
    ///
    /// ```
    /// use sportanalytics::running::{Distance, METERS_PER_MILE};
    ///
    /// let one = Distance::from_miles(1.0).unwrap();
    /// assert_eq!(one.meters(), METERS_PER_MILE);
    /// ```
    pub fn from_miles(miles: f64) -> Result<Self, Error> {
        Self::from_meters(miles * METERS_PER_MILE)
    }

    /// A custom distance with a display label such as `"8K"` or `"10 mile"`.
    ///
    /// ```
    /// use sportanalytics::running::Distance;
    ///
    /// let eight = Distance::custom(8_000.0, "8K").unwrap();
    /// assert_eq!(eight.to_string(), "8K");
    /// ```
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

    /// Distance in kilometres.
    pub fn kilometers(self) -> f64 {
        self.meters() / 1_000.0
    }

    /// Distance in international miles.
    pub fn miles(self) -> f64 {
        self.meters() / METERS_PER_MILE
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

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for Distance {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let compact: String = s
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .flat_map(|c| c.to_lowercase())
            .collect();
        match compact.as_str() {
            "3k" | "3000" | "3000m" => Ok(Self::ThreeK),
            "5k" | "5000" | "5000m" => Ok(Self::FiveK),
            "10k" | "10000" | "10000m" => Ok(Self::TenK),
            "hm" | "half" | "halfmarathon" => Ok(Self::HalfMarathon),
            "fm" | "marathon" | "full" | "fullmarathon" => Ok(Self::Marathon),
            other => parse_length(other),
        }
    }
}

fn parse_length(s: &str) -> Result<Distance, Error> {
    // Longest mile suffixes first so `miles` / `mile` win over `mi`.
    if let Some(num) = s
        .strip_suffix("miles")
        .or_else(|| s.strip_suffix("mile"))
        .or_else(|| s.strip_suffix("mi"))
    {
        let miles: f64 = num.parse().map_err(|_| Error::UnrecognizedDistance)?;
        return Distance::from_miles(miles);
    }
    if let Some(num) = s.strip_suffix('m') {
        let meters: f64 = num.parse().map_err(|_| Error::UnrecognizedDistance)?;
        return Distance::from_meters(meters);
    }
    if let Some(num) = s.strip_suffix('k') {
        let km: f64 = num.parse().map_err(|_| Error::UnrecognizedDistance)?;
        return Distance::from_km(km);
    }
    let meters: f64 = s.parse().map_err(|_| Error::UnrecognizedDistance)?;
    Distance::from_meters(meters)
}

// Serde keeps meters only for Custom so Distance stays Copy without Box::leak.
// In-process labels from Distance::custom are not round-tripped.
#[cfg(feature = "serde")]
mod distance_serde {
    use super::Distance;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    enum DistanceDto {
        ThreeK,
        FiveK,
        TenK,
        HalfMarathon,
        Marathon,
        Custom { meters: f64 },
    }

    impl From<Distance> for DistanceDto {
        fn from(distance: Distance) -> Self {
            match distance {
                Distance::ThreeK => Self::ThreeK,
                Distance::FiveK => Self::FiveK,
                Distance::TenK => Self::TenK,
                Distance::HalfMarathon => Self::HalfMarathon,
                Distance::Marathon => Self::Marathon,
                Distance::Custom { meters, .. } => Self::Custom { meters },
            }
        }
    }

    impl TryFrom<DistanceDto> for Distance {
        type Error = crate::Error;

        fn try_from(distance: DistanceDto) -> Result<Self, Self::Error> {
            match distance {
                DistanceDto::ThreeK => Ok(Distance::ThreeK),
                DistanceDto::FiveK => Ok(Distance::FiveK),
                DistanceDto::TenK => Ok(Distance::TenK),
                DistanceDto::HalfMarathon => Ok(Distance::HalfMarathon),
                DistanceDto::Marathon => Ok(Distance::Marathon),
                DistanceDto::Custom { meters } => Distance::from_meters(meters),
            }
        }
    }

    impl Serialize for Distance {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            DistanceDto::from(*self).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Distance {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            DistanceDto::deserialize(deserializer)?
                .try_into()
                .map_err(serde::de::Error::custom)
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

    #[test]
    fn display_uses_label() {
        assert_eq!(Distance::FiveK.to_string(), "5K");
        assert_eq!(Distance::custom(8_000.0, "8K").unwrap().to_string(), "8K");
    }

    #[test]
    fn from_str_named_aliases() {
        assert_eq!("5K".parse::<Distance>().unwrap(), Distance::FiveK);
        assert_eq!("hm".parse::<Distance>().unwrap(), Distance::HalfMarathon);
        assert_eq!("marathon".parse::<Distance>().unwrap(), Distance::Marathon);
        assert_eq!("10k".parse::<Distance>().unwrap(), Distance::TenK);
        assert_eq!("3K".parse::<Distance>().unwrap(), Distance::ThreeK);
    }

    #[test]
    fn from_str_custom_length() {
        let eight = "8k".parse::<Distance>().unwrap();
        assert_eq!(eight.meters(), 8_000.0);
        let mile = "16093.4m".parse::<Distance>().unwrap();
        assert!((mile.meters() - 16_093.4).abs() < 1e-9);
        assert_eq!("nope".parse::<Distance>(), Err(Error::UnrecognizedDistance));
    }

    #[test]
    fn from_km_and_from_miles() {
        assert_eq!(Distance::from_km(8.0).unwrap().meters(), 8_000.0);
        assert_eq!(Distance::from_miles(1.0).unwrap().meters(), METERS_PER_MILE);
        assert_eq!(Distance::from_miles(0.0), Err(Error::InvalidDistance));
        assert_eq!(Distance::from_km(f64::NAN), Err(Error::InvalidDistance));
    }

    #[test]
    fn kilometers_and_miles_accessors() {
        assert_eq!(Distance::FiveK.kilometers(), 5.0);
        assert!((Distance::from_miles(8.0).unwrap().miles() - 8.0).abs() < 1e-12);
    }

    #[test]
    fn from_str_mile_suffixes() {
        let eight = "8mi".parse::<Distance>().unwrap();
        assert!((eight.meters() - 8.0 * METERS_PER_MILE).abs() < 1e-9);
        assert_eq!(
            "8mile".parse::<Distance>().unwrap().meters(),
            eight.meters()
        );
        assert_eq!(
            "10miles".parse::<Distance>().unwrap().meters(),
            Distance::from_miles(10.0).unwrap().meters()
        );
        let half = "13.1mi".parse::<Distance>().unwrap();
        assert!((half.meters() - 13.1 * METERS_PER_MILE).abs() < 1e-9);
        // Zero length parses as a number but fails the positive-distance check.
        assert_eq!("0mi".parse::<Distance>(), Err(Error::InvalidDistance));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_named_and_custom_roundtrip() {
        let json = serde_json::to_string(&Distance::HalfMarathon).unwrap();
        assert_eq!(
            serde_json::from_str::<Distance>(&json).unwrap(),
            Distance::HalfMarathon
        );
        let eight = Distance::custom(8_000.0, "8K").unwrap();
        let encoded = serde_json::to_string(&eight).unwrap();
        assert_eq!(encoded, r#"{"Custom":{"meters":8000.0}}"#);
        let back: Distance = serde_json::from_str(&encoded).unwrap();
        assert_eq!(back.meters(), 8_000.0);
        assert_eq!(back.label(), "custom");
    }
}
