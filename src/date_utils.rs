use chrono::{NaiveDate, NaiveDateTime, Utc};

pub(crate) trait DateUtils {
    fn latest_prediction_date() -> NaiveDateTime;
    fn date_to_timestamp(date: &str) -> Option<f64>;
}

impl DateUtils for NaiveDateTime {
    fn latest_prediction_date() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    ///
    /// Converts a date in the format returned by Metaculus (`YYYY-MM-DD`) into a number of non-leap
    /// seconds since midnight, January 1st, 1970, or `None` if the string is not a properly
    /// formatted date.
    ///
    fn date_to_timestamp(date: &str) -> Option<f64> {
        let date_format = "%Y-%m-%d";
        Some(
            NaiveDate::parse_from_str(date, date_format)
                .ok()?
                .and_hms(0, 0, 0)
                .timestamp() as f64,
        )
    }
}