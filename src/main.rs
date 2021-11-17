use env_logger;
use log::LevelFilter;
use crate::metaculus::Metaculus;

mod metaculus;

fn main() {
    setup_logger();
    retrieve_xrisk_questions();
}

fn setup_logger() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
}

fn retrieve_xrisk_questions() {
    let m = Metaculus::new();

    let catastrophe_question_ids = vec!["1500", "1494", "1495", "1502", "1501"].into_iter();
    let existential_question_ids = vec!["1604", "1585", "2513", "2514", "7795"].into_iter();

    let cat_overall = m.get_prediction_for("1493").unwrap();

    let x_total: f64 = catastrophe_question_ids
        .map(|id| m.get_prediction_for(id).unwrap() * cat_overall)
        .zip(existential_question_ids.map(|id| m.get_prediction_for(id).unwrap()))
        .map(|p| p.0 * p.1)
        .sum();

    println!(
        "Metaculus community estimates a total existential risk of {}%.",
        x_total * 100.0
    )
}