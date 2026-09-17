use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use super::units::METERS_PER_YARD;
use crate::Error;

/// Named pool event or a caller-supplied custom length (metres internally).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// 50 metres.
    M50,
    /// 100 metres.
    M100,
    /// 200 metres.
    M200,
    /// 400 metres.
    M400,
    /// 800 metres.
    M800,
    /// 1500 metres.
    M1500,
    /// 50 yards.
    Y50,
    /// 100 yards.
    Y100,
    /// 200 yards.
    Y200,
    /// 500 yards.
    Y500,
    /// 1000 yards.
    Y1000,
    /// 1650 yards.
    Y1650,
    /// Any positive finite length. Prefer [`Event::from_meters`] or
    /// [`Event::custom`] so invalid values are rejected.
    ///
    /// With the `serde` feature, only `meters` is serialized. Deserialization
    /// always uses the label `"custom"` (in-process labels from
    /// [`Event::custom`] are not preserved across serde).
    Custom {
        /// Length in metres.
        meters: f64,
        /// Short display label (for example `"300m"`). Not preserved by serde.
        label: &'static str,
    },
}

impl Eq for Event {}

impl Hash for Event {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        if let Self::Custom { meters, label } = *self {
            meters.to_bits().hash(state);
            label.hash(state);
        }
    }
}

impl Event {
    /// Named metric events, shortest to longest.
    pub const fn metric_named() -> [Event; 6] {
        [
            Self::M50,
            Self::M100,
            Self::M200,
            Self::M400,
            Self::M800,
            Self::M1500,
        ]
    }

    /// Named SCY events, shortest to longest.
    pub const fn scy_named() -> [Event; 6] {
        [
            Self::Y50,
            Self::Y100,
            Self::Y200,
            Self::Y500,
            Self::Y1000,
            Self::Y1650,
        ]
    }

    /// Snap exact metric metres to a named event; otherwise a custom length.
    ///
    /// ```
    /// use sportanalytics::swimming::Event;
    ///
    /// assert_eq!(Event::from_meters(100.0).unwrap(), Event::M100);
    /// assert_eq!(Event::from_meters(300.0).unwrap().meters(), 300.0);
    /// ```
    pub fn from_meters(meters: f64) -> Result<Self, Error> {
        if !(meters.is_finite() && meters > 0.0) {
            return Err(Error::InvalidDistance);
        }
        Ok(match meters {
            m if (m - 50.0).abs() < 1e-9 => Self::M50,
            m if (m - 100.0).abs() < 1e-9 => Self::M100,
            m if (m - 200.0).abs() < 1e-9 => Self::M200,
            m if (m - 400.0).abs() < 1e-9 => Self::M400,
            m if (m - 800.0).abs() < 1e-9 => Self::M800,
            m if (m - 1500.0).abs() < 1e-9 => Self::M1500,
            _ => Self::Custom {
                meters,
                label: "custom",
            },
        })
    }

    /// Convert yards to metres and snap exact SCY named events.
    pub fn from_yards(yards: f64) -> Result<Self, Error> {
        if !(yards.is_finite() && yards > 0.0) {
            return Err(Error::InvalidDistance);
        }
        Ok(match yards {
            y if (y - 50.0).abs() < 1e-9 => Self::Y50,
            y if (y - 100.0).abs() < 1e-9 => Self::Y100,
            y if (y - 200.0).abs() < 1e-9 => Self::Y200,
            y if (y - 500.0).abs() < 1e-9 => Self::Y500,
            y if (y - 1000.0).abs() < 1e-9 => Self::Y1000,
            y if (y - 1650.0).abs() < 1e-9 => Self::Y1650,
            _ => Self::Custom {
                meters: yards * METERS_PER_YARD,
                label: "custom",
            },
        })
    }

    /// A custom distance with a display label such as `"300m"`.
    pub fn custom(meters: f64, label: &'static str) -> Result<Self, Error> {
        if meters.is_finite() && meters > 0.0 {
            Ok(Self::Custom { meters, label })
        } else {
            Err(Error::InvalidDistance)
        }
    }

    /// Distance in metres.
    pub fn meters(self) -> f64 {
        match self {
            Self::M50 => 50.0,
            Self::M100 => 100.0,
            Self::M200 => 200.0,
            Self::M400 => 400.0,
            Self::M800 => 800.0,
            Self::M1500 => 1500.0,
            Self::Y50 => 50.0 * METERS_PER_YARD,
            Self::Y100 => 100.0 * METERS_PER_YARD,
            Self::Y200 => 200.0 * METERS_PER_YARD,
            Self::Y500 => 500.0 * METERS_PER_YARD,
            Self::Y1000 => 1000.0 * METERS_PER_YARD,
            Self::Y1650 => 1650.0 * METERS_PER_YARD,
            Self::Custom { meters, .. } => meters,
        }
    }

    /// Distance in international yards.
    pub fn yards(self) -> f64 {
        self.meters() / METERS_PER_YARD
    }

    /// Short label used in display output.
    pub const fn label(self) -> &'static str {
        match self {
            Self::M50 => "50m",
            Self::M100 => "100m",
            Self::M200 => "200m",
            Self::M400 => "400m",
            Self::M800 => "800m",
            Self::M1500 => "1500m",
            Self::Y50 => "50y",
            Self::Y100 => "100y",
            Self::Y200 => "200y",
            Self::Y500 => "500y",
            Self::Y1000 => "1000y",
            Self::Y1650 => "1650y",
            Self::Custom { label, .. } => label,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for Event {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let compact: String = s
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .flat_map(|c| c.to_lowercase())
            .collect();
        match compact.as_str() {
            "50m" | "50" => Ok(Self::M50),
            "100m" | "100" => Ok(Self::M100),
            "200m" | "200" => Ok(Self::M200),
            "400m" | "400" => Ok(Self::M400),
            "800m" | "800" => Ok(Self::M800),
            "1500m" | "1500" => Ok(Self::M1500),
            "50y" | "50yd" | "50yards" => Ok(Self::Y50),
            "100y" | "100yd" | "100yards" => Ok(Self::Y100),
            "200y" | "200yd" | "200yards" => Ok(Self::Y200),
            "500y" | "500yd" | "500yards" => Ok(Self::Y500),
            "1000y" | "1000yd" | "1000yards" => Ok(Self::Y1000),
            "1650y" | "1650yd" | "1650yards" => Ok(Self::Y1650),
            other => parse_length(other),
        }
    }
}

fn parse_length(s: &str) -> Result<Event, Error> {
    if let Some(num) = s
        .strip_suffix("yards")
        .or_else(|| s.strip_suffix("yard"))
        .or_else(|| s.strip_suffix("yd"))
        .or_else(|| s.strip_suffix('y'))
    {
        let yards: f64 = num.parse().map_err(|_| Error::UnrecognizedDistance)?;
        return Event::from_yards(yards);
    }
    if let Some(num) = s.strip_suffix('m') {
        let meters: f64 = num.parse().map_err(|_| Error::UnrecognizedDistance)?;
        return Event::from_meters(meters);
    }
    let meters: f64 = s.parse().map_err(|_| Error::UnrecognizedDistance)?;
    Event::from_meters(meters)
}

#[cfg(feature = "serde")]
mod event_serde {
    use super::Event;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    enum EventDto {
        M50,
        M100,
        M200,
        M400,
        M800,
        M1500,
        Y50,
        Y100,
        Y200,
        Y500,
        Y1000,
        Y1650,
        Custom { meters: f64 },
    }

    impl From<Event> for EventDto {
        fn from(event: Event) -> Self {
            match event {
                Event::M50 => Self::M50,
                Event::M100 => Self::M100,
                Event::M200 => Self::M200,
                Event::M400 => Self::M400,
                Event::M800 => Self::M800,
                Event::M1500 => Self::M1500,
                Event::Y50 => Self::Y50,
                Event::Y100 => Self::Y100,
                Event::Y200 => Self::Y200,
                Event::Y500 => Self::Y500,
                Event::Y1000 => Self::Y1000,
                Event::Y1650 => Self::Y1650,
                Event::Custom { meters, .. } => Self::Custom { meters },
            }
        }
    }

    impl TryFrom<EventDto> for Event {
        type Error = crate::Error;

        fn try_from(event: EventDto) -> Result<Self, Self::Error> {
            match event {
                EventDto::M50 => Ok(Event::M50),
                EventDto::M100 => Ok(Event::M100),
                EventDto::M200 => Ok(Event::M200),
                EventDto::M400 => Ok(Event::M400),
                EventDto::M800 => Ok(Event::M800),
                EventDto::M1500 => Ok(Event::M1500),
                EventDto::Y50 => Ok(Event::Y50),
                EventDto::Y100 => Ok(Event::Y100),
                EventDto::Y200 => Ok(Event::Y200),
                EventDto::Y500 => Ok(Event::Y500),
                EventDto::Y1000 => Ok(Event::Y1000),
                EventDto::Y1650 => Ok(Event::Y1650),
                EventDto::Custom { meters } => Event::from_meters(meters),
            }
        }
    }

    impl Serialize for Event {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            EventDto::from(*self).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Event {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            EventDto::deserialize(deserializer)?
                .try_into()
                .map_err(serde::de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_and_scy_metres() {
        assert_eq!(Event::M100.meters(), 100.0);
        assert_eq!(Event::M1500.meters(), 1500.0);
        assert!((Event::Y1650.meters() - 1650.0 * METERS_PER_YARD).abs() < 1e-12);
        assert!((Event::Y100.yards() - 100.0).abs() < 1e-12);
    }

    #[test]
    fn from_meters_snaps_named() {
        assert_eq!(Event::from_meters(200.0).unwrap(), Event::M200);
        let custom = Event::from_meters(300.0).unwrap();
        assert_eq!(custom.meters(), 300.0);
        assert_eq!(custom.label(), "custom");
    }

    #[test]
    fn from_yards_snaps_named() {
        assert_eq!(Event::from_yards(500.0).unwrap(), Event::Y500);
        assert_eq!(Event::from_meters(0.0), Err(Error::InvalidDistance));
        assert_eq!(Event::from_yards(-1.0), Err(Error::InvalidDistance));
    }

    #[test]
    fn from_str_aliases() {
        assert_eq!("100m".parse::<Event>().unwrap(), Event::M100);
        assert_eq!("1500".parse::<Event>().unwrap(), Event::M1500);
        assert_eq!("1650y".parse::<Event>().unwrap(), Event::Y1650);
        assert_eq!("500yd".parse::<Event>().unwrap(), Event::Y500);
        assert_eq!("nope".parse::<Event>(), Err(Error::UnrecognizedDistance));
    }

    #[test]
    fn labels_are_stable() {
        assert_eq!(Event::M50.label(), "50m");
        assert_eq!(Event::Y1650.label(), "1650y");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_named_and_custom_roundtrip() {
        let json = serde_json::to_string(&Event::M400).unwrap();
        assert_eq!(serde_json::from_str::<Event>(&json).unwrap(), Event::M400);
        let custom = Event::custom(300.0, "300m").unwrap();
        let encoded = serde_json::to_string(&custom).unwrap();
        assert_eq!(encoded, r#"{"Custom":{"meters":300.0}}"#);
        let back: Event = serde_json::from_str(&encoded).unwrap();
        assert_eq!(back.meters(), 300.0);
        assert_eq!(back.label(), "custom");
    }
}
