//! Course, stroke, and sex for pool swimming.

use std::fmt;

/// Pool course. Distances in constructors stay metres internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Course {
    /// 50 m long-course metres (LCM).
    Lcm,
    /// 25 m short-course metres (SCM).
    Scm,
    /// 25 yd short-course yards (SCY). Display uses /100y.
    Scy,
}

impl Course {
    /// Length of one pool length in metres.
    pub fn length_metres(self) -> f64 {
        match self {
            Course::Lcm => 50.0,
            Course::Scm => 25.0,
            Course::Scy => 25.0 * crate::swimming::METERS_PER_YARD,
        }
    }
}

impl fmt::Display for Course {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Course::Lcm => "LCM",
            Course::Scm => "SCM",
            Course::Scy => "SCY",
        })
    }
}

/// Competitive stroke. CSS/zones may ignore this; points need it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Stroke {
    /// Freestyle.
    Free,
    /// Backstroke.
    Back,
    /// Breaststroke.
    Breast,
    /// Butterfly.
    Fly,
    /// Individual medley.
    Im,
}

impl fmt::Display for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Stroke::Free => "Free",
            Stroke::Back => "Back",
            Stroke::Breast => "Breast",
            Stroke::Fly => "Fly",
            Stroke::Im => "IM",
        })
    }
}

/// Sex column for World Aquatics base times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Sex {
    /// Female / women base-time column.
    Female,
    /// Male / men base-time column.
    Male,
}

impl fmt::Display for Sex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Sex::Female => "Female",
            Sex::Male => "Male",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swimming::METERS_PER_YARD;

    #[test]
    fn course_lengths() {
        assert_eq!(Course::Lcm.length_metres(), 50.0);
        assert_eq!(Course::Scm.length_metres(), 25.0);
        assert!((Course::Scy.length_metres() - 25.0 * METERS_PER_YARD).abs() < 1e-12);
    }

    #[test]
    fn display_labels() {
        assert_eq!(Course::Lcm.to_string(), "LCM");
        assert_eq!(Course::Scm.to_string(), "SCM");
        assert_eq!(Course::Scy.to_string(), "SCY");
        assert_eq!(Stroke::Free.to_string(), "Free");
        assert_eq!(Stroke::Im.to_string(), "IM");
        assert_eq!(Sex::Male.to_string(), "Male");
    }
}
