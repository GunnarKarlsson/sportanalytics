/// Supported road / track distances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

impl Distance {
    /// Official distance in metres.
    pub const fn meters(self) -> f64 {
        match self {
            Self::ThreeK => 3_000.0,
            Self::FiveK => 5_000.0,
            Self::TenK => 10_000.0,
            Self::HalfMarathon => 21_097.5,
            Self::Marathon => 42_195.0,
        }
    }

    /// Every supported distance, shortest to longest.
    pub const fn all() -> [Distance; 5] {
        [
            Self::ThreeK,
            Self::FiveK,
            Self::TenK,
            Self::HalfMarathon,
            Self::Marathon,
        ]
    }

    /// Short label used in display output (`3K`, `5K`, `10K`, `HM`, `FM`).
    pub const fn label(self) -> &'static str {
        match self {
            Self::ThreeK => "3K",
            Self::FiveK => "5K",
            Self::TenK => "10K",
            Self::HalfMarathon => "HM",
            Self::Marathon => "FM",
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
}
