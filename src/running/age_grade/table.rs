//! Lookup against embedded [`AgeGradeTable`] data.

use super::mldr_2025::{EventTable, EVENTS, MAX_AGE, MIN_AGE};
use super::{AgeGradeTable, Gender};
use crate::running::Distance;
use crate::Error;

/// Resolved age factor and age standard for one lookup.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Lookup {
    /// Age factor (`open / age_standard`).
    pub factor: f64,
    /// Age standard seconds (`open / factor`).
    pub age_standard_secs: f64,
}

pub(crate) fn check_age(age: u8) -> Result<(), Error> {
    if (MIN_AGE..=MAX_AGE).contains(&age) {
        Ok(())
    } else {
        Err(Error::AgeOutOfRange)
    }
}

fn factor_at(event: &EventTable, age: u8, gender: Gender) -> f64 {
    let idx = (age - MIN_AGE) as usize;
    match gender {
        Gender::Male => event.factor_male[idx],
        Gender::Female => event.factor_female[idx],
    }
}

fn open_at(event: &EventTable, gender: Gender) -> f64 {
    match gender {
        Gender::Male => event.open_male,
        Gender::Female => event.open_female,
    }
}

fn age_standard_at(event: &EventTable, age: u8, gender: Gender) -> f64 {
    open_at(event, gender) / factor_at(event, age, gender)
}

fn find_exact(meters: f64) -> Option<&'static EventTable> {
    const EPS: f64 = 1e-6;
    EVENTS.iter().find(|e| (e.meters - meters).abs() < EPS)
}

fn bracketing(meters: f64) -> Result<(&'static EventTable, &'static EventTable), Error> {
    if meters <= EVENTS[0].meters || meters >= EVENTS[EVENTS.len() - 1].meters {
        return Err(Error::UnsupportedAgeGradeDistance);
    }
    for pair in EVENTS.windows(2) {
        if meters > pair[0].meters && meters < pair[1].meters {
            return Ok((&pair[0], &pair[1]));
        }
    }
    Err(Error::UnsupportedAgeGradeDistance)
}

/// Jones 2025 log-distance weight between neighbouring official events.
fn log_u(d: f64, d0: f64, d1: f64) -> f64 {
    (d.ln() - d0.ln()) / (d1.ln() - d0.ln())
}

pub(crate) fn lookup(
    table: AgeGradeTable,
    distance: Distance,
    age: u8,
    gender: Gender,
) -> Result<Lookup, Error> {
    match table {
        AgeGradeTable::UsatfMldr2025 => lookup_mldr2025(distance, age, gender),
    }
}

fn lookup_mldr2025(distance: Distance, age: u8, gender: Gender) -> Result<Lookup, Error> {
    check_age(age)?;
    // 3K is not in the 2025 road zip (Jones briefly added then removed 1K/3K).
    if matches!(distance, Distance::ThreeK) {
        return Err(Error::UnsupportedAgeGradeDistance);
    }
    let meters = distance.meters();
    if let Some(event) = find_exact(meters) {
        let open_secs = open_at(event, gender);
        let factor = factor_at(event, age, gender);
        return Ok(Lookup {
            factor,
            age_standard_secs: open_secs / factor,
        });
    }
    let (e0, e1) = bracketing(meters)?;
    let u = log_u(meters, e0.meters, e1.meters);
    let s0 = age_standard_at(e0, age, gender);
    let s1 = age_standard_at(e1, age, gender);
    let age_standard_secs = s0 * (1.0 - u) + s1 * u;
    let open0 = open_at(e0, gender);
    let open1 = open_at(e1, gender);
    let open_secs = open0 * (1.0 - u) + open1 * u;
    let factor = open_secs / age_standard_secs;
    Ok(Lookup {
        factor,
        age_standard_secs,
    })
}

pub(crate) fn open_standard_mldr2025(distance: Distance, gender: Gender) -> Result<f64, Error> {
    if matches!(distance, Distance::ThreeK) {
        return Err(Error::UnsupportedAgeGradeDistance);
    }
    let meters = distance.meters();
    if let Some(event) = find_exact(meters) {
        return Ok(open_at(event, gender));
    }
    let (e0, e1) = bracketing(meters)?;
    let u = log_u(meters, e0.meters, e1.meters);
    Ok(open_at(e0, gender) * (1.0 - u) + open_at(e1, gender) * u)
}
