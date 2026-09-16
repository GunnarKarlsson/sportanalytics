//! 20:00 5K: VDOT, race predictions, and Daniels training zones.

use sportanalytics::running::{
    predict_daniels_and_cameron, predict_times, training_zones, vdot, vo2max_from_races, Distance,
    LengthUnit, PredictionModel, RaceTime,
};

fn main() {
    let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0).unwrap();
    let ten = RaceTime::from_hms(Distance::TenK, 0, 42, 0).unwrap();

    println!("VDOT from 20:00 5K: {:.1}", vdot(five).value());
    let vo2 = vo2max_from_races(&[five, ten]).unwrap();
    println!(
        "with 42:00 10K: mean {:.1}, best {:.1}",
        vo2.mean.value(),
        vo2.best.value()
    );

    let pred = predict_times(five, PredictionModel::DanielsVdot).unwrap();
    println!(
        "Daniels HM {}  FM {}",
        pred.formatted(Distance::HalfMarathon).unwrap(),
        pred.formatted(Distance::Marathon).unwrap()
    );

    let both = predict_daniels_and_cameron(five).unwrap();
    println!(
        "Cameron HM {}",
        both.cameron.formatted(Distance::HalfMarathon).unwrap()
    );

    let z = training_zones(five);
    println!("E {}  T {}  I {}", z.easy, z.threshold, z.interval);
    println!("E miles {}", z.easy.display(LengthUnit::Mile));
    println!("{z}");

    let eight = RaceTime::from_hms(Distance::from_miles(8.0).unwrap(), 1, 0, 0).unwrap();
    println!(
        "8 mi in 1:00:00 → {} ({:.2} km)",
        eight.pace(),
        eight.distance().kilometers()
    );
}
