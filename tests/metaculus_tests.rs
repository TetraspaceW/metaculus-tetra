use std::fs::File;
use std::io::BufReader;

use metaculustetra::*;

fn read_file(filename: &str) -> Question {
    let file = File::open(format!("tests/{}.json", filename)).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

#[test]
fn test_resolved_probability_question() {
    let question = read_file("resolved_probability_example");

    assert_eq!(
        question.title_short,
        "Infrastructure package passed in 2021"
    );
    assert_eq!(question.get_resolution(), Some(1.0));
    assert_eq!(question.get_community_prediction(), Some(0.99));
    assert_eq!(question.get_metaculus_prediction(), Some(0.986684322309624));
    assert_eq!(question.get_best_prediction(), Some(1.0));
}

#[test]
fn test_unresolved_probability_question() {
    let question = read_file("probability_example");

    assert_eq!(
        question.title_short,
        "Global population to fall by >10% by 2100?"
    );
    assert_eq!(question.get_resolution(), None);
    assert_eq!(question.get_community_prediction(), Some(0.2));
    assert_eq!(question.get_metaculus_prediction(), None);
    assert_eq!(question.get_best_prediction(), Some(0.2));
}

#[test]
fn test_resolved_range_question() {
    let question = read_file("resolved_range_example");
    assert_eq!(question.get_resolution(), Some(0.44 * (30.0 - -20.0) + -20.0));
    assert_eq!(question.get_metaculus_prediction(), Some(0.49315 * (30.0 - -20.0) + -20.0));
    assert_eq!(question.get_community_prediction(), Some(0.49337 * (30.0 - -20.0) + -20.0));
    assert_eq!(question.get_best_prediction(), Some(0.44 * (30.0 - -20.0) + -20.0));
}

#[test]
fn test_range_question() {
    let question = read_file("range_example");
    assert_eq!(question.get_community_prediction(), Some(0.1132));
    assert_eq!(question.get_best_prediction(), Some(0.1132));
    assert_eq!(question.get_metaculus_prediction(), None);
    assert_eq!(question.get_resolution(), None);
}

#[test]
fn test_logarithmic_range_question() {
    let question = read_file("logarithmic_range_example");
}

#[test]
fn test_date_range_question() {
    let question = read_file("date_range_example");
}

#[test]
fn test_logarithmic_date_range_question() {
    let question = read_file("logarithmic_date_range_example");
}

#[test]
fn test_discussion_question() {
    let question = read_file("discussion_example");
}

#[test]
fn test_ambiguously_resolved_question() {
    let question = read_file("ambiguously_resolved_example");
}
