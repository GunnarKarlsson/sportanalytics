//! Road-cycling power ↔ speed (Martin et al. 1998 balance).
//!
//! ```text
//! P_pedal = (1/η) * [
//!     ½ CdA ρ (v+vw)² v
//!   + Crr m g cosθ · v
//!   + m g sinθ · v
//! ]
//! ```
//!
//! Steady-state only: **no** kinetic-energy / acceleration term, **no** drafting,
//! **no** bearing / drivetrain split beyond η. `θ = arctan(grade)` (0.06 = 6%).
//! Positive `wind_ms` is a headwind.
//!
//! [`Error::UnsolvableSpeed`] on downhill when required pedal power is ≤ 0 is
//! intentional — this crate has no coasting model.

use std::time::Duration;

use super::effort::{Mass, Power, WattsPerKg};
use crate::Error;

/// Standard gravity (m/s²).
pub const G: f64 = 9.80665;
/// Default CdA for drops / hoods (m²).
pub const DEFAULT_CDA: f64 = 0.30;
/// Default rolling resistance coefficient.
pub const DEFAULT_CRR: f64 = 0.004;
/// Default drivetrain efficiency.
pub const DEFAULT_ETA: f64 = 0.97;
/// Sea-level air density (kg/m³).
pub const SEA_LEVEL_RHO: f64 = 1.225;

/// Rider + bike aerodynamic and mechanical parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiderBike {
    /// Total mass (rider + bike + kit) in kilograms.
    pub mass: Mass,
    /// Drag area CdA in m².
    pub cda: f64,
    /// Rolling resistance coefficient Crr.
    pub crr: f64,
    /// Drivetrain efficiency η (e.g. 0.97).
    pub drivetrain_eta: f64,
}

impl RiderBike {
    /// Validate CdA, Crr, and η.
    pub fn new(mass: Mass, cda: f64, crr: f64, drivetrain_eta: f64) -> Result<Self, Error> {
        if !cda.is_finite() || cda <= 0.0 || cda > 2.0 {
            return Err(Error::InvalidPhysicsParam);
        }
        if !crr.is_finite() || !(0.0..=0.05).contains(&crr) {
            return Err(Error::InvalidPhysicsParam);
        }
        if !drivetrain_eta.is_finite() || drivetrain_eta <= 0.0 || drivetrain_eta > 1.0 {
            return Err(Error::InvalidPhysicsParam);
        }
        Ok(Self {
            mass,
            cda,
            crr,
            drivetrain_eta,
        })
    }

    /// Road defaults: CdA 0.30, Crr 0.004, η 0.97.
    pub fn road_default(mass: Mass) -> Self {
        Self {
            mass,
            cda: DEFAULT_CDA,
            crr: DEFAULT_CRR,
            drivetrain_eta: DEFAULT_ETA,
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RiderBike {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("RiderBike", 4)?;
        state.serialize_field("mass_kg", &self.mass.kg())?;
        state.serialize_field("cda", &self.cda)?;
        state.serialize_field("crr", &self.crr)?;
        state.serialize_field("drivetrain_eta", &self.drivetrain_eta)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RiderBike {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Helper {
            mass_kg: f64,
            cda: f64,
            crr: f64,
            drivetrain_eta: f64,
        }
        let h = Helper::deserialize(deserializer)?;
        let mass = Mass::from_kg(h.mass_kg).map_err(serde::de::Error::custom)?;
        RiderBike::new(mass, h.cda, h.crr, h.drivetrain_eta).map_err(serde::de::Error::custom)
    }
}

/// Environmental conditions for the power balance.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Environment {
    /// Air density ρ in kg/m³.
    pub air_density: f64,
    /// Wind speed in m/s; positive = headwind.
    pub wind_ms: f64,
    /// Road grade as a fraction (0.06 = 6%).
    pub grade: f64,
}

impl Environment {
    /// Flat, calm, sea-level defaults.
    pub fn flat_calm_sea_level() -> Self {
        Self {
            air_density: SEA_LEVEL_RHO,
            wind_ms: 0.0,
            grade: 0.0,
        }
    }

    /// Flat, calm conditions with air density from altitude and temperature.
    pub fn from_altitude_celsius(altitude_m: f64, temp_c: f64) -> Result<Self, Error> {
        Ok(Self {
            air_density: air_density(altitude_m, temp_c)?,
            wind_ms: 0.0,
            grade: 0.0,
        })
    }

    fn validate(self) -> Result<(), Error> {
        if !self.air_density.is_finite() || self.air_density <= 0.0 {
            return Err(Error::InvalidPhysicsParam);
        }
        if !self.wind_ms.is_finite() {
            return Err(Error::InvalidPhysicsParam);
        }
        if !self.grade.is_finite() || self.grade.abs() > 1.0 {
            return Err(Error::InvalidPhysicsParam);
        }
        Ok(())
    }
}

/// Approximate air density (kg/m³) from altitude and Celsius temperature.
///
/// Uses ISA tropospheric pressure with an ideal-gas correction for `temp_c`.
pub fn air_density(altitude_m: f64, temp_c: f64) -> Result<f64, Error> {
    if !altitude_m.is_finite() || !(-500.0..=9000.0).contains(&altitude_m) {
        return Err(Error::InvalidPhysicsParam);
    }
    if !temp_c.is_finite() || !(-60.0..=55.0).contains(&temp_c) {
        return Err(Error::InvalidPhysicsParam);
    }
    let temp_k = temp_c + 273.15;
    let pressure = 101_325.0 * (1.0 - 2.25577e-5 * altitude_m).powf(5.25588);
    if !pressure.is_finite() || pressure <= 0.0 {
        return Err(Error::InvalidPhysicsParam);
    }
    let rho = pressure / (287.05 * temp_k);
    if !rho.is_finite() || rho <= 0.0 {
        return Err(Error::InvalidPhysicsParam);
    }
    Ok(rho)
}

fn theta(grade: f64) -> f64 {
    grade.atan()
}

/// Pedal power required to hold ground speed `speed_m_s` (m/s).
pub fn power_for_speed(rider: RiderBike, env: Environment, speed_m_s: f64) -> Result<Power, Error> {
    env.validate()?;
    if !speed_m_s.is_finite() || speed_m_s <= 0.0 {
        return Err(Error::InvalidPhysicsParam);
    }
    let m = rider.mass.kg();
    let th = theta(env.grade);
    let v = speed_m_s;
    let vw = env.wind_ms;
    let aero = 0.5 * rider.cda * env.air_density * (v + vw).powi(2) * v;
    let roll = rider.crr * m * G * th.cos() * v;
    let grav = m * G * th.sin() * v;
    let p_pedal = (aero + roll + grav) / rider.drivetrain_eta;
    if !p_pedal.is_finite() {
        return Err(Error::InvalidPhysicsParam);
    }
    // Downhill with low power demand can be ≤ 0; no coasting model.
    if p_pedal <= 0.0 {
        return Err(Error::UnsolvableSpeed);
    }
    Power::new(p_pedal)
}

/// Ground speed (m/s) for a given pedal power (bisection on v ∈ (0.1, 30)).
pub fn speed_for_power(rider: RiderBike, env: Environment, power: Power) -> Result<f64, Error> {
    env.validate()?;
    let target = power.watts();
    let mut lo = 0.1;
    let mut hi = 30.0;
    let p_lo = match power_for_speed(rider, env, lo) {
        Ok(p) => p.watts(),
        Err(Error::UnsolvableSpeed) => 0.0,
        Err(e) => return Err(e),
    };
    let p_hi = power_for_speed(rider, env, hi)?.watts();
    if target < p_lo || target > p_hi {
        return Err(Error::UnsolvableSpeed);
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let p_mid = power_for_speed(rider, env, mid)?.watts();
        if p_mid < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

/// Time to cover `distance_m` at constant grade/wind and constant `power`.
pub fn time_for_distance(
    rider: RiderBike,
    env: Environment,
    power: Power,
    distance_m: f64,
) -> Result<Duration, Error> {
    if !distance_m.is_finite() || distance_m <= 0.0 {
        return Err(Error::InvalidPhysicsParam);
    }
    let v = speed_for_power(rider, env, power)?;
    Ok(Duration::from_secs_f64(distance_m / v))
}

/// Vertical ascent metres per hour: `ascent_m / duration * 3600`.
pub fn vam_m_per_hour(ascent_m: f64, duration: Duration) -> Result<f64, Error> {
    let t = duration.as_secs_f64();
    if !ascent_m.is_finite() || ascent_m < 0.0 || t <= 0.0 {
        return Err(Error::InvalidPhysicsParam);
    }
    Ok(ascent_m / t * 3600.0)
}

/// Climb time from W/kg, grade, distance, and total mass.
///
/// Uses [`RiderBike::road_default`] (hoods CdA). Aero is small on steep grades
/// but still wrong for a climb posture — prefer [`climb_time_with`] when you care.
pub fn climb_time(
    watts_per_kg: WattsPerKg,
    grade: f64,
    distance_m: f64,
    mass: Mass,
) -> Result<Duration, Error> {
    let rider = RiderBike::road_default(mass);
    let env = Environment {
        air_density: SEA_LEVEL_RHO,
        wind_ms: 0.0,
        grade,
    };
    climb_time_with(rider, env, watts_per_kg, distance_m)
}

/// Climb time with an explicit rider and environment (constant grade).
pub fn climb_time_with(
    rider: RiderBike,
    env: Environment,
    watts_per_kg: WattsPerKg,
    distance_m: f64,
) -> Result<Duration, Error> {
    let power = Power::new(watts_per_kg.value() * rider.mass.kg())?;
    time_for_distance(rider, env, power, distance_m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cycling")
    }

    #[test]
    fn fixture_martin_points() {
        let text = fs::read_to_string(fixtures_dir().join("physics.csv")).unwrap();
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }
            let c: Vec<_> = line.split(',').collect();
            // mass_kg,cda,crr,eta,rho,grade,wind,speed_m_s,expected_watts
            let mass = Mass::from_kg(c[0].parse().unwrap()).unwrap();
            let rider = RiderBike::new(
                mass,
                c[1].parse().unwrap(),
                c[2].parse().unwrap(),
                c[3].parse().unwrap(),
            )
            .unwrap();
            let env = Environment {
                air_density: c[4].parse().unwrap(),
                wind_ms: c[6].parse().unwrap(),
                grade: c[5].parse().unwrap(),
            };
            let speed: f64 = c[7].parse().unwrap();
            let expected: f64 = c[8].parse().unwrap();
            let p = power_for_speed(rider, env, speed).unwrap();
            assert!(
                (p.watts() - expected).abs() < 0.05,
                "row {i}: got {} want {expected}",
                p.watts()
            );
            let v_back = speed_for_power(rider, env, p).unwrap();
            assert!(
                (v_back - speed).abs() < 0.01,
                "row {i}: speed invert {v_back} vs {speed}"
            );
        }
    }

    #[test]
    fn time_for_distance_is_d_over_v() {
        let rider = RiderBike::road_default(Mass::from_kg(80.0).unwrap());
        let env = Environment::flat_calm_sea_level();
        let power = power_for_speed(rider, env, 10.0).unwrap();
        let t = time_for_distance(rider, env, power, 1000.0).unwrap();
        assert!((t.as_secs_f64() - 100.0).abs() < 0.05);
    }

    #[test]
    fn vam_300m_in_30min() {
        let vam = vam_m_per_hour(300.0, Duration::from_secs(1800)).unwrap();
        assert!((vam - 600.0).abs() < 1e-9);
    }

    #[test]
    fn unsolvable_speed_when_power_too_low_on_steep() {
        let rider = RiderBike::road_default(Mass::from_kg(80.0).unwrap());
        let env = Environment {
            air_density: SEA_LEVEL_RHO,
            wind_ms: 0.0,
            grade: 0.20,
        };
        // Tiny power cannot overcome gravity at any speed in bracket.
        let result = speed_for_power(rider, env, Power::new(1.0).unwrap());
        assert_eq!(result, Err(Error::UnsolvableSpeed));
    }

    #[test]
    fn air_density_sea_level_15c_near_isa() {
        let rho = air_density(0.0, 15.0).unwrap();
        assert!((rho - SEA_LEVEL_RHO).abs() < 0.01);
        let env = Environment::from_altitude_celsius(0.0, 15.0).unwrap();
        assert!((env.air_density - rho).abs() < 1e-12);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rider_roundtrip() {
        let rider = RiderBike::road_default(Mass::from_kg(83.0).unwrap());
        let back: RiderBike =
            serde_json::from_str(&serde_json::to_string(&rider).unwrap()).unwrap();
        assert_eq!(back, rider);
    }
}
