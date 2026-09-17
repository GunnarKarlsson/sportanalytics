//! Age-grade a 20:00 5K for a 42-year-old, with an equivalent at age 25.

use sportanalytics::running::{age_grade, Distance, Gender, RaceTime};

fn main() -> Result<(), sportanalytics::running::Error> {
    let five = RaceTime::from_hms(Distance::FiveK, 0, 20, 0)?;
    let ag = age_grade(five, 42, Gender::Male, Some(25))?;
    println!(
        "{:.1}% {} | open eq {} | as 25yo {}",
        ag.percent,
        ag.level.label(),
        ag.open_equivalent_hms(),
        ag.equivalent_at_age_hms().unwrap()
    );
    Ok(())
}
