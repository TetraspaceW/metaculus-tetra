use chrono::{TimeZone, Utc};
use env_logger;
use log::{info, LevelFilter};

use metaculustetra::*;

use crate::ignign_send_tweet::send_tweet;

mod ignign_send_tweet;

fn main() {
    setup_logger();

    let m = Metaculus::new();
    retrieve_xrisk_questions(m);
}

fn setup_logger() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
}

fn retrieve_xrisk_questions(m: Metaculus) {
    let catastrophe_question_ids = vec!["1500", "1494", "1495", "1502", "1501"].into_iter();
    let existential_question_ids = vec!["1604", "1585", "2513", "2514", "7795"].into_iter();

    let cat_overall = m.get_prediction_for("1493").unwrap();

    let x_total: f64 = catastrophe_question_ids
        .map(|id| m.get_prediction_for(id).unwrap() * cat_overall)
        .zip(existential_question_ids.map(|id| m.get_prediction_for(id).unwrap()))
        .map(|p| p.0 * p.1)
        .sum();

    let human_lifespan = 350000.0;
    let years_until_resolution =
        (Utc.ymd(2100, 1, 1) - Utc::today()).num_seconds() as f64 / (365.25 * 86400.0);

    let real_years_until_end = years_until_resolution / (1.0 / (1.0 - x_total)).ln();

    let clock_seconds_until_end =
        real_years_until_end / (real_years_until_end + human_lifespan) * 86400.0;
    let minutes = (clock_seconds_until_end / 60.0).floor();
    let seconds = clock_seconds_until_end - minutes * 60.0;

    let time_until_end = format!("{:02}:{:05.2}", minutes, seconds);
    let tweet_text = format!("[Bot Tweet] The Doomsday clock is currently at {} until midnight, from a Metaculus community median prediction of a {}% chance of extinction this century (https://www.metaculus.com/questions/2568/ragnar%25C3%25B6k-seriesresults-so-far/).", time_until_end, x_total*100.0);

    info!("Sending tweet with text {}", tweet_text);

    send_tweet(tweet_text);
}
