//! A library for interacting with Metaculus.
//!
//! [Repository](https://github.com/TetraspaceW/metaculus-tetra)
//!

pub mod index;
mod date_utils;

use crate::MetaculusPredictionTimeseriesPoint::{NumericMPTP, RangeMPTP};
use crate::Prediction::{AmbP, DatP, NumP};
use crate::PredictionTimeseriesPoint::{NumericPTP, RangePTP};
use crate::RangeQuestionScale::{DateRangeQuestionScale, NumericRangeQuestionScale};
use chrono::NaiveDateTime;
use log::info;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use crate::date_utils::DateUtils;

///
/// An API client for retrieving Metaculus question data . Contains the domain (e.g. `www`,
/// `pandemic`, `ai`) of the Metaculus instance and the blocking reqwest client.
///
/// # Example
///
/// ``` rust
/// use metaculustetra::Metaculus;
/// use reqwest::blocking::Client;
/// // Standard Metaculus client, accesses https://www.metaculus.com
/// let m = Metaculus::standard();
/// // Pandemic Metaculus client, accesses https://pandemic.metaculus.com
/// let mp = Metaculus { domain: "pandemic", client: Client::new() };
/// ```
pub struct Metaculus<'a> {
    pub domain: &'a str,
    pub client: Client,
}

impl Metaculus<'_> {
    pub fn standard() -> Metaculus<'static> {
        Metaculus {
            domain: "www",
            client: Client::new(),
        }
    }

    ///
    /// Returns the numeric prediction for the question `id` if it is a numerical question, and
    /// `None` otherwise.
    ///
    pub fn get_numeric_prediction_for(&self, id: &str) -> Option<f64> {
        self.get_prediction_for(id)?.get_if_numeric()
    }

    ///
    /// Returns the date prediction for the question `id` if it is a numerical question, and `None`
    /// otherwise.
    ///
    pub fn get_date_prediction_for(&self, id: &str) -> Option<NaiveDateTime> {
        self.get_prediction_for(id)?.get_if_date()
    }

    ///
    /// Returns the best prediction available (prioritising the actual resolution, then the Metaculus
    /// prediction, then the community prediction) far the question with id `id` as a [Prediction],
    /// if the question exists and has any predictions.
    ///
    pub fn get_prediction_for(&self, id: &str) -> Option<Prediction> {
        self.get_question(id)?.get_best_prediction()
    }

    ///
    /// Returns the question with id `id` as a [Question] if it exists.
    ///
    pub fn get_question(&self, id: &str) -> Option<Question> {
        let url = format!(
            "https://{}.metaculus.com/api2/questions/{}",
            self.domain, id
        );
        let response = self.client.get(url).send().ok()?.text().ok()?;
        let question_response = serde_json::from_str(&response).ok()?;
        info!("Question id {} retrieved successfully.", id);

        return Some(question_response);
    }
}

///
/// Data on a single Metaculus question.
///
#[derive(Serialize, Deserialize, Clone)]
pub struct Question {
    pub title_short: String,
    prediction_timeseries: Option<Vec<PredictionTimeseriesPoint>>,
    metaculus_prediction: Option<MetaculusPrediction>,
    resolution: Option<f64>,
    possibilities: QuestionPossibilities,
    resolve_time: Option<String>
}

impl Question {
    ///
    /// Returns the best prediction available (prioritising the actual resolution, then the
    /// Metaculus prediction, then the community prediction) for the question as a [Prediction], if
    /// the question has any predictions.
    ///
    /// # Example
    /// ```rust
    /// use metaculustetra::Prediction::NumP;
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use metaculustetra::Question;
    ///
    /// // Load .json response from a file
    /// let file = File::open("tests/resolved_probability_example.json").unwrap();
    /// let reader = BufReader::new(file);
    /// let question: Question = serde_json::from_reader(reader).unwrap();
    /// // The question resolved at yes, which is 1.0 (100%).
    /// assert_eq!(question.get_resolution(), Some(NumP(1.0)));
    /// // The community median prediction was 99%.
    /// assert_eq!(question.get_community_prediction(), Some(NumP(0.99)));
    /// // The Metaculus prediction was 98.7% with a few more decimal places.
    /// assert_eq!(question.get_metaculus_prediction(), Some(NumP(0.986684322309624)));
    /// // The best available prediction is the resolution, 100%.
    /// assert_eq!(question.get_best_prediction(), Some(NumP(1.0)));
    /// ```
    ///
    pub fn get_best_prediction(&self) -> Option<Prediction> {
        self.get_best_prediction_before(NaiveDateTime::latest_prediction_date())
    }

    pub fn get_best_prediction_before(&self, date: NaiveDateTime) -> Option<Prediction> {
        return match self.get_resolution_before(date) {
            None => match self.get_metaculus_prediction_before(date) {
                None => self.get_community_prediction_before(date),
                mp => mp,
            },
            r => r,
        };
    }

    ///
    /// Returns the community median prediction, if it exists.
    ///
    pub fn get_community_prediction(&self) -> Option<Prediction> {
        self.get_community_prediction_before(NaiveDateTime::latest_prediction_date())
    }

    ///
    /// Returns the Metaculus prediction, if it exists and is available.
    ///
    pub fn get_metaculus_prediction(&self) -> Option<Prediction> {
        self.get_metaculus_prediction_before(NaiveDateTime::latest_prediction_date())
    }

    fn convert_range_prediction(&self, prediction: f64) -> Option<Prediction> {
        let scale = self.possibilities.scale.as_ref()?;

        match scale {
            NumericRangeQuestionScale { min, max, .. } => {
                Some(NumP(self.scale_range_prediction(prediction, *min, *max)))
            }
            DateRangeQuestionScale { min, max, .. } => {
                let min_ts = NaiveDateTime::date_to_timestamp(min)?;
                let max_ts = NaiveDateTime::date_to_timestamp(max)?;
                Some(DatP(NaiveDateTime::from_timestamp(
                    self.scale_range_prediction(prediction, min_ts, max_ts) as i64,
                    0,
                )))
            }
        }
    }

    fn scale_range_prediction(&self, prediction: f64, min: f64, max: f64) -> f64 {
        return if self.is_logarithmic() {
            (max / min).powf(prediction) * min
        } else {
            prediction * (max - min) + min
        };
    }

    ///
    /// Returns the question resolution, if it exists. This will be a [Prediction::AmbP] if the
    /// question has resolved ambiguously.
    ///
    pub fn get_resolution(&self) -> Option<Prediction> {
        return if self.resolution? == -1.0 {
            Some(AmbP)
        } else if self.possibilities.question_type == "continuous" {
            Some(self.convert_range_prediction(self.resolution?)?)
        } else {
            Some(NumP(self.resolution?))
        };
    }

    pub fn is_logarithmic(&self) -> bool {
        match self.possibilities.scale {
            Some(NumericRangeQuestionScale { deriv_ratio, .. }) => deriv_ratio != 1.0 as f64,
            Some(DateRangeQuestionScale { deriv_ratio, .. }) => deriv_ratio != 1.0 as f64,
            None => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        self.possibilities.question_type == String::from("binary")
    }

    pub fn get_community_prediction_before(&self, date: NaiveDateTime) -> Option<Prediction> {
        let mut predictions = self.prediction_timeseries.as_ref()?.clone();
        predictions.reverse();
        match predictions
            .iter()
            .find(|it| it.timestamp() <= date.timestamp() as f64)?
        {
            NumericPTP {
                community_prediction,
                ..
            } => Some(NumP(*community_prediction)),
            RangePTP {
                community_prediction,
                ..
            } => self.convert_range_prediction(community_prediction.q2),
        }
    }

    pub fn get_metaculus_prediction_before(&self, date: NaiveDateTime) -> Option<Prediction> {
        let mut metaculus_predictions = self.metaculus_prediction.as_ref()?.history.clone();
        metaculus_predictions.reverse();
        match metaculus_predictions
            .iter()
            .find(|it| it.timestamp() <= date.timestamp() as f64)?
        {
            NumericMPTP { x, .. } => Some(NumP(*x)),
            RangeMPTP { x, .. } => self.convert_range_prediction(x.q2),
        }
    }

    fn get_resolution_before(&self, date: NaiveDateTime) -> Option<Prediction> {
        if NaiveDateTime::parse_from_str(&*self.resolve_time.as_ref()?, "%Y-%m-%dT%H:%M:%SZ").ok()? <= date {
            self.get_resolution()
        } else {
            None
        }
    }
}

///
/// An aggregated overall prediction on a Metaculus question.
///
#[derive(PartialEq, Debug)]
pub enum Prediction {
    /// Represents an Ambiguous resolution.
    AmbP,
    /// Represents a numeric prediction, either a probability (from 0.0 to 1.0) or continuous.
    NumP(f64),
    /// Represents a date prediction.
    DatP(NaiveDateTime),
}

impl Prediction {
    pub fn get_if_numeric(&self) -> Option<f64> {
        match self {
            NumP(p) => Some(*p),
            _ => None,
        }
    }

    pub fn get_if_date(&self) -> Option<NaiveDateTime> {
        match self {
            DatP(p) => Some(*p),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
enum PredictionTimeseriesPoint {
    NumericPTP {
        t: f64,
        community_prediction: f64,
    },
    RangePTP {
        t: f64,
        community_prediction: RangeCommunityPrediction,
    },
}

impl PredictionTimeseriesPoint {
    fn timestamp(&self) -> f64 {
        match self {
            NumericPTP { t, .. } => *t,
            RangePTP { t, .. } => *t,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RangeCommunityPrediction {
    q2: f64,
}

#[derive(Serialize, Deserialize, Clone)]
struct MetaculusPrediction {
    history: Vec<MetaculusPredictionTimeseriesPoint>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum MetaculusPredictionTimeseriesPoint {
    NumericMPTP { t: f64, x: f64 },
    RangeMPTP { t: f64, x: RangeMetaculusPrediction },
}

impl MetaculusPredictionTimeseriesPoint {
    fn timestamp(&self) -> f64 {
        match self {
            NumericMPTP { t, .. } => *t,
            RangeMPTP { t, .. } => *t,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct RangeMetaculusPrediction {
    q2: f64,
}

#[derive(Serialize, Deserialize, Clone)]
struct QuestionPossibilities {
    #[serde(rename = "type")]
    question_type: String,
    scale: Option<RangeQuestionScale>,
    format: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum RangeQuestionScale {
    NumericRangeQuestionScale {
        min: f64,
        max: f64,
        deriv_ratio: f64,
    },
    DateRangeQuestionScale {
        min: String,
        max: String,
        deriv_ratio: f64,
    },
}
