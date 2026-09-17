//! 400/200 SCM Free: CSS, training zones, Riegel prediction, WA points sample.

use sportanalytics::swimming::{
    css_from_trials, predict_times, training_zones_from_css, wa_points_with, Course, Event,
    PredictionModel, Stroke, SwimTime,
};

fn main() {
    let t200 =
        SwimTime::from_hms_cents(Event::M200, Course::Scm, Stroke::Free, 0, 2, 30, 0).unwrap();
    let t400 =
        SwimTime::from_hms_cents(Event::M400, Course::Scm, Stroke::Free, 0, 5, 20, 0).unwrap();
    let css = css_from_trials(t200, t400).unwrap();
    println!("CSS {:.1} s/100m", css.pace().unwrap().sec_per_100m());
    let z = training_zones_from_css(css).unwrap();
    println!("threshold {}", z.threshold);
    let pred = predict_times(t400, PredictionModel::Riegel).unwrap();
    println!("Riegel 1500 {}", pred.formatted(Event::M1500).unwrap());
    println!(
        "WA pts sample {}",
        wa_points_with(t400.seconds(), 220.0).unwrap()
    );
}
