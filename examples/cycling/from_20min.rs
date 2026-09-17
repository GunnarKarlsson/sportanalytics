//! 20 min @ 280 W and 5 min @ 340 W: FTP, zones, CP, VO2, age factor.

use std::time::Duration;

use sportanalytics::cycling::{
    age_equivalent_ftp, critical_power, estimated_vo2max, ftp_from_20min, predict_power_from_cp,
    training_zones, Effort, Mass,
};

fn main() -> Result<(), sportanalytics::cycling::Error> {
    let twenty = Effort::from_watts_secs(280.0, 20.0 * 60.0)?;
    let five = Effort::from_watts_secs(340.0, 5.0 * 60.0)?;
    let mass = Mass::from_kg(75.0)?;

    let ftp = ftp_from_20min(twenty)?;
    let zones = training_zones(ftp);
    println!("{ftp} | W/kg {:.2}", twenty.watts_per_kg(mass).value());
    println!("Z2 {}  Z4 {}", zones.endurance, zones.threshold);

    let cp = critical_power(&[five, twenty])?;
    println!(
        "CP {:.1} W  W′ {:.1} kJ  R² {:.4}",
        cp.model.cp.watts(),
        cp.model.w_prime.kj(),
        cp.r_squared
    );

    let hour = predict_power_from_cp(cp.model, Duration::from_secs(3600))?;
    println!("Predicted 60 min power: {}", hour);

    let vo2 = estimated_vo2max(five.power(), mass);
    println!("ACSM VO2 from 5 min MAP: {vo2}");

    let eq = age_equivalent_ftp(ftp, 50, 35)?;
    println!("Age-equivalent FTP at 35 (from age 50): {eq}");

    Ok(())
}
