use chrono::{NaiveDate, NaiveDateTime};

use crate::utils::read_q_from_file;
use metaculustetra::Prediction::{AmbP, DatP, NumP};

mod utils;

#[test]
fn test_resolved_probability_question() {
    let question = read_q_from_file("resolved_probability_example");

    assert_eq!(
        question.title_short,
        "Infrastructure package passed in 2021"
    );
    assert_eq!(question.get_resolution().unwrap(), NumP(1.0));
    assert_eq!(question.get_community_prediction().unwrap(), NumP(0.99));
    assert_eq!(
        question.get_metaculus_prediction().unwrap(),
        NumP(0.986684322309624)
    );
    assert_eq!(question.get_best_prediction().unwrap(), NumP(1.0));
}

#[test]
fn test_unresolved_probability_question() {
    let question = read_q_from_file("probability_example");

    assert_eq!(
        question.title_short,
        "Global population to fall by >10% by 2100?"
    );
    assert_eq!(question.get_resolution(), None);
    assert_eq!(question.get_community_prediction().unwrap(), NumP(0.2));
    assert_eq!(question.get_metaculus_prediction(), None);
    assert_eq!(question.get_best_prediction().unwrap(), NumP(0.2));
}

#[test]
fn test_resolved_range_question() {
    let question = read_q_from_file("resolved_range_example");
    assert_eq!(
        question.get_resolution().unwrap(),
        NumP(0.44 * (30.0 - -20.0) + -20.0)
    );
    assert_eq!(
        question.get_metaculus_prediction().unwrap(),
        NumP(0.49315 * (30.0 - -20.0) + -20.0)
    );
    assert_eq!(
        question.get_community_prediction().unwrap(),
        NumP(0.49337 * (30.0 - -20.0) + -20.0)
    );
    assert_eq!(
        question.get_best_prediction().unwrap(),
        NumP(0.44 * (30.0 - -20.0) + -20.0)
    );
}

#[test]
fn test_range_question() {
    let question = read_q_from_file("range_example");
    assert_eq!(question.get_community_prediction().unwrap(), NumP(0.1132));
    assert_eq!(question.get_best_prediction().unwrap(), NumP(0.1132));
    assert_eq!(question.get_metaculus_prediction(), None);
    assert_eq!(question.get_resolution(), None);
}

#[test]
fn test_logarithmic_range_question() {
    let question = read_q_from_file("logarithmic_range_example");

    let community_prediction =
        ((100000000000000000000000000.0 / 1000000000000.0) as f64).powf(0.41079) * 1000000000000.0;

    assert_eq!(
        question.get_best_prediction().unwrap(),
        NumP(community_prediction)
    );
    assert_eq!(
        question.get_community_prediction().unwrap(),
        NumP(community_prediction)
    );
}

#[test]
fn test_date_range_question() {
    let question = read_q_from_file("date_range_example");

    let start_date = NaiveDate::parse_from_str("2021-01-15", "%Y-%m-%d")
        .unwrap()
        .and_hms(0, 0, 0)
        .timestamp() as f64;
    let end_date = NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d")
        .unwrap()
        .and_hms(0, 0, 0)
        .timestamp() as f64;

    let community_date =
        NaiveDateTime::from_timestamp((0.27891 * (end_date - start_date) + start_date) as i64, 0);

    assert_eq!(
        question.get_best_prediction().unwrap(),
        DatP(community_date)
    );
    assert_eq!(
        question.get_community_prediction().unwrap(),
        DatP(community_date)
    );
}

#[test]
fn test_logarithmic_date_range_question() {
    let question = read_q_from_file("logarithmic_date_range_example");

    let start_date = NaiveDate::parse_from_str("2020-03-27", "%Y-%m-%d")
        .unwrap()
        .and_hms(0, 0, 0)
        .timestamp() as f64;
    let end_date = NaiveDate::parse_from_str("2200-01-04", "%Y-%m-%d")
        .unwrap()
        .and_hms(0, 0, 0)
        .timestamp() as f64;

    let community_date = NaiveDateTime::from_timestamp(
        ((end_date / start_date).powf(0.70277) * start_date) as i64,
        0,
    );

    assert_eq!(
        question.get_best_prediction().unwrap(),
        DatP(community_date)
    );
    assert_eq!(
        question.get_community_prediction().unwrap(),
        DatP(community_date)
    );
}

#[test]
fn test_discussion_question() {
    let question = read_q_from_file("discussion_example");
    assert_eq!(question.get_best_prediction(), None);
    assert_eq!(question.get_resolution(), None);
    assert_eq!(question.get_metaculus_prediction(), None);
    assert_eq!(question.get_community_prediction(), None);
}

#[test]
fn test_ambiguously_resolved_question() {
    let question = read_q_from_file("ambiguously_resolved_example");
    assert_eq!(question.get_best_prediction().unwrap(), AmbP);
    assert_eq!(question.get_resolution().unwrap(), AmbP);
    assert_eq!(question.get_community_prediction().unwrap(), NumP(0.05));
    assert_eq!(
        question.get_metaculus_prediction().unwrap(),
        NumP(0.05758051341132789)
    );
}

#[test]
fn test_ambiguously_resolved_range_question() {
    let question = read_q_from_file("ambiguously_resolved_range_example");
    assert_eq!(question.get_best_prediction().unwrap(), AmbP);
    assert_eq!(question.get_resolution().unwrap(), AmbP);
    assert_eq!(question.get_community_prediction().unwrap(), NumP(40.749));
    assert_eq!(question.get_metaculus_prediction().unwrap(), NumP(38.73));
}

#[test]
fn test_unrevealed_question() {
    let question = read_q_from_file("tournament_example");
    assert_eq!(question.get_best_prediction().unwrap(), NumP(0.3));
    assert_eq!(question.get_community_prediction().unwrap(), NumP(0.3));
}

#[test]
fn test_get_if() {
    let ambiguous_prediction = AmbP;
    let date_prediction = DatP(NaiveDate::from_ymd(1954, 03, 02).and_hms(0, 0, 0));
    let numerical_prediction = NumP(42.0);

    assert_eq!(ambiguous_prediction.get_if_numeric(), None);
    assert_eq!(ambiguous_prediction.get_if_date(), None);

    assert_eq!(date_prediction.get_if_numeric(), None);
    assert_eq!(
        date_prediction.get_if_date(),
        Some(NaiveDate::from_ymd(1954, 03, 02).and_hms(0, 0, 0))
    );

    assert_eq!(numerical_prediction.get_if_numeric(), Some(42.0));
    assert_eq!(numerical_prediction.get_if_date(), None);
}

#[test]
fn test_get_community_prediction_before() {
    let question = read_q_from_file("probability_example");
    assert_eq!(
        question.get_community_prediction_before(NaiveDate::from_ymd(1945, 1, 1).and_hms(0, 0, 0)),
        None
    );
    assert_eq!(
        question
            .get_community_prediction_before(NaiveDate::from_ymd(2018, 10, 12).and_hms(0, 0, 0))
            .unwrap(),
        NumP(0.2)
    );
    assert_eq!(
        question
            .get_community_prediction_before(NaiveDate::from_ymd(2019, 3, 6).and_hms(0, 0, 0))
            .unwrap(),
        NumP(0.38)
    );
    assert_eq!(
        question
            .get_community_prediction_before(NaiveDate::from_ymd(2100, 1, 1).and_hms(0, 0, 0))
            .unwrap(),
        NumP(0.2)
    );
}
