//! Road speed and climb time from power (Martin 1998 balance).

use std::time::Duration;

use sportanalytics::cycling::{
    climb_time, power_for_speed, speed_for_power, time_for_distance, vam_m_per_hour, Environment,
    Mass, Power, RiderBike, WattsPerKg,
};

fn main() -> Result<(), sportanalytics::Error> {
    let mass = Mass::from_kg(75.0 + 8.0)?; // rider + bike
    let rider = RiderBike::road_default(mass);
    let env = Environment::flat_calm_sea_level();
    let power = Power::new(250.0)?;

    let v = speed_for_power(rider, env, power)?;
    let kmh = v * 3.6;
    let forty_km = time_for_distance(rider, env, power, 40_000.0)?;
    println!(
        "250 W flat calm: {:.2} km/h — 40 km in {}",
        kmh,
        format_hms(forty_km)
    );
    let p_check = power_for_speed(rider, env, v)?;
    println!("round-trip power check: {p_check}");

    let wkg = WattsPerKg::new(4.0)?;
    let climb = climb_time(wkg, 0.08, 5_000.0, mass)?;
    let ascent = 5_000.0 * 0.08;
    let vam = vam_m_per_hour(ascent, climb)?;
    println!(
        "4.0 W/kg on 8% / 5 km: {} — VAM {:.0} m/h",
        format_hms(climb),
        vam
    );

    Ok(())
}

fn format_hms(d: Duration) -> String {
    let total = d.as_secs_f64().round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
